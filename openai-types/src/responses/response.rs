// MANUAL — hand-maintained. py2rust sync will not overwrite.
// Main Response struct and related types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::create::{Reasoning, ResponseTextConfig};
use super::output::{FunctionCall, ResponseOutputItem};
use super::tools::{ResponseTool, ResponseToolChoice};

/// An error returned when the model fails to generate a Response.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ResponseError {
    /// The error code (e.g. "server_error", "rate_limit_exceeded", "invalid_prompt").
    pub code: String,
    /// A human-readable description of the error.
    pub message: String,
}

/// Details about why the response is incomplete.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct IncompleteDetails {
    /// The reason: "max_output_tokens" or "content_filter".
    #[serde(default)]
    pub reason: Option<String>,
}

/// An annotation on response output text (e.g. URL citation, file citation).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ResponseAnnotation {
    /// Annotation type (e.g. "url_citation", "file_citation", "file_path").
    #[serde(rename = "type")]
    pub type_: String,
    /// Start index in the text.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<i64>,
    /// End index in the text.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<i64>,
    /// URL for url_citation annotations.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Title for url_citation annotations.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// File ID for file_citation/file_path annotations.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
}

/// Input token usage details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct InputTokensDetails {
    /// Number of cached tokens.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<i64>,
}

/// Output token usage details.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct OutputTokensDetails {
    /// Number of reasoning tokens.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<i64>,
}

/// Usage for the Responses API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ResponseUsage {
    /// Number of input tokens.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<i64>,
    /// Number of output tokens.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<i64>,
    /// Total tokens used.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<i64>,
    /// Detailed input token breakdown.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens_details: Option<InputTokensDetails>,
    /// Detailed output token breakdown.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

/// Response from `POST /responses`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Response {
    /// Unique response identifier.
    pub id: String,
    /// Object type — always "response".
    pub object: String,
    /// Unix timestamp when the response was created.
    pub created_at: f64,
    /// Model used for generation.
    pub model: String,
    /// Output items (messages, function calls, reasoning, etc.).
    pub output: Vec<ResponseOutputItem>,
    /// Response status: "in_progress", "completed", "failed", "incomplete".
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Error details if the response failed.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
    /// Details about why the response is incomplete.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<IncompleteDetails>,
    /// System instructions used.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Metadata key-value pairs.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Temperature used.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Top-p (nucleus sampling) used.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Max output tokens limit.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,
    /// Previous response ID for multi-turn.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Token usage information.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponseUsage>,
    /// Tools used in the response.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ResponseTool>>,
    /// Tool choice setting.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ResponseToolChoice>,
    /// Whether parallel tool calls were enabled.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Truncation strategy.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    /// Reasoning configuration.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,
    /// Service tier used.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// Text output configuration.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextConfig>,
    /// Unix timestamp when the response completed.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<f64>,
    /// Whether the response ran in background mode.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    /// End user identifier.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Top log probabilities count.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i64>,
    /// Maximum number of tool calls.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<i64>,
}

impl Response {
    /// Get the text output, concatenating all text content blocks.
    pub fn output_text(&self) -> String {
        let mut result = String::new();
        for item in &self.output {
            if let Some(content) = &item.content {
                for block in content {
                    if block.type_ == "output_text"
                        && let Some(text) = &block.text
                    {
                        result.push_str(text);
                    }
                }
            }
        }
        result
    }

    /// Extract all function calls from the response output.
    pub fn function_calls(&self) -> Vec<FunctionCall> {
        self.output
            .iter()
            .filter(|item| item.type_ == "function_call")
            .map(|item| {
                let call_id = item
                    .call_id
                    .as_deref()
                    .or(item.id.as_deref())
                    .unwrap_or("unknown")
                    .to_string();
                let name = item.name.clone().unwrap_or_default();
                let arguments = item
                    .arguments
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok())
                    .unwrap_or(serde_json::Value::Object(Default::default()));
                FunctionCall {
                    call_id,
                    name,
                    arguments,
                }
            })
            .collect()
    }

    /// Check if the response has any function calls.
    pub fn has_function_calls(&self) -> bool {
        self.output.iter().any(|item| item.type_ == "function_call")
    }
}

// Compat aliases for async-openai migration.

/// Alias for [`InputTokensDetails`].
pub type InputTokenDetails = InputTokensDetails;
/// Alias for [`OutputTokensDetails`].
pub type OutputTokenDetails = OutputTokensDetails;
