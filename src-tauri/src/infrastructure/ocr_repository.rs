use std::{io::Read, path::Path, process::Command, sync::Arc};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

use super::{crypto, keychain::MasterKey, storage_error::StorageError};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrResult { pub upload_id: String, pub text: String, pub languages: String, pub source: OcrSource, pub created_at_unix_ms: i64 }

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OcrSource { EmbeddedPdfText, EmbeddedDocumentText, ImageOcr, ScannedPdfOcr }

pub struct OcrRepository { connection: Connection, master_key: Arc<MasterKey> }

impl OcrRepository {
    pub fn open(app_data: &Path, master_key: Arc<MasterKey>) -> Result<Self, StorageError> {
        let connection = Connection::open(app_data.join("lifeos.sqlite3"))?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA secure_delete = ON; CREATE TABLE IF NOT EXISTS ocr_results (upload_id TEXT PRIMARY KEY NOT NULL, encrypted_payload BLOB NOT NULL);")?;
        Ok(Self { connection, master_key })
    }
    pub fn save(&mut self, result: &OcrResult) -> Result<(), StorageError> {
        let encrypted_payload = crypto::encrypt(&self.master_key, &serde_json::to_vec(result)?)?;
        self.connection.execute("INSERT INTO ocr_results (upload_id, encrypted_payload) VALUES (?1, ?2) ON CONFLICT(upload_id) DO UPDATE SET encrypted_payload = excluded.encrypted_payload", params![result.upload_id, encrypted_payload])?;
        Ok(())
    }
    pub fn get(&self, upload_id: &str) -> Result<Option<OcrResult>, StorageError> {
        let payload: Option<Vec<u8>> = self.connection.query_row("SELECT encrypted_payload FROM ocr_results WHERE upload_id = ?1", params![upload_id], |row| row.get(0)).optional()?;
        payload.map(|value| serde_json::from_slice(&crypto::decrypt(&self.master_key, &value)?).map_err(Into::into)).transpose()
    }
    pub fn list(&self, limit: usize) -> Result<Vec<OcrResult>, StorageError> {
        let limit = limit.min(500);
        let mut statement = self.connection.prepare("SELECT encrypted_payload FROM ocr_results ORDER BY rowid DESC LIMIT ?1")?;
        let rows = statement.query_map(params![limit as i64], |row| row.get::<_, Vec<u8>>(0))?;
        let mut results = Vec::new();
        for row in rows {
            let payload = row?;
            results.push(serde_json::from_slice(&crypto::decrypt(&self.master_key, &payload)?)?);
        }
        Ok(results)
    }
}

pub struct LocalOcrEngine { binaries_directory: std::path::PathBuf, working_directory: std::path::PathBuf }

impl LocalOcrEngine {
    pub fn new(app_data: &Path) -> Self { Self { binaries_directory: app_data.join("ocr-tools"), working_directory: app_data.join("ocr-work") } }
    pub fn extract(&self, file: &Path, languages: &str, progress: impl Fn(u8, &'static str)) -> Result<(String, OcrSource), StorageError> {
        validate_languages(languages)?;
        match file.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
            "png" | "jpg" | "jpeg" => { progress(78, "Extracting text from image"); Ok((self.tesseract(file, languages)?, OcrSource::ImageOcr)) }
            "pdf" => self.extract_pdf(file, languages, progress),
            "docx" => { progress(78, "Extracting text from document"); Ok((self.extract_docx(file)?, OcrSource::EmbeddedDocumentText)) }
            _ => Err(StorageError::OcrUnsupportedFile),
        }
    }
    fn extract_pdf(&self, file: &Path, languages: &str, progress: impl Fn(u8, &'static str)) -> Result<(String, OcrSource), StorageError> {
        progress(74, "Detecting scanned PDF");
        let output = Command::new(self.program("pdftotext")?).args(["-enc", "UTF-8"]).arg(file).arg("-").output().map_err(|_| StorageError::OcrUnavailable)?;
        let embedded_text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if output.status.success() && embedded_text.chars().count() >= 32 { return Ok((embedded_text, OcrSource::EmbeddedPdfText)); }
        progress(80, "Rendering scanned pages"); fs::create_dir_all(&self.working_directory)?;
        let job_directory = self.working_directory.join(uuid::Uuid::new_v4().to_string()); fs::create_dir_all(&job_directory)?;
        let prefix = job_directory.join("page");
        let render = Command::new(self.program("pdftoppm")?).args(["-png", "-r", "200"]).arg(file).arg(&prefix).status().map_err(|_| StorageError::OcrUnavailable)?;
        if !render.success() { let _ = fs::remove_dir_all(&job_directory); return Err(StorageError::OcrFailed); }
        let mut pages = fs::read_dir(&job_directory)?.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.extension().and_then(|value| value.to_str()) == Some("png")).collect::<Vec<_>>(); pages.sort();
        let text = pages.iter().map(|page| self.tesseract(page, languages)).collect::<Result<Vec<_>, _>>()?.join("\n\n"); let _ = fs::remove_dir_all(job_directory);
        Ok((text, OcrSource::ScannedPdfOcr))
    }
    fn tesseract(&self, image: &Path, languages: &str) -> Result<String, StorageError> {
        let output = Command::new(self.program("tesseract")?).arg(image).arg("stdout").args(["-l", languages, "--psm", "3"]).output().map_err(|_| StorageError::OcrUnavailable)?;
        if !output.status.success() { return Err(StorageError::OcrFailed); } Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
    fn extract_docx(&self, file: &Path) -> Result<String, StorageError> {
        let mut archive = ZipArchive::new(std::fs::File::open(file)?).map_err(|_| StorageError::OcrUnsupportedFile)?;
        let mut document = archive.by_name("word/document.xml").map_err(|_| StorageError::OcrUnsupportedFile)?;
        let mut xml = String::new(); document.read_to_string(&mut xml)?;
        Ok(strip_xml(&xml))
    }
    fn program(&self, name: &str) -> Result<std::path::PathBuf, StorageError> {
        if !name.chars().all(|character| character.is_ascii_alphanumeric()) {
            return Err(StorageError::OcrUnavailable);
        }
        let suffix = if cfg!(target_os = "windows") { ".exe" } else { "" };
        let bundled = self.binaries_directory.join(format!("{name}{suffix}"));
        if bundled.is_file() {
            Ok(bundled)
        } else {
            Err(StorageError::OcrUnavailable)
        }
    }
}

fn validate_languages(languages: &str) -> Result<(), StorageError> { if languages.is_empty() || languages.len() > 128 || !languages.split('+').all(|id| !id.is_empty() && id.chars().all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')) { return Err(StorageError::InvalidOcrLanguage); } Ok(()) }
fn strip_xml(xml: &str) -> String { let mut output = String::new(); let mut in_tag = false; for character in xml.chars() { match character { '<' => { in_tag = true; output.push(' '); }, '>' => in_tag = false, _ if !in_tag => output.push(character), _ => {} } } output.split_whitespace().collect::<Vec<_>>().join(" ") }
use std::fs;
