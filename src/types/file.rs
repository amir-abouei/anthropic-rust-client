use serde::Deserialize;

/// Scope that constrains file visibility (e.g., tied to a session).
#[derive(Debug, Clone, Deserialize)]
pub struct FileScope {
    pub id: String,
    #[serde(rename = "type")]
    pub scope_type: String,
}

/// Metadata returned when uploading or retrieving a file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub created_at: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    #[serde(rename = "type")]
    pub object_type: String,
    pub downloadable: Option<bool>,
    pub scope: Option<FileScope>,
}

/// Response from `GET /v1/files`.
#[derive(Debug, Clone, Deserialize)]
pub struct FileList {
    pub data: Vec<FileMetadata>,
    pub has_more: bool,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
}

/// Response from `DELETE /v1/files/{id}`.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletedFile {
    pub id: String,
    pub deleted: bool,
}

/// Query parameters for listing files.
#[derive(Debug, Clone, Default)]
pub struct ListFilesParams {
    pub after_id: Option<String>,
    pub before_id: Option<String>,
    pub limit: Option<u32>,
}

impl ListFilesParams {
    pub fn new() -> Self { Self::default() }

    pub fn after_id(mut self, id: impl Into<String>) -> Self {
        self.after_id = Some(id.into());
        self
    }

    pub fn before_id(mut self, id: impl Into<String>) -> Self {
        self.before_id = Some(id.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}
