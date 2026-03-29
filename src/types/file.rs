// File types — re-exported from openai-types.
pub use openai_types::file::*;

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
    fn test_deserialize_file_list_with_pagination() {
        let json = r#"{
            "object": "list",
            "data": [{
                "id": "file-abc123",
                "object": "file",
                "bytes": 120000,
                "created_at": 1677610602,
                "filename": "data.jsonl",
                "purpose": "fine-tune",
                "status": "processed"
            }],
            "has_more": true,
            "first_id": "file-abc123",
            "last_id": "file-abc123"
        }"#;
        let list: FileList = serde_json::from_str(json).unwrap();
        assert_eq!(list.data.len(), 1);
        assert_eq!(list.has_more, Some(true));
        assert_eq!(list.first_id.as_deref(), Some("file-abc123"));
        assert_eq!(list.last_id.as_deref(), Some("file-abc123"));
    }

    #[test]
    fn test_file_list_params_to_query() {
        let params = FileListParams::new()
            .after("file-cursor")
            .limit(10)
            .order("desc")
            .purpose(FilePurpose::FineTune);
        let query = params.to_query();
        assert!(query.contains(&("after".into(), "file-cursor".into())));
        assert!(query.contains(&("limit".into(), "10".into())));
        assert!(query.contains(&("order".into(), "desc".into())));
        assert!(query.contains(&("purpose".into(), "fine-tune".into())));
    }

    #[test]
    fn test_deserialize_file_deleted() {
        let json = r#"{"id": "file-abc123", "object": "file", "deleted": true}"#;
        let deleted: FileDeleted = serde_json::from_str(json).unwrap();
        assert!(deleted.deleted);
    }
}
