// Hand-crafted file types for openai-types.
// These types mirror the original src/types/file.rs but are standalone (no crate:: imports).

use serde::{Deserialize, Serialize};

/// Helper to serialize a bare String value (used by the Other variant).
#[allow(clippy::ptr_arg)]
fn serialize_other_string<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serialize_other(value, serializer)
}

/// Helper function to serialize the `Other(String)` variant.
pub fn serialize_other<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(value)
}

/// The intended purpose of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum FilePurpose {
    #[serde(rename = "assistants")]
    Assistants,
    #[serde(rename = "assistants_output")]
    AssistantsOutput,
    #[serde(rename = "batch")]
    Batch,
    #[serde(rename = "batch_output")]
    BatchOutput,
    #[serde(rename = "fine-tune")]
    FineTune,
    #[serde(rename = "fine-tune-results")]
    FineTuneResults,
    #[serde(rename = "vision")]
    Vision,
    #[serde(rename = "user_data")]
    UserData,
    /// Catch-all for unknown purposes (forward compatibility).
    #[serde(serialize_with = "serialize_other_string")]
    Other(String),
}

impl<'de> Deserialize<'de> for FilePurpose {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "assistants" => Ok(FilePurpose::Assistants),
            "assistants_output" => Ok(FilePurpose::AssistantsOutput),
            "batch" => Ok(FilePurpose::Batch),
            "batch_output" => Ok(FilePurpose::BatchOutput),
            "fine-tune" => Ok(FilePurpose::FineTune),
            "fine-tune-results" => Ok(FilePurpose::FineTuneResults),
            "vision" => Ok(FilePurpose::Vision),
            "user_data" => Ok(FilePurpose::UserData),
            _ => Ok(FilePurpose::Other(s)),
        }
    }
}

/// Processing status of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum FileStatus {
    #[serde(rename = "uploaded")]
    Uploaded,
    #[serde(rename = "processed")]
    Processed,
    #[serde(rename = "error")]
    Error,
}

/// A file object from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub bytes: i64,
    pub created_at: i64,
    pub filename: String,
    pub object: String,
    pub purpose: FilePurpose,
    pub status: FileStatus,
    #[serde(default)]
    pub status_details: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}

/// Response from listing files.
#[derive(Debug, Clone, Deserialize)]
pub struct FileList {
    pub object: String,
    pub data: Vec<FileObject>,
    /// Whether there are more results available.
    #[serde(default)]
    pub has_more: Option<bool>,
    /// ID of the first object in the list.
    #[serde(default)]
    pub first_id: Option<String>,
    /// ID of the last object in the list.
    #[serde(default)]
    pub last_id: Option<String>,
}

/// Response from deleting a file.
#[derive(Debug, Clone, Deserialize)]
pub struct FileDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

/// Parameters for file upload (multipart).
#[derive(Debug)]
pub struct FileUploadParams {
    pub file: Vec<u8>,
    pub filename: String,
    pub purpose: FilePurpose,
}

impl FileUploadParams {
    pub fn new(file: Vec<u8>, filename: impl Into<String>, purpose: FilePurpose) -> Self {
        Self {
            file,
            filename: filename.into(),
            purpose,
        }
    }
}

/// Parameters for listing files with pagination.
#[derive(Debug, Clone, Default)]
pub struct FileListParams {
    /// Cursor for pagination -- fetch results after this file ID.
    pub after: Option<String>,
    /// Maximum number of results per page (1-10000).
    pub limit: Option<i64>,
    /// Sort order by `created_at`.
    pub order: Option<String>,
    /// Filter by file purpose.
    pub purpose: Option<FilePurpose>,
}

impl FileListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    pub fn purpose(mut self, purpose: FilePurpose) -> Self {
        self.purpose = Some(purpose);
        self
    }

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        if let Some(ref order) = self.order {
            q.push(("order".into(), order.clone()));
        }
        if let Some(ref purpose) = self.purpose {
            q.push((
                "purpose".into(),
                serde_json::to_value(purpose)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        q
    }
}
