// Fine-tuning types — re-exported from openai-types.
pub use openai_types::fine_tuning::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_create_request() {
        let req = FineTuningJobCreateRequest::new("gpt-4o-mini", "file-abc123");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o-mini");
        assert_eq!(json["training_file"], "file-abc123");
    }

    #[test]
    fn test_deserialize_fine_tuning_job() {
        let json = r#"{
            "id": "ftjob-abc123",
            "object": "fine_tuning.job",
            "created_at": 1677610602,
            "model": "gpt-4o-mini",
            "training_file": "file-abc123",
            "status": "running",
            "organization_id": "org-123",
            "result_files": [],
            "seed": 42
        }"#;
        let job: FineTuningJob = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "ftjob-abc123");
        assert_eq!(job.status, FineTuningStatus::Running);
    }

    #[test]
    fn test_deserialize_fine_tuning_event() {
        let json = r#"{
            "id": "ftevent-abc123",
            "object": "fine_tuning.job.event",
            "created_at": 1677610602,
            "level": "info",
            "message": "Training started"
        }"#;
        let event: FineTuningJobEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.level, FineTuningEventLevel::Info);
        assert_eq!(event.message, "Training started");
    }
}
