use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::infrastructure::{metadata_repository::StructuredMetadata, storage_error::StorageError};

use super::metadata_service::MetadataService;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalReminder {
    pub id: String,
    pub title: String,
    pub reminder_type: ReminderType,
    pub due_date: String,
    pub days_until: i64,
    pub severity: ReminderSeverity,
    pub source: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReminderType {
    Expiry,
    Bill,
    Insurance,
    Subscription,
    VehicleService,
    Passport,
    DrivingLicence,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReminderSeverity {
    Overdue,
    Critical,
    Soon,
    Later,
}

pub struct ReminderService;

impl ReminderService {
    pub fn detect(metadata: &MetadataService) -> Result<Vec<LocalReminder>, StorageError> {
        let today = current_epoch_days()?;
        let mut reminders = metadata
            .list()?
            .iter()
            .flat_map(|item| reminders_from_metadata(item, today))
            .collect::<Vec<_>>();
        reminders.sort_by(|left, right| left.days_until.cmp(&right.days_until).then(left.title.cmp(&right.title)));
        reminders.dedup_by(|left, right| left.id == right.id && left.reminder_type_name() == right.reminder_type_name());
        Ok(reminders)
    }
}

impl LocalReminder {
    fn reminder_type_name(&self) -> &'static str {
        match self.reminder_type {
            ReminderType::Expiry => "expiry",
            ReminderType::Bill => "bill",
            ReminderType::Insurance => "insurance",
            ReminderType::Subscription => "subscription",
            ReminderType::VehicleService => "vehicle-service",
            ReminderType::Passport => "passport",
            ReminderType::DrivingLicence => "driving-licence",
        }
    }
}

fn reminders_from_metadata(metadata: &StructuredMetadata, today: i64) -> Vec<LocalReminder> {
    let mut reminders = Vec::new();
    let title = metadata.title.clone().unwrap_or_else(|| metadata.document_type.clone());
    let text = format!("{} {}", title, metadata.document_type).to_ascii_lowercase();
    let due_date = metadata.expiry_date.clone().or_else(|| metadata.document_date.clone());
    if let Some(date) = due_date {
        if let Some(due_days) = parse_date_to_epoch_days(&date) {
            let days_until = due_days - today;
            if days_until <= 90 {
                if metadata.expiry_date.is_some() {
                    reminders.push(make_reminder(metadata, &title, ReminderType::Expiry, &date, days_until));
                }
                if text.contains("bill") || text.contains("invoice") || text.contains("electricity") || text.contains("utility") {
                    reminders.push(make_reminder(metadata, &title, ReminderType::Bill, &date, days_until));
                }
                if text.contains("insurance") || text.contains("policy") {
                    reminders.push(make_reminder(metadata, &title, ReminderType::Insurance, &date, days_until));
                }
                if text.contains("subscription") || text.contains("membership") {
                    reminders.push(make_reminder(metadata, &title, ReminderType::Subscription, &date, days_until));
                }
                if text.contains("vehicle") || text.contains("car") || text.contains("service") || text.contains("maintenance") {
                    reminders.push(make_reminder(metadata, &title, ReminderType::VehicleService, &date, days_until));
                }
                if text.contains("passport") {
                    reminders.push(make_reminder(metadata, &title, ReminderType::Passport, &date, days_until));
                }
                if text.contains("driving") || text.contains("licence") || text.contains("license") {
                    reminders.push(make_reminder(metadata, &title, ReminderType::DrivingLicence, &date, days_until));
                }
            }
        }
    }
    reminders
}

fn make_reminder(
    metadata: &StructuredMetadata,
    title: &str,
    reminder_type: ReminderType,
    due_date: &str,
    days_until: i64,
) -> LocalReminder {
    LocalReminder {
        id: metadata.upload_id.clone(),
        title: reminder_title(title, &reminder_type),
        reminder_type,
        due_date: due_date.to_owned(),
        days_until,
        severity: severity(days_until),
        source: title.to_owned(),
    }
}

fn reminder_title(title: &str, reminder_type: &ReminderType) -> String {
    match reminder_type {
        ReminderType::Expiry => format!("{title} expires"),
        ReminderType::Bill => format!("{title} bill due"),
        ReminderType::Insurance => format!("{title} insurance reminder"),
        ReminderType::Subscription => format!("{title} subscription renewal"),
        ReminderType::VehicleService => format!("{title} vehicle service"),
        ReminderType::Passport => "Passport renewal reminder".to_owned(),
        ReminderType::DrivingLicence => "Driving licence renewal reminder".to_owned(),
    }
}

fn severity(days_until: i64) -> ReminderSeverity {
    if days_until < 0 {
        ReminderSeverity::Overdue
    } else if days_until <= 7 {
        ReminderSeverity::Critical
    } else if days_until <= 30 {
        ReminderSeverity::Soon
    } else {
        ReminderSeverity::Later
    }
}

fn current_epoch_days() -> Result<i64, StorageError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| StorageError::Io(std::io::Error::other(error)))
        .map(|duration| (duration.as_secs() / 86_400) as i64)
}

fn parse_date_to_epoch_days(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.len() >= 10 && trimmed.as_bytes().get(4) == Some(&b'-') && trimmed.as_bytes().get(7) == Some(&b'-') {
        let year = trimmed.get(0..4)?.parse().ok()?;
        let month = trimmed.get(5..7)?.parse().ok()?;
        let day = trimmed.get(8..10)?.parse().ok()?;
        return Some(days_from_civil(year, month, day));
    }
    let parts = trimmed.split(&['/', '-'][..]).collect::<Vec<_>>();
    if parts.len() >= 3 && parts[2].len() == 4 {
        let day = parts[0].parse().ok()?;
        let month = parts[1].parse().ok()?;
        let year = parts[2].parse().ok()?;
        return Some(days_from_civil(year, month, day));
    }
    None
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 }.div_euclid(400);
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2).div_euclid(5) + day - 1;
    let doe = yoe * 365 + yoe.div_euclid(4) - yoe.div_euclid(100) + doy;
    era * 146_097 + doe - 719_468
}
