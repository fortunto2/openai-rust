// Beta API types — Assistants, Threads, Runs, Vector Stores

use serde::{Deserialize, Serialize};

// ── Assistants ──

/// Request body for creating an assistant.
#[derive(Debug, Clone, Serialize)]
pub struct AssistantCreateRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
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
#[derive(Debug, Clone, Deserialize)]
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
    pub tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
}

/// List of assistants.
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantList {
    pub object: String,
    pub data: Vec<Assistant>,
}

/// Deleted assistant response.
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

// ── Threads ──

/// Request body for creating a thread.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ThreadCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<ThreadMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// A message in a thread creation request.
#[derive(Debug, Clone, Serialize)]
pub struct ThreadMessage {
    pub role: String,
    pub content: String,
}

/// A thread object.
#[derive(Debug, Clone, Deserialize)]
pub struct Thread {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Deleted thread response.
#[derive(Debug, Clone, Deserialize)]
pub struct ThreadDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

/// A message within a thread.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub thread_id: String,
    pub role: String,
    pub content: Vec<MessageContent>,
    #[serde(default)]
    pub assistant_id: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Content block in a message.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageContent {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub text: Option<MessageText>,
}

/// Text content in a message.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageText {
    pub value: String,
    #[serde(default)]
    pub annotations: Vec<serde_json::Value>,
}

/// Request body for creating a message.
#[derive(Debug, Clone, Serialize)]
pub struct MessageCreateRequest {
    pub role: String,
    pub content: String,
}

/// List of messages.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageList {
    pub object: String,
    pub data: Vec<Message>,
}

// ── Runs ──

/// Request body for creating a run.
#[derive(Debug, Clone, Serialize)]
pub struct RunCreateRequest {
    pub assistant_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
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
#[derive(Debug, Clone, Deserialize)]
pub struct Run {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    pub thread_id: String,
    pub assistant_id: String,
    pub status: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub cancelled_at: Option<i64>,
    #[serde(default)]
    pub failed_at: Option<i64>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Submit tool outputs request.
#[derive(Debug, Clone, Serialize)]
pub struct SubmitToolOutputsRequest {
    pub tool_outputs: Vec<ToolOutput>,
}

/// A single tool output.
#[derive(Debug, Clone, Serialize)]
pub struct ToolOutput {
    pub tool_call_id: String,
    pub output: String,
}

// ── Vector Stores ──

/// Request body for creating a vector store.
#[derive(Debug, Clone, Default, Serialize)]
pub struct VectorStoreCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// A vector store object.
#[derive(Debug, Clone, Deserialize)]
pub struct VectorStore {
    pub id: String,
    pub object: String,
    pub created_at: i64,
    #[serde(default)]
    pub name: Option<String>,
    pub status: String,
    #[serde(default)]
    pub file_counts: Option<VectorStoreFileCounts>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// File counts for a vector store.
#[derive(Debug, Clone, Deserialize)]
pub struct VectorStoreFileCounts {
    pub in_progress: i64,
    pub completed: i64,
    pub failed: i64,
    pub cancelled: i64,
    pub total: i64,
}

/// List of vector stores.
#[derive(Debug, Clone, Deserialize)]
pub struct VectorStoreList {
    pub object: String,
    pub data: Vec<VectorStore>,
}

/// Deleted vector store response.
#[derive(Debug, Clone, Deserialize)]
pub struct VectorStoreDeleted {
    pub id: String,
    pub deleted: bool,
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_assistant_create() {
        let req = AssistantCreateRequest::new("gpt-4o");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
    }

    #[test]
    fn test_deserialize_assistant() {
        let json = r#"{
            "id": "asst_abc123",
            "object": "assistant",
            "created_at": 1699009709,
            "model": "gpt-4o",
            "tools": [{"type": "code_interpreter"}]
        }"#;
        let asst: Assistant = serde_json::from_str(json).unwrap();
        assert_eq!(asst.id, "asst_abc123");
        assert_eq!(asst.tools.len(), 1);
    }

    #[test]
    fn test_deserialize_thread() {
        let json = r#"{
            "id": "thread_abc123",
            "object": "thread",
            "created_at": 1699012949
        }"#;
        let thread: Thread = serde_json::from_str(json).unwrap();
        assert_eq!(thread.id, "thread_abc123");
    }

    #[test]
    fn test_deserialize_run() {
        let json = r#"{
            "id": "run_abc123",
            "object": "thread.run",
            "created_at": 1699012949,
            "thread_id": "thread_abc123",
            "assistant_id": "asst_abc123",
            "status": "completed",
            "tools": []
        }"#;
        let run: Run = serde_json::from_str(json).unwrap();
        assert_eq!(run.status, "completed");
    }

    #[test]
    fn test_deserialize_vector_store() {
        let json = r#"{
            "id": "vs_abc123",
            "object": "vector_store",
            "created_at": 1699012949,
            "name": "My Store",
            "status": "completed",
            "file_counts": {
                "in_progress": 0,
                "completed": 5,
                "failed": 0,
                "cancelled": 0,
                "total": 5
            }
        }"#;
        let vs: VectorStore = serde_json::from_str(json).unwrap();
        assert_eq!(vs.id, "vs_abc123");
        assert_eq!(vs.file_counts.unwrap().completed, 5);
    }
}
