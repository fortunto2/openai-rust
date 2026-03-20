// Upload types — mirrors openai-python types/upload.py

use serde::{Deserialize, Serialize};

/// Request body for `POST /uploads`.
#[derive(Debug, Clone, Serialize)]
pub struct UploadCreateRequest {
    /// File size in bytes.
    pub bytes: i64,

    /// Filename.
    pub filename: String,

    /// MIME type.
    pub mime_type: String,

    /// Purpose (e.g. assistants, batch, fine-tune).
    pub purpose: crate::types::file::FilePurpose,
}

impl UploadCreateRequest {
    pub fn new(
        bytes: i64,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        purpose: crate::types::file::FilePurpose,
    ) -> Self {
        Self {
            bytes,
            filename: filename.into(),
            mime_type: mime_type.into(),
            purpose,
        }
    }
}

/// Request body for `POST /uploads/{upload_id}/complete`.
#[derive(Debug, Clone, Serialize)]
pub struct UploadCompleteRequest {
    pub part_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,
}

/// Status of an upload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum UploadStatus {
    Pending,
    Completed,
    Cancelled,
    Expired,
}

/// An upload object.
#[derive(Debug, Clone, Deserialize)]
pub struct Upload {
    pub id: String,
    pub object: String,
    pub bytes: i64,
    pub filename: String,
    pub purpose: crate::types::file::FilePurpose,
    pub status: UploadStatus,
    pub created_at: i64,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub file: Option<crate::types::file::FileObject>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_upload_create() {
        let req = UploadCreateRequest::new(
            2_000_000,
            "data.jsonl",
            "text/jsonl",
            crate::types::file::FilePurpose::FineTune,
        );
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["bytes"], 2_000_000);
        assert_eq!(json["filename"], "data.jsonl");
    }

    #[test]
    fn test_deserialize_upload() {
        let json = r#"{
            "id": "upload_abc123",
            "object": "upload",
            "bytes": 2000000,
            "filename": "data.jsonl",
            "purpose": "fine-tune",
            "status": "pending",
            "created_at": 1699012949,
            "expires_at": 1699016549
        }"#;
        let upload: Upload = serde_json::from_str(json).unwrap();
        assert_eq!(upload.id, "upload_abc123");
        assert_eq!(upload.status, UploadStatus::Pending);
    }
}
