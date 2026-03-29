// Manual: generic pagination type for list endpoints.

use serde::{Deserialize, Serialize};

/// Generic list response with cursor-based pagination.
///
/// Used by all list endpoints: models, files, batches, fine-tuning jobs, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub object: String,
    pub data: Vec<T>,
    #[serde(default)]
    pub has_more: Option<bool>,
    #[serde(default)]
    pub first_id: Option<String>,
    #[serde(default)]
    pub last_id: Option<String>,
}
