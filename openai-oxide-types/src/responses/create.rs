// Request types for the Responses API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::ReasoningEffort;
use super::input::ResponseInput;
use super::tools::{ResponseTool, ResponseToolChoice};

/// Summary mode for reasoning output.
pub use super::common::ReasoningSummary;

/// Reasoning configuration for o-series models.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Reasoning {
    /// Effort level for reasoning.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Summary mode for reasoning output.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

impl Reasoning {
    /// Builder-style: set effort.
    pub fn effort(&mut self, effort: ReasoningEffort) -> &mut Self {
        self.effort = Some(effort);
        self
    }
    /// Builder-style: set summary.
    pub fn summary(&mut self, summary: ReasoningSummary) -> &mut Self {
        self.summary = Some(summary);
        self
    }
}

/// Text output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextConfig {
    /// Format configuration (text, json_object, or json_schema).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ResponseTextFormat>,
    /// Verbosity level for the response.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,
}

/// Text output format for the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ResponseTextFormat {
    /// Plain text output.
    #[serde(rename = "text")]
    Text,
    /// JSON object output.
    #[serde(rename = "json_object")]
    JsonObject,
    /// JSON schema output with structured schema.
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// Schema name.
        name: String,
        /// Schema description.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// JSON Schema definition.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
        /// Whether to enforce strict schema validation.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

/// Request body for `POST /responses`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResponseCreateRequest {
    /// Model to use.
    #[serde(default)]
    pub model: String,

    /// Input text or messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<ResponseInput>,

    /// System instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ResponseTool>>,

    /// How the model selects tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ResponseToolChoice>,

    /// Whether to enable parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// Previous response ID for multi-turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,

    /// Temperature (0-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// Max output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i64>,

    /// Truncation strategy: "auto" or "disabled".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,

    /// Reasoning configuration for o-series models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<Reasoning>,

    /// Store for evals/distillation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// Metadata key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Additional data to include in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// Whether to stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Service tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    /// End user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Text output configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<ResponseTextConfig>,

    /// Prompt cache key — caches system prompt prefix server-side for faster repeat calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    /// Prompt cache retention: "in-memory" or "24h" for extended caching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,

    /// Whether to run in background mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
}

impl ResponseCreateRequest {
    /// Create a new request with the given model.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Set the input text or messages.
    pub fn input(mut self, input: impl Into<ResponseInput>) -> Self {
        self.input = Some(input.into());
        self
    }

    /// Set system instructions.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set the tools.
    pub fn tools(mut self, tools: Vec<ResponseTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set how the model selects tools.
    pub fn tool_choice(mut self, choice: ResponseToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Set previous response ID for multi-turn.
    pub fn previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.previous_response_id = Some(id.into());
        self
    }

    /// Set the temperature (0-2).
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max output tokens.
    pub fn max_output_tokens(mut self, max: i64) -> Self {
        self.max_output_tokens = Some(max);
        self
    }

    /// Set reasoning configuration.
    pub fn reasoning(mut self, reasoning: Reasoning) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Set truncation strategy.
    pub fn truncation(mut self, truncation: impl Into<String>) -> Self {
        self.truncation = Some(truncation.into());
        self
    }

    /// Enable storage for evals/distillation.
    pub fn store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Set model.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set text output configuration (format + verbosity).
    pub fn text(mut self, text: ResponseTextConfig) -> Self {
        self.text = Some(text);
        self
    }

    /// Set top_p (nucleus sampling).
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Enable or disable parallel tool calls.
    pub fn parallel_tool_calls(mut self, parallel: bool) -> Self {
        self.parallel_tool_calls = Some(parallel);
        self
    }

    /// Set metadata key-value pairs.
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set include fields for additional response data.
    pub fn include(mut self, include: Vec<String>) -> Self {
        self.include = Some(include);
        self
    }

    /// Set service tier ("auto", "default", "flex", "scale", "priority").
    pub fn service_tier(mut self, tier: impl Into<String>) -> Self {
        self.service_tier = Some(tier.into());
        self
    }

    /// Set end user identifier.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set prompt cache key for server-side system prompt caching.
    pub fn prompt_cache_key(mut self, key: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(key.into());
        self
    }

    /// Set prompt cache retention: "in-memory" or "24h".
    pub fn prompt_cache_retention(mut self, retention: impl Into<String>) -> Self {
        self.prompt_cache_retention = Some(retention.into());
        self
    }

    /// Run response in background mode.
    pub fn background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }
}
