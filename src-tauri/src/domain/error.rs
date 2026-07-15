use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("document identifier is invalid")]
    InvalidDocumentId,
    #[error("document title cannot be empty")]
    EmptyTitle,
    #[error("document title exceeds the allowed length")]
    TitleTooLong,
    #[error("document content exceeds the allowed size")]
    ContentTooLarge,
}
