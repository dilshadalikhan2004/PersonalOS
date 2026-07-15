use std::sync::Mutex;

use crate::infrastructure::{
    chat_repository::{ChatCitation, ChatMessage, ChatRepository, ChatRole},
    ocr_repository::OcrResult,
    ollama_client::LocalOllamaClient,
    storage_error::StorageError,
    vector_repository::VectorSearchHit,
};

use super::{ocr_service::OcrService, vector_service::VectorService};

const MAX_CONTEXTS: usize = 5;
const MAX_HISTORY: usize = 20;
const MAX_ACCEPTED_DISTANCE: f32 = 0.85;
const MAX_LEXICAL_DOCUMENTS: usize = 200;

pub struct ChatService {
    repository: Mutex<ChatRepository>,
}

impl ChatService {
    pub fn new(repository: ChatRepository) -> Self {
        Self { repository: Mutex::new(repository) }
    }

    pub fn history(&self) -> Result<Vec<ChatMessage>, StorageError> {
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.recent(MAX_HISTORY)
    }

    pub fn answer(
        &self,
        question: String,
        ocr: &OcrService,
        vectors: &VectorService,
        on_token: impl Fn(&str),
    ) -> Result<ChatMessage, StorageError> {
        let question = question.trim().to_owned();
        if question.is_empty() {
            return Err(StorageError::InvalidModelResponse);
        }
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.append(ChatRole::User, question.clone(), Vec::new())?;
        let evidence = self.retrieve_evidence(&question, ocr, vectors)?;
        if evidence.is_empty() {
            on_token("I don't know.");
            return self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.append(ChatRole::Assistant, "I don't know.".to_owned(), Vec::new());
        }
        let citations = evidence.iter().map(|item| ChatCitation { upload_id: item.upload_id.clone(), chunk_index: item.chunk_index, excerpt: item.excerpt.clone() }).collect::<Vec<_>>();
        let answer = LocalOllamaClient::stream_generate(&prompt(&question, &evidence), |token| on_token(token))?;
        let cleaned = finalize_answer(&answer, &evidence);
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.append(ChatRole::Assistant, cleaned, citations)
    }

    fn retrieve_evidence(&self, question: &str, ocr: &OcrService, vectors: &VectorService) -> Result<Vec<Evidence>, StorageError> {
        let hits = vectors.search(question, MAX_CONTEXTS).unwrap_or_default();
        let mut evidence = Vec::new();
        for hit in hits.into_iter().filter(|hit| hit.distance <= MAX_ACCEPTED_DISTANCE).take(MAX_CONTEXTS) {
            if let Some(result) = ocr.get(&hit.upload_id)? {
                if let Some(excerpt) = excerpt(&result, &hit) {
                    evidence.push(Evidence { upload_id: hit.upload_id, chunk_index: hit.chunk_index, excerpt });
                }
            }
        }
        if evidence.is_empty() {
            evidence = lexical_evidence(question, &ocr.list(MAX_LEXICAL_DOCUMENTS)?);
        }
        Ok(evidence)
    }
}

struct Evidence {
    upload_id: String,
    chunk_index: i32,
    excerpt: String,
}

fn excerpt(result: &OcrResult, hit: &VectorSearchHit) -> Option<String> {
    let start = (hit.start.max(0) as usize).min(result.text.len());
    let end = (hit.end.max(hit.start) as usize).min(result.text.len());
    result.text.get(start..end).map(|text| text.trim().chars().take(1_800).collect::<String>()).filter(|text| !text.is_empty())
}

fn prompt(question: &str, evidence: &[Evidence]) -> String {
    let mut context = String::new();
    for item in evidence {
        context.push_str(&format!(
            "[source:{}:{}]\n{}\n\n",
            item.upload_id, item.chunk_index, item.excerpt
        ));
    }
    format!(
        "You are LifeOS, a local-only assistant. Answer the user using only the sources below. Every factual sentence must include one source marker exactly as provided. If the sources do not contain the answer, respond exactly: I don't know.\n\nSOURCES:\n{}\nQUESTION:\n{}",
        context, question
    )
}

fn finalize_answer(answer: &str, evidence: &[Evidence]) -> String {
    let answer = answer.trim();
    if answer.is_empty() || answer == "I don't know." {
        return "I don't know.".to_owned();
    }
    if answer_uses_only_allowed_citations(answer, evidence) {
        return answer.to_owned();
    }
    let sources = evidence.iter().map(|item| format!("[source:{}:{}]", item.upload_id, item.chunk_index)).take(3).collect::<Vec<_>>().join(" ");
    format!("{answer}\n\nSources: {sources}")
}

fn answer_uses_only_allowed_citations(answer: &str, evidence: &[Evidence]) -> bool {
    if answer.trim() == "I don't know." {
        return true;
    }
    let allowed = evidence.iter().map(|item| format!("[source:{}:{}]", item.upload_id, item.chunk_index)).collect::<Vec<_>>();
    let mut has_citation = false;
    let mut remainder = answer;
    while let Some(start) = remainder.find("[source:") {
        has_citation = true;
        let after_start = &remainder[start..];
        let Some(end) = after_start.find(']') else { return false };
        let marker = &after_start[..=end];
        if !allowed.iter().any(|allowed_marker| allowed_marker == marker) {
            return false;
        }
        remainder = &after_start[end + 1..];
    }
    has_citation
}

fn lexical_evidence(question: &str, documents: &[OcrResult]) -> Vec<Evidence> {
    let terms = keywords(question);
    if terms.is_empty() {
        return Vec::new();
    }
    let mut scored = documents
        .iter()
        .filter_map(|document| {
            let lowercase = document.text.to_ascii_lowercase();
            let score = terms.iter().filter(|term| lowercase.contains(term.as_str())).count();
            (score > 0).then(|| (score, document))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(right.1.created_at_unix_ms.cmp(&left.1.created_at_unix_ms)));
    scored
        .into_iter()
        .take(MAX_CONTEXTS)
        .enumerate()
        .filter_map(|(index, (_, document))| lexical_excerpt(document, &terms).map(|excerpt| Evidence { upload_id: document.upload_id.clone(), chunk_index: -(index as i32 + 1), excerpt }))
        .collect()
}

fn keywords(question: &str) -> Vec<String> {
    question
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| term.len() >= 3 && !matches!(term.as_str(), "the" | "and" | "for" | "show" | "find" | "what" | "when" | "where" | "about" | "document" | "documents"))
        .take(12)
        .collect()
}

fn lexical_excerpt(document: &OcrResult, terms: &[String]) -> Option<String> {
    let lowercase = document.text.to_ascii_lowercase();
    let start = terms.iter().filter_map(|term| lowercase.find(term)).min().unwrap_or(0);
    let safe_start = document.text[..start.min(document.text.len())].char_indices().rev().nth(240).map(|(index, _)| index).unwrap_or(0);
    let safe_end = document.text[start.min(document.text.len())..].char_indices().nth(1_200).map(|(index, _)| start + index).unwrap_or(document.text.len());
    document.text.get(safe_start..safe_end).map(|value| value.trim().to_owned()).filter(|value| !value.is_empty())
}
