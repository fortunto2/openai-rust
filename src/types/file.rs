// File types — mirrors openai-python types/file_object.py

use serde::{Deserialize, Serialize};

/// The intended purpose of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

/// Processing status of an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FileStatus {
    Uploaded,
    Processed,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_file_object() {
        let json = r#"{
            "id": "file-abc123",
            "object": "file",
            "bytes": 120000,
            "created_at": 1677610602,
            "filename": "data.jsonl",
            "purpose": "fine-tune",
            "status": "processed"
        }"#;
        let file: FileObject = serde_json::from_str(json).unwrap();
        assert_eq!(file.id, "file-abc123");
        assert_eq!(file.bytes, 120000);
        assert_eq!(file.purpose, FilePurpose::FineTune);
        assert_eq!(file.status, FileStatus::Processed);
    }

    #[test]
    fn test_deserialize_file_deleted() {
        let json = r#"{"id": "file-abc123", "object": "file", "deleted": true}"#;
        let deleted: FileDeleted = serde_json::from_str(json).unwrap();
        assert!(deleted.deleted);
    }
}
