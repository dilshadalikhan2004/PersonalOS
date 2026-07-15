use std::{path::Path, sync::Arc};

use arrow_array::{types::Float32Type, FixedSizeListArray, Float32Array, Int32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::{connect, query::{ExecutableQuery, QueryBase}, DistanceType, Table};
use serde::Serialize;
use uuid::Uuid;

use super::storage_error::StorageError;

const TABLE_NAME: &str = "document_chunks";

#[derive(Clone)]
pub struct VectorChunk {
    pub upload_id: String,
    pub chunk_index: i32,
    pub start: i32,
    pub end: i32,
    pub embedding: Vec<f32>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorSearchHit {
    pub upload_id: String,
    pub chunk_index: i32,
    pub start: i32,
    pub end: i32,
    pub distance: f32,
}

pub struct LanceVectorRepository {
    uri: String,
}

impl LanceVectorRepository {
    pub fn open(app_data: &Path) -> Result<Self, StorageError> {
        let directory = app_data.join("lancedb");
        std::fs::create_dir_all(&directory)?;
        Ok(Self { uri: directory.to_string_lossy().to_string() })
    }

    pub fn replace_upload_chunks(&self, upload_id: &str, chunks: &[VectorChunk]) -> Result<(), StorageError> {
        if chunks.is_empty() {
            return Ok(());
        }
        let upload_uuid = Uuid::parse_str(upload_id).map_err(|_| StorageError::InvalidStoragePath)?;
        tauri::async_runtime::block_on(async {
            let db = connect(&self.uri).execute().await.map_err(vector_error)?;
            let table_names = db.table_names().execute().await.map_err(vector_error)?;
            let batch = make_batch(chunks)?;
            if table_names.iter().any(|name| name == TABLE_NAME) {
                let table = db.open_table(TABLE_NAME).execute().await.map_err(vector_error)?;
                table.delete(&format!("upload_id = '{}'", upload_uuid)).await.map_err(vector_error)?;
                table.add(batch).execute().await.map_err(vector_error)?;
            } else {
                db.create_table(TABLE_NAME, batch).execute().await.map_err(vector_error)?;
            }
            Ok(())
        })
    }

    pub fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<VectorSearchHit>, StorageError> {
        let limit = limit.clamp(1, 25);
        tauri::async_runtime::block_on(async {
            let db = connect(&self.uri).execute().await.map_err(vector_error)?;
            if !db.table_names().execute().await.map_err(vector_error)?.iter().any(|name| name == TABLE_NAME) {
                return Ok(Vec::new());
            }
            let table = db.open_table(TABLE_NAME).execute().await.map_err(vector_error)?;
            collect_hits(table, embedding, limit).await
        })
    }
}

async fn collect_hits(table: Table, embedding: &[f32], limit: usize) -> Result<Vec<VectorSearchHit>, StorageError> {
    let batches = table
        .query()
        .nearest_to(embedding)
        .map_err(vector_error)?
        .distance_type(DistanceType::Cosine)
        .limit(limit)
        .execute()
        .await
        .map_err(vector_error)?
        .try_collect::<Vec<_>>()
        .await
        .map_err(vector_error)?;
    let mut hits = Vec::new();
    for batch in batches {
        let upload_ids = batch.column_by_name("upload_id").and_then(|column| column.as_any().downcast_ref::<StringArray>()).ok_or(StorageError::InvalidModelResponse)?;
        let chunk_indexes = batch.column_by_name("chunk_index").and_then(|column| column.as_any().downcast_ref::<Int32Array>()).ok_or(StorageError::InvalidModelResponse)?;
        let starts = batch.column_by_name("start").and_then(|column| column.as_any().downcast_ref::<Int32Array>()).ok_or(StorageError::InvalidModelResponse)?;
        let ends = batch.column_by_name("end").and_then(|column| column.as_any().downcast_ref::<Int32Array>()).ok_or(StorageError::InvalidModelResponse)?;
        let distances = batch.column_by_name("_distance").and_then(|column| column.as_any().downcast_ref::<Float32Array>()).ok_or(StorageError::InvalidModelResponse)?;
        for row in 0..batch.num_rows() {
            hits.push(VectorSearchHit {
                upload_id: upload_ids.value(row).to_owned(),
                chunk_index: chunk_indexes.value(row),
                start: starts.value(row),
                end: ends.value(row),
                distance: distances.value(row),
            });
        }
    }
    Ok(hits)
}

fn make_batch(chunks: &[VectorChunk]) -> Result<RecordBatch, StorageError> {
    let dimension = chunks.first().map(|chunk| chunk.embedding.len()).ok_or(StorageError::InvalidModelResponse)?;
    if dimension == 0 || chunks.iter().any(|chunk| chunk.embedding.len() != dimension) {
        return Err(StorageError::InvalidModelResponse);
    }
    let schema = Arc::new(Schema::new(vec![
        Field::new("upload_id", DataType::Utf8, false),
        Field::new("chunk_index", DataType::Int32, false),
        Field::new("start", DataType::Int32, false),
        Field::new("end", DataType::Int32, false),
        Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dimension as i32), true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from_iter_values(chunks.iter().map(|chunk| chunk.upload_id.as_str()))),
            Arc::new(Int32Array::from_iter_values(chunks.iter().map(|chunk| chunk.chunk_index))),
            Arc::new(Int32Array::from_iter_values(chunks.iter().map(|chunk| chunk.start))),
            Arc::new(Int32Array::from_iter_values(chunks.iter().map(|chunk| chunk.end))),
            Arc::new(FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                chunks.iter().map(|chunk| Some(chunk.embedding.iter().map(|value| Some(*value)).collect::<Vec<_>>())),
                dimension as i32,
            )),
        ],
    )
    .map_err(|error| StorageError::Vector(error.to_string()))
}

fn vector_error(error: impl std::fmt::Display) -> StorageError {
    StorageError::Vector(error.to_string())
}
