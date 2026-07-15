use std::collections::BTreeMap;

use serde::Serialize;

use crate::infrastructure::{
    metadata_repository::StructuredMetadata,
    storage_error::StorageError,
    upload_repository::UploadedFile,
};

use super::{metadata_service::MetadataService, upload_service::UploadService};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub upcoming_expiry: Vec<DashboardItem>,
    pub bills: Vec<DashboardItem>,
    pub insurance: Vec<DashboardItem>,
    pub timeline: Vec<DashboardItem>,
    pub recent_uploads: Vec<UploadedFile>,
    pub categories: Vec<CategoryCount>,
    pub storage_usage_bytes: u64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardItem {
    pub upload_id: String,
    pub title: String,
    pub subtitle: String,
    pub date: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    pub category: String,
    pub count: usize,
}

pub struct DashboardService;

impl DashboardService {
    pub fn summary(uploads: &UploadService, metadata: &MetadataService) -> Result<DashboardSummary, StorageError> {
        let recent_uploads = uploads.recent(8)?;
        let all_metadata = metadata.list()?;
        let storage_usage_bytes = recent_uploads.iter().map(|upload| upload.size_bytes).sum();
        Ok(DashboardSummary {
            upcoming_expiry: by_expiry(&all_metadata),
            bills: by_type(&all_metadata, &["bill", "invoice", "utility", "electricity"]),
            insurance: by_type(&all_metadata, &["insurance", "policy"]),
            timeline: timeline(&all_metadata),
            recent_uploads,
            categories: categories(&all_metadata),
            storage_usage_bytes,
        })
    }
}

fn by_expiry(metadata: &[StructuredMetadata]) -> Vec<DashboardItem> {
    let mut items = metadata.iter().filter(|item| item.expiry_date.is_some()).map(item_from_metadata).collect::<Vec<_>>();
    items.sort_by(|left, right| left.date.cmp(&right.date));
    items.truncate(6);
    items
}

fn by_type(metadata: &[StructuredMetadata], needles: &[&str]) -> Vec<DashboardItem> {
    metadata
        .iter()
        .filter(|item| {
            let haystack = format!("{} {}", item.document_type, item.title.clone().unwrap_or_default()).to_ascii_lowercase();
            needles.iter().any(|needle| haystack.contains(needle))
        })
        .take(6)
        .map(item_from_metadata)
        .collect()
}

fn timeline(metadata: &[StructuredMetadata]) -> Vec<DashboardItem> {
    let mut items = metadata.iter().map(item_from_metadata).collect::<Vec<_>>();
    items.sort_by(|left, right| right.date.cmp(&left.date));
    items.truncate(8);
    items
}

fn categories(metadata: &[StructuredMetadata]) -> Vec<CategoryCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for item in metadata {
        *counts.entry(item.document_type.clone()).or_insert(0) += 1;
    }
    counts.into_iter().map(|(category, count)| CategoryCount { category, count }).collect()
}

fn item_from_metadata(metadata: &StructuredMetadata) -> DashboardItem {
    DashboardItem {
        upload_id: metadata.upload_id.clone(),
        title: metadata.title.clone().unwrap_or_else(|| "Untitled document".to_owned()),
        subtitle: metadata.document_type.clone(),
        date: metadata.expiry_date.clone().or_else(|| metadata.document_date.clone()),
    }
}
