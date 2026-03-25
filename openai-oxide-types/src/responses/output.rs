// Output items — the typed output from a Response.

use serde::{Deserialize, Serialize};

use super::common::Role;
use super::response::ResponseAnnotation;

/// Content block within an output item.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseOutputContent {
    /// Content type (e.g. "output_text", "refusal").
    #[serde(rename = "type")]
    pub type_: String,
    /// Text content (for output_text blocks).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Annotations on the text (citations, file paths, etc.).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<ResponseAnnotation>>,
}

/// Output item in a response.
///
/// Covers multiple output types: `message`, `function_call`, `web_search_call`, etc.
/// Uses a flat struct with optional fields rather than a tagged enum for maximum
/// forward compatibility with new output types.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResponseOutputItem {
    /// Item type: "message", "function_call", "function_call_output", "web_search_call", etc.
    #[serde(rename = "type")]
    pub type_: String,
    /// Unique ID of the output item.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Role (for message items).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    /// Content blocks (for message items).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ResponseOutputContent>>,
    /// Item status: "in_progress", "completed", "incomplete".
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Function name (for function_call items).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// JSON-encoded arguments string (for function_call items).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    /// Unique call ID for matching with function_call_output (for function_call items).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
}

/// A typed output item enum (discriminated union).
///
/// Discriminated by the `type` field. Covers message, function_call, reasoning,
/// and other output types. Used where stronger typing is preferred.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum OutputItem {
    /// Output message from the model.
    #[serde(rename = "message")]
    Message {
        /// Unique ID.
        #[serde(default)]
        id: Option<String>,
        /// Role — always "assistant".
        #[serde(default)]
        role: Option<Role>,
        /// Content blocks.
        #[serde(default)]
        content: Option<Vec<ResponseOutputContent>>,
        /// Item status.
        #[serde(default)]
        status: Option<String>,
    },
    /// Function call from the model.
    #[serde(rename = "function_call")]
    FunctionCall(FunctionToolCall),
    /// Reasoning chain-of-thought.
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningItem),
    /// Web search call.
    #[serde(rename = "web_search_call")]
    WebSearchCall(serde_json::Value),
    /// File search call.
    #[serde(rename = "file_search_call")]
    FileSearchCall(serde_json::Value),
    /// Catch-all for unknown output item types.
    #[serde(other)]
    Other,
}

/// A function tool call from the model.
///
/// Maps to Python SDK `ResponseFunctionToolCall`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionToolCall {
    /// JSON-encoded arguments string.
    pub arguments: String,
    /// Unique call ID for matching with function_call_output.
    pub call_id: String,
    /// Function name.
    pub name: String,
    /// Unique ID of the function tool call (populated when returned via API).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Item status.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// A reasoning chain-of-thought item.
///
/// Contains summary, optional content, and optional encrypted content for
/// multi-turn replay. Maps to Python SDK `ResponseReasoningItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningItem {
    /// Unique identifier.
    pub id: String,
    /// Reasoning summary parts.
    pub summary: Vec<SummaryPart>,
    /// Reasoning text content.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ReasoningContent>>,
    /// Encrypted content for multi-turn replay.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    /// Item status.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// A part of a reasoning summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum SummaryPart {
    /// Summary text content.
    #[serde(rename = "summary_text")]
    SummaryText(SummaryTextContent),
}

/// Text content within a reasoning summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SummaryTextContent {
    /// The summary text.
    pub text: String,
}

/// Reasoning text content within a reasoning item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReasoningContent {
    /// The reasoning text.
    pub text: String,
    /// Content type — always "reasoning_text".
    #[serde(rename = "type")]
    #[serde(default = "default_reasoning_text_type")]
    pub type_: String,
}

fn default_reasoning_text_type() -> String {
    "reasoning_text".to_string()
}

/// A function call extracted from response output.
///
/// Convenience struct with parsed JSON arguments.
#[derive(Debug, Clone)]
pub struct FunctionCall {
    /// The call ID for matching with function_call_output.
    pub call_id: String,
    /// Function name.
    pub name: String,
    /// Parsed JSON arguments.
    pub arguments: serde_json::Value,
}
