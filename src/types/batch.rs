// Batch types — re-exported from openai-types.
pub use openai_types::batch::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_batch_create() {
        let req = BatchCreateRequest::new("file-abc123", "/v1/chat/completions", "24h");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input_file_id"], "file-abc123");
        assert_eq!(json["endpoint"], "/v1/chat/completions");
        assert_eq!(json["completion_window"], "24h");
    }

    #[test]
    fn test_deserialize_batch() {
        let json = r#"{
            "id": "batch_abc123",
            "object": "batch",
            "endpoint": "/v1/chat/completions",
            "input_file_id": "file-abc123",
            "completion_window": "24h",
            "status": "completed",
            "created_at": 1699012949,
            "request_counts": {"total": 100, "completed": 95, "failed": 5}
        }"#;
        let batch: Batch = serde_json::from_str(json).unwrap();
        assert_eq!(batch.id, "batch_abc123");
        assert_eq!(batch.status, BatchStatus::Completed);
        let counts = batch.request_counts.unwrap();
        assert_eq!(counts.total, 100);
    }
}
