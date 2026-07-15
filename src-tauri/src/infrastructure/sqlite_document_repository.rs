use std::{fs, path::{Component, Path, PathBuf}, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::domain::{Document, DocumentId, DocumentPayload, NewDocument, UpdateDocument};

use super::{crypto, keychain::MasterKey, storage_error::StorageError};

/// SQLite provides the transactional index; document text is stored only in AES-256-GCM files.
pub struct SqliteDocumentRepository {
    connection: Connection,
    encrypted_files_directory: PathBuf,
    master_key: Arc<MasterKey>,
}

struct DocumentRecord {
    id: DocumentId,
    file_name: String,
}

impl SqliteDocumentRepository {
    pub fn open(application_data_directory: &Path, master_key: Arc<MasterKey>) -> Result<Self, StorageError> {
        fs::create_dir_all(application_data_directory)?;
        let encrypted_files_directory = application_data_directory.join("documents");
        fs::create_dir_all(&encrypted_files_directory)?;

        let connection = Connection::open(application_data_directory.join("lifeos.sqlite3"))?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA secure_delete = ON;
             CREATE TABLE IF NOT EXISTS documents (
                 id TEXT PRIMARY KEY NOT NULL,
                 file_name TEXT NOT NULL UNIQUE
             );",
        )?;

        Ok(Self { connection, encrypted_files_directory, master_key })
    }

    pub fn create(&mut self, new_document: NewDocument) -> Result<Document, StorageError> {
        let id = DocumentId::new();
        let now = unix_time_ms()?;
        let payload = DocumentPayload { title: new_document.title, content: new_document.content, created_at_unix_ms: now, updated_at_unix_ms: now };
        let file_name = self.write_payload(&payload)?;
        let insert_result = self.connection.execute(
            "INSERT INTO documents (id, file_name) VALUES (?1, ?2)",
            params![id.as_str(), &file_name],
        );
        if let Err(error) = insert_result {
            let _ = fs::remove_file(self.document_path(&file_name)?);
            return Err(error.into());
        }
        Ok(Document { id, title: payload.title, content: payload.content, created_at_unix_ms: payload.created_at_unix_ms, updated_at_unix_ms: payload.updated_at_unix_ms })
    }

    pub fn get(&self, id: &DocumentId) -> Result<Document, StorageError> {
        let record = self.find_record(id)?.ok_or(StorageError::NotFound)?;
        self.read_record(record)
    }

    pub fn list(&self) -> Result<Vec<Document>, StorageError> {
        let mut statement = self.connection.prepare("SELECT id, file_name FROM documents")?;
        let records = statement.query_map([], |row| {
            let id: String = row.get(0)?;
            let id = DocumentId::parse(&id).map_err(|_| rusqlite::Error::InvalidQuery)?;
            Ok(DocumentRecord { id, file_name: row.get(1)? })
        })?.collect::<Result<Vec<_>, _>>()?;
        let mut documents = records.into_iter().map(|record| self.read_record(record)).collect::<Result<Vec<_>, _>>()?;
        documents.sort_by(|left, right| right.updated_at_unix_ms.cmp(&left.updated_at_unix_ms));
        Ok(documents)
    }

    pub fn update(&mut self, id: &DocumentId, update: UpdateDocument) -> Result<Document, StorageError> {
        let existing = self.find_record(id)?.ok_or(StorageError::NotFound)?;
        let previous_document = self.read_record(DocumentRecord { id: existing.id.clone(), file_name: existing.file_name.clone() })?;
        let now = unix_time_ms()?;
        let payload = DocumentPayload { title: update.title, content: update.content, created_at_unix_ms: previous_document.created_at_unix_ms, updated_at_unix_ms: now };
        let new_file_name = self.write_payload(&payload)?;
        let updated = self.connection.execute(
            "UPDATE documents SET file_name = ?1 WHERE id = ?2",
            params![&new_file_name, id.as_str()],
        )?;
        if updated != 1 {
            let _ = fs::remove_file(self.document_path(&new_file_name)?);
            return Err(StorageError::NotFound);
        }
        // Old encrypted blobs are safe to remove only after SQLite points to the replacement.
        let _ = fs::remove_file(self.document_path(&existing.file_name)?);
        Ok(Document { id: id.clone(), title: payload.title, content: payload.content, created_at_unix_ms: payload.created_at_unix_ms, updated_at_unix_ms: payload.updated_at_unix_ms })
    }

    pub fn delete(&mut self, id: &DocumentId) -> Result<(), StorageError> {
        let record = self.find_record(id)?.ok_or(StorageError::NotFound)?;
        self.connection.execute("DELETE FROM documents WHERE id = ?1", params![id.as_str()])?;
        // A failed cleanup cannot restore the document through the app; leave encrypted remnants for later cleanup.
        let _ = fs::remove_file(self.document_path(&record.file_name)?);
        Ok(())
    }

    fn find_record(&self, id: &DocumentId) -> Result<Option<DocumentRecord>, StorageError> {
        self.connection.query_row(
            "SELECT id, file_name FROM documents WHERE id = ?1",
            params![id.as_str()],
            |row| Ok(DocumentRecord { id: DocumentId::parse(&row.get::<_, String>(0)?).map_err(|_| rusqlite::Error::InvalidQuery)?, file_name: row.get(1)? }),
        ).optional().map_err(Into::into)
    }

    fn read_record(&self, record: DocumentRecord) -> Result<Document, StorageError> {
        let encrypted_payload = fs::read(self.document_path(&record.file_name)?)?;
        let plaintext = crypto::decrypt(&self.master_key, &encrypted_payload)?;
        let payload: DocumentPayload = serde_json::from_slice(&plaintext)?;
        Ok(Document { id: record.id, title: payload.title, content: payload.content, created_at_unix_ms: payload.created_at_unix_ms, updated_at_unix_ms: payload.updated_at_unix_ms })
    }

    fn write_payload(&self, payload: &DocumentPayload) -> Result<String, StorageError> {
        let plaintext = serde_json::to_vec(payload)?;
        let encrypted = crypto::encrypt(&self.master_key, &plaintext)?;
        let file_name = format!("{}.lifeos", Uuid::new_v4());
        let destination = self.document_path(&file_name)?;
        let temporary = destination.with_extension("tmp");
        fs::write(&temporary, encrypted)?;
        fs::rename(temporary, destination)?;
        Ok(file_name)
    }

    fn document_path(&self, file_name: &str) -> Result<PathBuf, StorageError> {
        let candidate = Path::new(file_name);
        if candidate.extension().and_then(|extension| extension.to_str()) != Some("lifeos") || candidate.components().count() != 1 || !matches!(candidate.components().next(), Some(Component::Normal(_))) {
            return Err(StorageError::InvalidStoragePath);
        }
        Ok(self.encrypted_files_directory.join(candidate))
    }
}

fn unix_time_ms() -> Result<i64, StorageError> {
    SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| StorageError::Io(std::io::Error::other(error))).map(|duration| duration.as_millis().try_into().unwrap_or(i64::MAX))
}
