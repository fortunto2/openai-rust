// Manual: hand-crafted upload types (enums, builders, response structs).

use serde::{Deserialize, Serialize};

/// Request body for `POST /uploads`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct UploadCreateRequest {
    /// File size in bytes.
    pub bytes: i64,

    /// Filename.
    pub filename: String,

    /// MIME type.
    pub mime_type: String,

    /// Purpose (e.g. "assistants", "batch", "fine-tune").
    pub purpose: String,
}

impl UploadCreateRequest {
    pub fn new(
        bytes: i64,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        purpose: impl Into<String>,
    ) -> Self {
        Self {
            bytes,
            filename: filename.into(),
            mime_type: mime_type.into(),
            purpose: purpose.into(),
        }
    }
}

/// Request body for `POST /uploads/{upload_id}/complete`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct UploadCompleteRequest {
    pub part_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}

/// Status of an upload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum UploadStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "expired")]
    Expired,
}

/// An upload object.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Upload {
    pub id: String,
    pub object: String,
    pub bytes: i64,
    pub filename: String,
    pub purpose: String,
    pub status: UploadStatus,
    pub created_at: i64,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub file: Option<serde_json::Value>,
}
