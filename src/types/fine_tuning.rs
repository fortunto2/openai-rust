// Fine-tuning types — mirrors openai-python types/fine_tuning/

use serde::{Deserialize, Serialize};

/// Status of a fine-tuning job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FineTuningStatus {
    ValidatingFiles,
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

/// Event severity level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FineTuningEventLevel {
    Info,
    Warn,
    Error,
}

// ── Request types ──

/// Request body for `POST /fine_tuning/jobs`.
#[derive(Debug, Clone, Serialize)]
pub struct FineTuningJobCreateRequest {
    /// Base model to fine-tune.
    pub model: String,

    /// Training file ID.
    pub training_file: String,

    /// Hyperparameters for the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<Hyperparameters>,

    /// Suffix for the fine-tuned model name (max 64 chars).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Validation file ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_file: Option<String>,

    /// Seed for reproducibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}

impl FineTuningJobCreateRequest {
    pub fn new(model: impl Into<String>, training_file: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            training_file: training_file.into(),
            hyperparameters: None,
            suffix: None,
            validation_file: None,
            seed: None,
        }
    }
}

/// Hyperparameters for fine-tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperparameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<crate::types::common::AutoOrFixed<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<crate::types::common::AutoOrFixed<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_rate_multiplier: Option<crate::types::common::AutoOrFixed<f64>>,
}

// ── Response types ──

/// Error info for a failed fine-tuning job.
#[derive(Debug, Clone, Deserialize)]
pub struct FineTuningError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub param: Option<String>,
}

/// A fine-tuning job object.
#[derive(Debug, Clone, Deserialize)]
pub struct FineTuningJob {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub model: String,
    pub training_file: String,
    pub status: FineTuningStatus,
    #[serde(default)]
    pub fine_tuned_model: Option<String>,
    #[serde(default)]
    pub finished_at: Option<i64>,
    #[serde(default)]
    pub error: Option<FineTuningError>,
    #[serde(default)]
    pub hyperparameters: Option<Hyperparameters>,
    pub organization_id: String,
    #[serde(default)]
    pub result_files: Vec<String>,
    #[serde(default)]
    pub trained_tokens: Option<i64>,
    #[serde(default)]
    pub validation_file: Option<String>,
    #[serde(default)]
    pub estimated_finish: Option<i64>,
    pub seed: i64,
}

/// List of fine-tuning jobs.
#[derive(Debug, Clone, Deserialize)]
pub struct FineTuningJobList {
    pub object: String,
    pub data: Vec<FineTuningJob>,
    #[serde(default)]
    pub has_more: bool,
}

/// A fine-tuning job event.
#[derive(Debug, Clone, Deserialize)]
pub struct FineTuningJobEvent {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub level: FineTuningEventLevel,
    pub message: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
}

/// List of fine-tuning job events.
#[derive(Debug, Clone, Deserialize)]
pub struct FineTuningJobEventList {
    pub object: String,
    pub data: Vec<FineTuningJobEvent>,
    #[serde(default)]
    pub has_more: bool,
}

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
