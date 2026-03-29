// Manual: hand-crafted batch types (enums, builders, response structs).

use serde::{Deserialize, Serialize};

/// Status of a batch job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum BatchStatus {
    #[serde(rename = "validating")]
    Validating,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "finalizing")]
    Finalizing,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "cancelling")]
    Cancelling,
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// Request body for `POST /batches`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct BatchCreateRequest {
    /// File ID of the input JSONL file.
    pub input_file_id: String,

    /// API endpoint (e.g. "/v1/chat/completions").
    pub endpoint: String,

    /// Completion window (currently only "24h").
    pub completion_window: String,

    /// Metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl BatchCreateRequest {
    pub fn new(
        input_file_id: impl Into<String>,
        endpoint: impl Into<String>,
        completion_window: impl Into<String>,
    ) -> Self {
        Self {
            input_file_id: input_file_id.into(),
            endpoint: endpoint.into(),
            completion_window: completion_window.into(),
            metadata: None,
        }
    }
}

/// A batch object.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Batch {
    pub id: String,
    pub object: String,
    pub endpoint: String,
    pub input_file_id: String,
    pub completion_window: String,
    pub status: BatchStatus,
    pub created_at: i64,
    #[serde(default)]
    pub output_file_id: Option<String>,
    #[serde(default)]
    pub error_file_id: Option<String>,
    #[serde(default)]
    pub in_progress_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub failed_at: Option<i64>,
    #[serde(default)]
    pub cancelled_at: Option<i64>,
    #[serde(default)]
    pub expired_at: Option<i64>,
    #[serde(default)]
    pub request_counts: Option<BatchRequestCounts>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Request counts for a batch.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct BatchRequestCounts {
    pub total: i64,
    pub completed: i64,
    pub failed: i64,
}

/// List of batches.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct BatchList {
    pub object: String,
    pub data: Vec<Batch>,
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

/// Parameters for listing batches with pagination.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct BatchListParams {
    /// Cursor for pagination -- fetch results after this batch ID.
    pub after: Option<String>,
    /// Maximum number of results per page (1-100).
    pub limit: Option<i64>,
}

impl BatchListParams {
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

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        q
    }
}
