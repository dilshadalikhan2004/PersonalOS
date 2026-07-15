use serde::Serialize;

use crate::infrastructure::{
    metadata_repository::StructuredMetadata,
    storage_error::StorageError,
    upload_repository::UploadedFile,
};

use super::{metadata_service::MetadataService, upload_service::UploadService};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifeTimelineEvent {
    pub id: String,
    pub title: String,
    pub category: String,
    pub date: String,
    pub source: String,
    pub confidence: f32,
}

pub struct TimelineService;

impl TimelineService {
    pub fn generate(
        uploads: &UploadService,
        metadata: &MetadataService,
    ) -> Result<Vec<LifeTimelineEvent>, StorageError> {
        let uploads = uploads.recent(250)?;
        let metadata = metadata.list()?;
        let mut events = metadata
            .iter()
            .filter_map(event_from_metadata)
            .chain(uploads.iter().filter_map(event_from_upload))
            .collect::<Vec<_>>();
        events.sort_by(|left, right| left.date.cmp(&right.date).then(left.title.cmp(&right.title)));
        events.dedup_by(|left, right| left.title == right.title && left.date == right.date);
        Ok(events)
    }
}

fn event_from_metadata(metadata: &StructuredMetadata) -> Option<LifeTimelineEvent> {
    let date = metadata
        .document_date
        .clone()
        .or_else(|| metadata.expiry_date.clone())?;
    let document_type = metadata.document_type.to_ascii_lowercase();
    let title_text = metadata.title.clone().unwrap_or_else(|| metadata.document_type.clone());
    let combined = format!("{title_text} {document_type}").to_ascii_lowercase();
    let (title, category) = if combined.contains("passport") {
        ("Passport renewed".to_owned(), "Identity".to_owned())
    } else if combined.contains("car") || combined.contains("vehicle") {
        ("Car purchased".to_owned(), "Asset".to_owned())
    } else if combined.contains("medical") || combined.contains("health") || combined.contains("checkup") {
        ("Medical checkup".to_owned(), "Health".to_owned())
    } else if combined.contains("insurance") || combined.contains("policy") {
        ("Insurance renewed".to_owned(), "Protection".to_owned())
    } else if combined.contains("laptop") || combined.contains("computer") {
        ("Laptop purchased".to_owned(), "Asset".to_owned())
    } else {
        (title_text, metadata.document_type.clone())
    };
    Some(LifeTimelineEvent {
        id: metadata.upload_id.clone(),
        title,
        category,
        date,
        source: metadata.title.clone().unwrap_or_else(|| "Structured metadata".to_owned()),
        confidence: 0.88,
    })
}

fn event_from_upload(upload: &UploadedFile) -> Option<LifeTimelineEvent> {
    let date = unix_ms_to_date(upload.created_at_unix_ms);
    Some(LifeTimelineEvent {
        id: upload.id.clone(),
        title: format!("{} uploaded", upload.file_name),
        category: "Upload".to_owned(),
        date,
        source: upload.media_type.clone(),
        confidence: 0.62,
    })
}

fn unix_ms_to_date(value: i64) -> String {
    let days = value.div_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe.div_euclid(1_460) + doe.div_euclid(36_524) - doe.div_euclid(146_096)).div_euclid(365);
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe.div_euclid(4) - yoe.div_euclid(100));
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year, month, day)
}
