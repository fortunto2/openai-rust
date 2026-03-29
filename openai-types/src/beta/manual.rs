// Manual: hand-crafted beta types (Assistants, Threads, Runs, Vector Stores).
// These supplement the auto-generated _gen.rs types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export shared types (canonical definitions in shared/common.rs)
pub use crate::shared::{Role, SortOrder};

/// Status of a thread run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RunStatus {
    Queued,
    InProgress,
    RequiresAction,
    Cancelling,
    Cancelled,
    Failed,
    Completed,
    Incomplete,
    Expired,
    /// Catch-all for unknown variants (forward compatibility).
    Other(String),
}

impl serde::Serialize for RunStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Queued => serializer.serialize_str("queued"),
            Self::InProgress => serializer.serialize_str("in_progress"),
            Self::RequiresAction => serializer.serialize_str("requires_action"),
            Self::Cancelling => serializer.serialize_str("cancelling"),
            Self::Cancelled => serializer.serialize_str("cancelled"),
            Self::Failed => serializer.serialize_str("failed"),
            Self::Completed => serializer.serialize_str("completed"),
            Self::Incomplete => serializer.serialize_str("incomplete"),
            Self::Expired => serializer.serialize_str("expired"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> serde::Deserialize<'de> for RunStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "queued" => Ok(Self::Queued),
            "in_progress" => Ok(Self::InProgress),
            "requires_action" => Ok(Self::RequiresAction),
            "cancelling" => Ok(Self::Cancelling),
            "cancelled" => Ok(Self::Cancelled),
            "failed" => Ok(Self::Failed),
            "completed" => Ok(Self::Completed),
            "incomplete" => Ok(Self::Incomplete),
            "expired" => Ok(Self::Expired),
            _ => Ok(Self::Other(s)),
        }
    }
}

/// Status of a vector store.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VectorStoreStatus {
    Expired,
    InProgress,
    Completed,
    /// Catch-all for unknown variants (forward compatibility).
    Other(String),
}

impl serde::Serialize for VectorStoreStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Expired => serializer.serialize_str("expired"),
            Self::InProgress => serializer.serialize_str("in_progress"),
            Self::Completed => serializer.serialize_str("completed"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> serde::Deserialize<'de> for VectorStoreStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "expired" => Ok(Self::Expired),
            "in_progress" => Ok(Self::InProgress),
            "completed" => Ok(Self::Completed),
            _ => Ok(Self::Other(s)),
        }
    }
}

// ── Tool types ──

/// A tool available to an assistant or run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum BetaTool {
    /// Code interpreter tool.
    #[serde(rename = "code_interpreter")]
    CodeInterpreter,
    /// File search tool with optional configuration.
    #[serde(rename = "file_search")]
    FileSearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        file_search: Option<FileSearchConfig>,
    },
    /// Function tool.
    #[serde(rename = "function")]
    Function { function: BetaFunctionDef },
}

/// Configuration for the file search tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct FileSearchConfig {
    /// Maximum number of results (1-50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_num_results: Option<i64>,
    /// Ranking options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking_options: Option<FileSearchRankingOptions>,
}

/// Ranking options for file search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct FileSearchRankingOptions {
    /// Score threshold (0.0-1.0).
    pub score_threshold: f64,
    /// Ranker to use: "auto" or "default_2024_08_21".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranker: Option<String>,
}

/// Function definition within a beta tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct BetaFunctionDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// An annotation on message text (file citation or file path).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct MessageAnnotation {
    /// Annotation type: "file_citation" or "file_path".
    #[serde(rename = "type")]
    pub type_: String,
    /// The text in the message content being annotated.
    #[serde(default)]
    pub text: Option<String>,
    /// Start index of the annotation in the text.
    #[serde(default)]
    pub start_index: Option<i64>,
    /// End index of the annotation in the text.
    #[serde(default)]
    pub end_index: Option<i64>,
    /// File citation details (for file_citation type).
    #[serde(default)]
    pub file_citation: Option<FileCitation>,
    /// File path details (for file_path type).
    #[serde(default)]
    pub file_path: Option<FilePath>,
}

/// File citation in an annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct FileCitation {
    pub file_id: String,
    #[serde(default)]
    pub quote: Option<String>,
}

/// File path in an annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct FilePath {
    pub file_id: String,
}

// ── Assistants ──

/// Request body for creating an assistant.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct AssistantCreateRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<BetaTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
}

impl AssistantCreateRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            name: None,
            description: None,
            instructions: None,
            tools: None,
            metadata: None,
            temperature: None,
            top_p: None,
        }
    }
}

/// An assistant object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Assistant {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub model: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub tools: Vec<BetaTool>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
}

/// List of assistants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct AssistantList {
    pub object: String,
    pub data: Vec<Assistant>,
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

/// Deleted assistant response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct AssistantDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

// ── Threads ──

/// Request body for creating a thread.
#[derive(Debug, Clone, Default, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ThreadCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<ThreadMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// A message in a thread creation request.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ThreadMessage {
    pub role: Role,
    pub content: String,
}

/// A thread object.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Thread {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Deleted thread response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ThreadDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

/// A message within a thread.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Message {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub thread_id: String,
    pub role: Role,
    pub content: Vec<MessageContent>,
    #[serde(default)]
    pub assistant_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Content block in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct MessageContent {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub text: Option<MessageText>,
}

/// Text content in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct MessageText {
    pub value: String,
    #[serde(default)]
    pub annotations: Vec<MessageAnnotation>,
}

/// Request body for creating a message.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct MessageCreateRequest {
    pub role: Role,
    pub content: String,
}

/// List of messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct MessageList {
    pub object: String,
    pub data: Vec<Message>,
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

// ── Runs ──

/// Request body for creating a run.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct RunCreateRequest {
    pub assistant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<BetaTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl RunCreateRequest {
    pub fn new(assistant_id: impl Into<String>) -> Self {
        Self {
            assistant_id: assistant_id.into(),
            instructions: None,
            tools: None,
            metadata: None,
            model: None,
        }
    }
}

/// A run object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Run {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub thread_id: String,
    pub assistant_id: String,
    pub status: RunStatus,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub tools: Vec<BetaTool>,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub cancelled_at: Option<i64>,
    #[serde(default)]
    pub failed_at: Option<i64>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Submit tool outputs request.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct SubmitToolOutputsRequest {
    pub tool_outputs: Vec<ToolOutput>,
}

/// A single tool output.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ToolOutput {
    pub tool_call_id: String,
    pub output: String,
}

// ── Vector Stores ──

/// Request body for creating a vector store.
#[derive(Debug, Clone, Default, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct VectorStoreCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// A vector store object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct VectorStore {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    #[serde(default)]
    pub name: Option<String>,
    pub status: VectorStoreStatus,
    #[serde(default)]
    pub file_counts: Option<VectorStoreFileCounts>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// File counts for a vector store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct VectorStoreFileCounts {
    pub in_progress: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
    pub total: i64,
}

/// List of vector stores.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct VectorStoreList {
    pub object: String,
    pub data: Vec<VectorStore>,
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

/// Deleted vector store response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct VectorStoreDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

// ── Pagination params ──

/// Parameters for listing assistants with pagination.
#[derive(Debug, Clone, Default)]
pub struct AssistantListParams {
    /// Cursor for pagination -- fetch results after this assistant ID.
    pub after: Option<String>,
    /// Cursor for backward pagination -- fetch results before this assistant ID.
    pub before: Option<String>,
    /// Maximum number of results per page (1-100).
    pub limit: Option<i64>,
    /// Sort order by `created_at`.
    pub order: Option<SortOrder>,
}

impl AssistantListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
        self
    }

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(ref before) = self.before {
            q.push(("before".into(), before.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        if let Some(ref order) = self.order {
            q.push((
                "order".into(),
                serde_json::to_value(order)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        q
    }
}

/// Parameters for listing messages with pagination.
#[derive(Debug, Clone, Default)]
pub struct MessageListParams {
    /// Cursor for pagination -- fetch results after this message ID.
    pub after: Option<String>,
    /// Cursor for backward pagination -- fetch results before this message ID.
    pub before: Option<String>,
    /// Maximum number of results per page (1-100).
    pub limit: Option<i64>,
    /// Sort order by `created_at`.
    pub order: Option<SortOrder>,
    /// Filter by run ID.
    pub run_id: Option<String>,
}

impl MessageListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn run_id(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(ref before) = self.before {
            q.push(("before".into(), before.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        if let Some(ref order) = self.order {
            q.push((
                "order".into(),
                serde_json::to_value(order)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        if let Some(ref run_id) = self.run_id {
            q.push(("run_id".into(), run_id.clone()));
        }
        q
    }
}

/// Parameters for listing vector stores with pagination.
#[derive(Debug, Clone, Default)]
pub struct VectorStoreListParams {
    /// Cursor for pagination -- fetch results after this vector store ID.
    pub after: Option<String>,
    /// Cursor for backward pagination -- fetch results before this vector store ID.
    pub before: Option<String>,
    /// Maximum number of results per page (1-100).
    pub limit: Option<i64>,
    /// Sort order by `created_at`.
    pub order: Option<SortOrder>,
}

impl VectorStoreListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
        self
    }

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(ref before) = self.before {
            q.push(("before".into(), before.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        if let Some(ref order) = self.order {
            q.push((
                "order".into(),
                serde_json::to_value(order)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        q
    }
}

/// Parameters for listing runs with pagination.
#[derive(Debug, Clone, Default)]
pub struct RunListParams {
    /// Cursor for pagination -- fetch results after this run ID.
    pub after: Option<String>,
    /// Cursor for backward pagination -- fetch results before this run ID.
    pub before: Option<String>,
    /// Maximum number of results per page (1-100).
    pub limit: Option<i64>,
    /// Sort order by `created_at`.
    pub order: Option<SortOrder>,
}

impl RunListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    pub fn before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
        self
    }

    /// Convert to query parameter pairs for the HTTP request.
    pub fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(ref after) = self.after {
            q.push(("after".into(), after.clone()));
        }
        if let Some(ref before) = self.before {
            q.push(("before".into(), before.clone()));
        }
        if let Some(limit) = self.limit {
            q.push(("limit".into(), limit.to_string()));
        }
        if let Some(ref order) = self.order {
            q.push((
                "order".into(),
                serde_json::to_value(order)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        q
    }
}
