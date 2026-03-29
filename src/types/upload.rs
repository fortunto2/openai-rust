// Upload types — re-exported from openai-types.
pub use openai_types::uploads::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_upload_create() {
        let req = UploadCreateRequest::new(2_000_000, "data.jsonl", "text/jsonl", "fine-tune");
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
