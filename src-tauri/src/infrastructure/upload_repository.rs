use std::{fs, io::Cursor, path::{Component, Path, PathBuf}, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zip::ZipArchive;

use super::{crypto, keychain::MasterKey, storage_error::StorageError};

const MAX_UPLOAD_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadProgress { pub percent: u8, pub stage: &'static str }

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadedFile { pub id: String, pub file_name: String, pub media_type: String, pub size_bytes: u64, pub created_at_unix_ms: i64 }

#[derive(Serialize, Deserialize)]
struct EncryptedUploadMetadata { file_name: String, media_type: String, size_bytes: u64, created_at_unix_ms: i64 }

#[derive(Copy, Clone)]
enum SupportedType { Pdf, Png, Jpeg, Docx }

impl SupportedType {
    fn media_type(self) -> &'static str { match self { Self::Pdf => "application/pdf", Self::Png => "image/png", Self::Jpeg => "image/jpeg", Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document" } }
}

pub struct UploadRepository { connection: Connection, originals_directory: PathBuf, thumbnails_directory: PathBuf, master_key: Arc<MasterKey> }

impl UploadRepository {
    pub fn open(app_data: &Path, master_key: Arc<MasterKey>) -> Result<Self, StorageError> {
        let originals_directory = app_data.join("uploads"); let thumbnails_directory = app_data.join("thumbnails");
        fs::create_dir_all(&originals_directory)?; fs::create_dir_all(&thumbnails_directory)?;
        let connection = Connection::open(app_data.join("lifeos.sqlite3"))?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA secure_delete = ON; CREATE TABLE IF NOT EXISTS uploads (id TEXT PRIMARY KEY NOT NULL, encrypted_metadata BLOB NOT NULL, original_file_name TEXT NOT NULL UNIQUE, thumbnail_file_name TEXT NOT NULL UNIQUE);")?;
        Ok(Self { connection, originals_directory, thumbnails_directory, master_key })
    }

    pub fn ingest(&mut self, source_path: &Path, mut progress: impl FnMut(UploadProgress)) -> Result<UploadedFile, StorageError> {
        progress(UploadProgress { percent: 8, stage: "Validating file" });
        let link_metadata = fs::symlink_metadata(source_path)?;
        if link_metadata.file_type().is_symlink() {
            return Err(StorageError::UnsupportedUpload);
        }
        let source_metadata = fs::metadata(source_path)?;
        if !source_metadata.is_file() || source_metadata.len() > MAX_UPLOAD_BYTES { return Err(StorageError::UnsupportedUpload); }
        let source_bytes = fs::read(source_path)?;
        let file_type = validate_file(source_path, &source_bytes)?;
        let file_name = source_path.file_name().and_then(|name| name.to_str()).filter(|name| !name.is_empty()).ok_or(StorageError::UnsupportedUpload)?.to_owned();
        let now = now_ms()?; let id = Uuid::new_v4().to_string();
        progress(UploadProgress { percent: 32, stage: "Encrypting original" });
        let original_name = self.write_encrypted(&self.originals_directory, &source_bytes)?;
        progress(UploadProgress { percent: 65, stage: "Generating thumbnail" });
        let thumbnail = generate_thumbnail(file_type, &source_bytes)?;
        let thumbnail_name = self.write_encrypted(&self.thumbnails_directory, &thumbnail)?;
        progress(UploadProgress { percent: 86, stage: "Securing metadata" });
        let metadata = EncryptedUploadMetadata { file_name: file_name.clone(), media_type: file_type.media_type().to_owned(), size_bytes: source_metadata.len(), created_at_unix_ms: now };
        let encrypted_metadata = crypto::encrypt(&self.master_key, &serde_json::to_vec(&metadata)?)?;
        if let Err(error) = self.connection.execute("INSERT INTO uploads (id, encrypted_metadata, original_file_name, thumbnail_file_name) VALUES (?1, ?2, ?3, ?4)", params![id, encrypted_metadata, &original_name, &thumbnail_name]) {
            let _ = fs::remove_file(self.safe_path(&self.originals_directory, &original_name)?); let _ = fs::remove_file(self.safe_path(&self.thumbnails_directory, &thumbnail_name)?); return Err(error.into());
        }
        progress(UploadProgress { percent: 90, stage: "Stored securely" });
        Ok(UploadedFile { id, file_name, media_type: file_type.media_type().to_owned(), size_bytes: source_metadata.len(), created_at_unix_ms: now })
    }

    pub fn list_recent(&self, limit: usize) -> Result<Vec<UploadedFile>, StorageError> {
        let limit = limit.min(500);
        let mut statement = self.connection.prepare("SELECT id, encrypted_metadata FROM uploads ORDER BY rowid DESC LIMIT ?1")?;
        let rows = statement.query_map(params![limit as i64], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?;
        let mut uploads = Vec::new();
        for row in rows {
            let (id, payload) = row?;
            let metadata: EncryptedUploadMetadata = serde_json::from_slice(&crypto::decrypt(&self.master_key, &payload)?)?;
            uploads.push(UploadedFile { id, file_name: metadata.file_name, media_type: metadata.media_type, size_bytes: metadata.size_bytes, created_at_unix_ms: metadata.created_at_unix_ms });
        }
        Ok(uploads)
    }

    fn write_encrypted(&self, directory: &Path, bytes: &[u8]) -> Result<String, StorageError> {
        let file_name = format!("{}.lifeos", Uuid::new_v4()); let destination = self.safe_path(directory, &file_name)?; let temporary = destination.with_extension("tmp");
        fs::write(&temporary, crypto::encrypt(&self.master_key, bytes)?)?; fs::rename(temporary, destination)?; Ok(file_name)
    }
    fn safe_path(&self, directory: &Path, file_name: &str) -> Result<PathBuf, StorageError> {
        let path = Path::new(file_name); if path.extension().and_then(|value| value.to_str()) != Some("lifeos") || path.components().count() != 1 || !matches!(path.components().next(), Some(Component::Normal(_))) { return Err(StorageError::InvalidStoragePath); } Ok(directory.join(path))
    }
}

fn validate_file(path: &Path, bytes: &[u8]) -> Result<SupportedType, StorageError> {
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    match extension.as_str() {
        "pdf" if bytes.starts_with(b"%PDF-") => Ok(SupportedType::Pdf),
        "png" if bytes.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]) => Ok(SupportedType::Png),
        "jpg" | "jpeg" if bytes.starts_with(&[255, 216, 255]) => Ok(SupportedType::Jpeg),
        "docx" => { let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|_| StorageError::UnsupportedUpload)?; if archive.by_name("[Content_Types].xml").is_ok() && archive.by_name("word/document.xml").is_ok() { Ok(SupportedType::Docx) } else { Err(StorageError::UnsupportedUpload) } }
        _ => Err(StorageError::UnsupportedUpload),
    }
}

fn generate_thumbnail(file_type: SupportedType, bytes: &[u8]) -> Result<Vec<u8>, StorageError> {
    let image = match file_type { SupportedType::Png | SupportedType::Jpeg => image::load_from_memory(bytes).map_err(|_| StorageError::UnsupportedUpload)?.thumbnail(640, 400), _ => placeholder(file_type) };
    let mut output = Cursor::new(Vec::new()); image.write_to(&mut output, ImageFormat::Png).map_err(|_| StorageError::ThumbnailGenerationFailed)?; Ok(output.into_inner())
}

fn placeholder(file_type: SupportedType) -> DynamicImage {
    let color = match file_type { SupportedType::Pdf => [220, 81, 88, 255], SupportedType::Docx => [73, 132, 219, 255], _ => [104, 92, 175, 255] };
    let mut image = RgbaImage::from_pixel(640, 400, Rgba([24, 25, 29, 255]));
    for y in 54..346 { for x in 188..452 { image.put_pixel(x, y, Rgba(color)); } }
    for y in 105..120 { for x in 224..416 { image.put_pixel(x, y, Rgba([255, 255, 255, 180])); } }
    for y in 145..155 { for x in 224..380 { image.put_pixel(x, y, Rgba([255, 255, 255, 110])); } }
    DynamicImage::ImageRgba8(image)
}

fn now_ms() -> Result<i64, StorageError> { SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| StorageError::Io(std::io::Error::other(error))).map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX)) }
