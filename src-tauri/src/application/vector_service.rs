use std::sync::Mutex;

use crate::infrastructure::{
    ollama_client::LocalOllamaClient,
    storage_error::StorageError,
    vector_repository::{LanceVectorRepository, VectorChunk, VectorSearchHit},
};

const CHUNK_TARGET_CHARS: usize = 1_200;
const CHUNK_OVERLAP_CHARS: usize = 180;

pub struct VectorService {
    repository: Mutex<LanceVectorRepository>,
}

impl VectorService {
    pub fn new(repository: LanceVectorRepository) -> Self {
        Self { repository: Mutex::new(repository) }
    }

    pub fn index_document(&self, upload_id: &str, text: &str) -> Result<usize, StorageError> {
        let chunks = chunk_text(text);
        let mut vector_chunks = Vec::with_capacity(chunks.len());
        for (index, chunk) in chunks.iter().enumerate() {
            let embedding = LocalOllamaClient::embed(chunk.text)?;
            vector_chunks.push(VectorChunk {
                upload_id: upload_id.to_owned(),
                chunk_index: index.try_into().unwrap_or(i32::MAX),
                start: chunk.start.try_into().unwrap_or(i32::MAX),
                end: chunk.end.try_into().unwrap_or(i32::MAX),
                embedding,
            });
        }
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.replace_upload_chunks(upload_id, &vector_chunks)?;
        Ok(vector_chunks.len())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<VectorSearchHit>, StorageError> {
        let embedding = LocalOllamaClient::embed(query)?;
        self.repository.lock().map_err(|_| StorageError::LockUnavailable)?.search(&embedding, limit)
    }
}

#[derive(Clone)]
struct TextChunk<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

fn chunk_text(text: &str) -> Vec<TextChunk<'_>> {
    let mut chunks = Vec::new();
    let indices = text.char_indices().map(|(index, _)| index).chain(std::iter::once(text.len())).collect::<Vec<_>>();
    if indices.len() <= 1 {
        return chunks;
    }
    let mut char_start = 0usize;
    let total_chars = indices.len() - 1;
    while char_start < total_chars {
        let char_end = (char_start + CHUNK_TARGET_CHARS).min(total_chars);
        let start = indices[char_start];
        let end = indices[char_end];
        let chunk = text[start..end].trim();
        if !chunk.is_empty() {
            chunks.push(TextChunk { text: chunk, start, end });
        }
        if char_end == total_chars {
            break;
        }
        char_start = char_end.saturating_sub(CHUNK_OVERLAP_CHARS);
    }
    chunks
}
