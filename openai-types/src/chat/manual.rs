// Manual: hand-crafted chat completion types (request, response, streaming, tools).
// These supplement the auto-generated _gen.rs types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export shared types (canonical definitions in shared/common.rs)
pub use crate::shared::{
    CompletionTokensDetails, FinishReason, PromptTokensDetails, ReasoningEffort, Role,
    SearchContextSize, ServiceTier, Usage,
};

// ── Request types ──

/// Request body for `POST /chat/completions`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChatCompletionRequest {
    /// Model ID, e.g. "gpt-4o", "gpt-4o-mini".
    pub model: String,

    /// Messages in the conversation.
    pub messages: Vec<ChatCompletionMessageParam>,

    /// Penalty for frequent tokens. Range: -2.0 to 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    /// Modify likelihood of specific tokens. Maps token ID -> bias (-100 to 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    /// Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of most likely tokens to return at each position (0-20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,

    /// Upper bound on tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i64>,

    /// Number of completions to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,

    /// Penalty for new tokens based on presence. Range: -2.0 to 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,

    /// Response format constraint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Seed for deterministic sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Service tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    /// Up to 4 stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Stream options (e.g., include_usage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    /// Sampling temperature (0-2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// How the model selects tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to enable parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// End user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Whether to store for evals/distillation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// Metadata key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Output modalities: ["text"] or ["text", "audio"].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,

    /// Reasoning effort for reasoning models (o-series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// Response verbosity level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,

    /// Audio output parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<ChatCompletionAudioParam>,

    /// Predicted output content (for Predicted Outputs feature).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<PredictionContent>,

    /// Web search options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,

    /// Stable key for prompt caching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    /// DEPRECATED: Maximum number of tokens to generate. Use max_completion_tokens instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,

    /// DEPRECATED: A list of functions the model may call. Use tools instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDef>>,

    /// DEPRECATED: Controls how the model calls functions. Use tool_choice instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCallOption>,
}

impl ChatCompletionRequest {
    /// Create a new request with the given model and messages.
    pub fn new(model: impl Into<String>, messages: Vec<ChatCompletionMessageParam>) -> Self {
        Self {
            model: model.into(),
            messages,
            frequency_penalty: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            max_completion_tokens: None,
            n: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            service_tier: None,
            stop: None,
            stream: None,
            stream_options: None,
            temperature: None,
            top_p: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            user: None,
            store: None,
            metadata: None,
            modalities: None,
            reasoning_effort: None,
            verbosity: None,
            audio: None,
            prediction: None,
            web_search_options: None,
            prompt_cache_key: None,
            max_tokens: None,
            functions: None,
            function_call: None,
        }
    }

    /// Set the model.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the messages.
    pub fn messages(mut self, messages: Vec<ChatCompletionMessageParam>) -> Self {
        self.messages = messages;
        self
    }

    /// Set the temperature (0-2).
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max completion tokens.
    pub fn max_completion_tokens(mut self, max: i64) -> Self {
        self.max_completion_tokens = Some(max);
        self
    }

    /// Set the tools.
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set the tool choice.
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Set the response format.
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Set reasoning effort for o-series models.
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    /// Set prediction content for Predicted Outputs.
    pub fn prediction(mut self, prediction: PredictionContent) -> Self {
        self.prediction = Some(prediction);
        self
    }

    /// Set top_p (nucleus sampling).
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Set seed for deterministic sampling.
    pub fn seed(mut self, seed: i64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set stop sequences.
    pub fn stop(mut self, stop: Stop) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Set user identifier.
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Enable storage for evals/distillation.
    pub fn store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Set number of completions.
    pub fn n(mut self, n: i32) -> Self {
        self.n = Some(n);
        self
    }
}

/// Stop sequences: either a single string or up to 4 strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum Stop {
    Single(String),
    Multiple(Vec<String>),
}

/// Stream options for chat completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct StreamOptions {
    /// If true, stream includes a final chunk with usage stats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// Audio output parameters for chat completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChatCompletionAudioParam {
    /// Audio output format.
    pub format: String,
    /// Voice to use for audio output.
    pub voice: String,
}

/// Predicted output content for the Predicted Outputs feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct PredictionContent {
    /// Always "content".
    #[serde(rename = "type")]
    pub type_: String,
    /// The predicted content.
    pub content: serde_json::Value,
}

/// Web search options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct WebSearchOptions {
    /// Search context size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<SearchContextSize>,
    /// User location for search relevance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
}

/// User location for web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct WebSearchUserLocation {
    /// Always "approximate".
    #[serde(rename = "type")]
    pub type_: String,
    /// Approximate location details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approximate: Option<ApproximateLocation>,
}

/// Approximate user location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ApproximateLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// Response format constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ResponseFormat {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json_object")]
    JsonObject,
    #[serde(rename = "json_schema")]
    JsonSchema { json_schema: JsonSchema },
}

/// JSON Schema for structured output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct JsonSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

// ── Message types (input) ──

/// A message in the conversation (request side).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(tag = "role")]
#[non_exhaustive]
pub enum ChatCompletionMessageParam {
    #[serde(rename = "system")]
    System {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename = "developer")]
    Developer {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename = "user")]
    User {
        content: UserContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    #[serde(rename = "assistant")]
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        refusal: Option<String>,
    },
    #[serde(rename = "tool")]
    Tool {
        content: String,
        tool_call_id: String,
    },
}

/// User message content: either a plain string or a list of content parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum UserContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// A content part in a multi-part user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
    #[serde(rename = "input_audio")]
    InputAudio { input_audio: InputAudio },
}

/// Image detail level for vision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// Image URL reference in a content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

/// Audio input in a content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct InputAudio {
    pub data: String,
    pub format: String,
}

// ── Tool / function calling types ──

/// A tool available to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Tool {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionDef,
}

/// Recursively strip `format` and `minimum:0` from JSON schema values.
/// schemars adds int32/int64/uint32/double format fields that some providers reject.
fn strip_format_recursive(value: &mut serde_json::Value) {
    if let Some(map) = value.as_object_mut() {
        map.remove("format");
        if map.get("minimum") == Some(&serde_json::json!(0.0)) {
            map.remove("minimum");
        }
        for v in map.values_mut() {
            strip_format_recursive(v);
        }
    } else if let Some(arr) = value.as_array_mut() {
        for v in arr {
            strip_format_recursive(v);
        }
    }
}

impl Tool {
    /// Create a standard function tool.
    /// Strips `format` and `minimum:0` fields recursively for cross-provider compatibility
    /// (schemars adds int32/int64/uint32/double which providers like Cerebras reject).
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        mut parameters: serde_json::Value,
    ) -> Self {
        strip_format_recursive(&mut parameters);
        Self {
            type_: "function".to_string(),
            function: FunctionDef {
                name: name.into(),
                description: Some(description.into()),
                parameters: Some(parameters),
                strict: Some(true),
            },
        }
    }

    /// Web search tool (used by gpt-4o-search models)
    pub fn web_search() -> Self {
        Self {
            type_: "web_search".to_string(),
            function: FunctionDef {
                name: "".to_string(),
                description: None,
                parameters: None,
                strict: None,
            },
        }
    }

    /// File search tool
    pub fn file_search() -> Self {
        Self {
            type_: "file_search".to_string(),
            function: FunctionDef {
                name: "".to_string(),
                description: None,
                parameters: None,
                strict: None,
            },
        }
    }

    /// Code interpreter tool
    pub fn code_interpreter() -> Self {
        Self {
            type_: "code_interpreter".to_string(),
            function: FunctionDef {
                name: "".to_string(),
                description: None,
                parameters: None,
                strict: None,
            },
        }
    }
}

/// Function definition within a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct FunctionDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// DEPRECATED: How the model calls functions (use ToolChoice instead).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum FunctionCallOption {
    /// "none" or "auto".
    Mode(String),
    /// Force a specific function.
    Named { name: String },
}

/// How the model picks tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum ToolChoice {
    /// "none", "auto", or "required"
    Mode(String),
    /// Force a specific function.
    Named {
        #[serde(rename = "type")]
        type_: String,
        function: ToolChoiceFunction,
    },
}

/// Specifies which function to call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ToolChoiceFunction {
    pub name: String,
}

/// A tool call made by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionCall,
}

/// A function call within a tool call (name + JSON arguments).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

// ── Response types ──

/// Response from `POST /chat/completions`.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub created: i64,
    pub model: String,
    pub object: String,
    #[serde(default)]
    pub service_tier: Option<ServiceTier>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A single choice in a chat completion response.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChatCompletionChoice {
    pub finish_reason: FinishReason,
    pub index: i32,
    pub message: ChatCompletionMessage,
    #[serde(default)]
    pub logprobs: Option<ChoiceLogprobs>,
}

/// The assistant's message in a response.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChatCompletionMessage {
    pub role: Role,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub refusal: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(default)]
    pub annotations: Option<Vec<Annotation>>,
}

/// Log probability information.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChoiceLogprobs {
    #[serde(default)]
    pub content: Option<Vec<TokenLogprob>>,
    #[serde(default)]
    pub refusal: Option<Vec<TokenLogprob>>,
}

/// Log probability for a single token.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct TokenLogprob {
    pub token: String,
    pub logprob: f64,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
    #[serde(default)]
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

/// Top logprob candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct TopLogprob {
    pub token: String,
    pub logprob: f64,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
}

/// URL citation annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Annotation {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub url_citation: Option<UrlCitation>,
}

/// URL citation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct UrlCitation {
    pub end_index: i32,
    pub start_index: i32,
    pub title: String,
    pub url: String,
}

// ── Streaming types ──

/// A chunk in a streaming chat completion response.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<ChunkChoice>,
    pub created: i64,
    pub model: String,
    pub object: String,
    #[serde(default)]
    pub service_tier: Option<ServiceTier>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A choice within a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChunkChoice {
    pub delta: ChoiceDelta,
    pub finish_reason: Option<FinishReason>,
    pub index: i32,
    #[serde(default)]
    pub logprobs: Option<ChoiceLogprobs>,
}

/// Delta content in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct ChoiceDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub refusal: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<DeltaToolCall>>,
}

/// A tool call delta in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct DeltaToolCall {
    pub index: i32,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub function: Option<DeltaFunctionCall>,
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
}

/// Function call delta in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct DeltaFunctionCall {
    #[serde(default)]
    pub arguments: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}
