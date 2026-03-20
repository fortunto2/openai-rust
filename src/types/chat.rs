// Chat completion types — mirrors openai-python types/chat/

use serde::{Deserialize, Serialize};

use super::common::{Role, Usage};

// ── Request types ──

/// Request body for `POST /chat/completions`.
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    /// Model ID, e.g. "gpt-4o", "gpt-4o-mini".
    pub model: String,

    /// Messages in the conversation.
    pub messages: Vec<ChatCompletionMessageParam>,

    /// Penalty for frequent tokens. Range: -2.0 to 2.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,

    /// Modify likelihood of specific tokens. Maps token ID → bias (-100 to 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<std::collections::HashMap<String, i32>>,

    /// Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of most likely tokens to return at each position (0–20).
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
    pub service_tier: Option<String>,

    /// Up to 4 stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Stop>,

    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Stream options (e.g., include_usage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,

    /// Sampling temperature (0–2).
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
    pub metadata: Option<std::collections::HashMap<String, String>>,

    /// Output modalities: ["text"] or ["text", "audio"].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,

    /// Reasoning effort for reasoning models (o-series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

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

    /// DEPRECATED: Maximum number of tokens to generate.
    /// Use max_completion_tokens instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,

    /// DEPRECATED: A list of functions the model may call.
    /// Use tools instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<FunctionDef>>,

    /// DEPRECATED: Controls how the model calls functions.
    /// Use tool_choice instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<serde_json::Value>,
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

    /// Set the temperature (0–2).
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
    pub fn reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(effort.into());
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
#[serde(untagged)]
#[non_exhaustive]
pub enum Stop {
    Single(String),
    Multiple(Vec<String>),
}

/// Stream options for chat completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamOptions {
    /// If true, stream includes a final chunk with usage stats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
}

/// Audio output parameters for chat completions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionAudioParam {
    /// Audio output format.
    pub format: String,
    /// Voice to use for audio output.
    pub voice: String,
}

/// Predicted output content for the Predicted Outputs feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionContent {
    /// Always "content".
    #[serde(rename = "type")]
    pub type_: String,
    /// The predicted content.
    pub content: serde_json::Value,
}

/// Web search options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchOptions {
    /// Search context size: "low", "medium", "high".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_context_size: Option<String>,
    /// User location for search relevance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_location: Option<WebSearchUserLocation>,
}

/// User location for web search.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(untagged)]
#[non_exhaustive]
pub enum UserContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// A content part in a multi-part user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Image URL reference in a content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Audio input in a content part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudio {
    pub data: String,
    pub format: String,
}

// ── Tool / function calling types ──

/// A tool available to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionDef,
}

/// Function definition within a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

/// How the model picks tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct ToolChoiceFunction {
    pub name: String,
}

/// A tool call made by the assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: FunctionCall,
}

/// A function call within a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

// ── Response types ──

/// Response from `POST /chat/completions`.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub created: i64,
    pub model: String,
    pub object: String,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A single choice in a chat completion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChoice {
    pub finish_reason: String,
    pub index: i32,
    pub message: ChatCompletionMessage,
    #[serde(default)]
    pub logprobs: Option<ChoiceLogprobs>,
}

/// The assistant's message in a response.
#[derive(Debug, Clone, Deserialize)]
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
pub struct ChoiceLogprobs {
    #[serde(default)]
    pub content: Option<Vec<TokenLogprob>>,
    #[serde(default)]
    pub refusal: Option<Vec<TokenLogprob>>,
}

/// Log probability for a single token.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenLogprob {
    pub token: String,
    pub logprob: f64,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
    #[serde(default)]
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

/// Top logprob candidate.
#[derive(Debug, Clone, Deserialize)]
pub struct TopLogprob {
    pub token: String,
    pub logprob: f64,
    #[serde(default)]
    pub bytes: Option<Vec<u8>>,
}

/// URL citation annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub url_citation: Option<UrlCitation>,
}

/// URL citation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlCitation {
    pub end_index: i32,
    pub start_index: i32,
    pub title: String,
    pub url: String,
}

// ── Streaming types ──

/// A chunk in a streaming chat completion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<ChunkChoice>,
    pub created: i64,
    pub model: String,
    pub object: String,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub system_fingerprint: Option<String>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A choice within a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkChoice {
    pub delta: ChoiceDelta,
    pub finish_reason: Option<String>,
    pub index: i32,
    #[serde(default)]
    pub logprobs: Option<ChoiceLogprobs>,
}

/// Delta content in a streaming chunk.
#[derive(Debug, Clone, Deserialize)]
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
pub struct DeltaFunctionCall {
    #[serde(default)]
    pub arguments: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_chat_completion_response() {
        let json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created": 1677858242,
            "model": "gpt-4o-mini",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help you today?"
                },
                "logprobs": null,
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 13,
                "completion_tokens": 7,
                "total_tokens": 20
            }
        }"#;

        let resp: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "chatcmpl-abc123");
        assert_eq!(resp.object, "chat.completion");
        assert_eq!(resp.model, "gpt-4o-mini");
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].finish_reason, "stop");
        assert_eq!(
            resp.choices[0].message.content.as_deref(),
            Some("Hello! How can I help you today?")
        );
        assert_eq!(resp.choices[0].message.role, Role::Assistant);
        let usage = resp.usage.as_ref().unwrap();
        assert_eq!(usage.prompt_tokens, Some(13));
        assert_eq!(usage.completion_tokens, Some(7));
        assert_eq!(usage.total_tokens, Some(20));
    }

    #[test]
    fn test_deserialize_response_with_tool_calls() {
        let json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created": 1699896916,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_abc123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"Boston\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 82,
                "completion_tokens": 17,
                "total_tokens": 99
            }
        }"#;

        let resp: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices[0].finish_reason, "tool_calls");
        let tool_calls = resp.choices[0].message.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_abc123");
        assert_eq!(tool_calls[0].function.name, "get_weather");
        assert!(tool_calls[0].function.arguments.contains("Boston"));
    }

    #[test]
    fn test_serialize_request() {
        let req = ChatCompletionRequest::new(
            "gpt-4o",
            vec![
                ChatCompletionMessageParam::System {
                    content: "You are a helpful assistant.".into(),
                    name: None,
                },
                ChatCompletionMessageParam::User {
                    content: UserContent::Text("Hello".into()),
                    name: None,
                },
            ],
        );

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["messages"].as_array().unwrap().len(), 2);
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(
            json["messages"][0]["content"],
            "You are a helpful assistant."
        );
        assert_eq!(json["messages"][1]["role"], "user");
        assert_eq!(json["messages"][1]["content"], "Hello");
        // Optional fields should be absent
        assert!(json.get("temperature").is_none());
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn test_serialize_request_with_tools() {
        let mut req = ChatCompletionRequest::new(
            "gpt-4o",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("What's the weather?".into()),
                name: None,
            }],
        );
        req.tools = Some(vec![Tool {
            type_: "function".into(),
            function: FunctionDef {
                name: "get_weather".into(),
                description: Some("Get weather for a location".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                })),
                strict: None,
            },
        }]);
        req.tool_choice = Some(ToolChoice::Mode("auto".into()));

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["tools"][0]["type"], "function");
        assert_eq!(json["tools"][0]["function"]["name"], "get_weather");
        assert_eq!(json["tool_choice"], "auto");
    }

    #[test]
    fn test_serialize_multipart_user_message() {
        let msg = ChatCompletionMessageParam::User {
            content: UserContent::Parts(vec![
                ContentPart::Text {
                    text: "What's in this image?".into(),
                },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "https://example.com/image.png".into(),
                        detail: Some("high".into()),
                    },
                },
            ]),
            name: None,
        };

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "user");
        let parts = json["content"].as_array().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[1]["type"], "image_url");
        assert_eq!(parts[1]["image_url"]["detail"], "high");
    }

    #[test]
    fn test_deserialize_streaming_chunk() {
        let json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion.chunk",
            "created": 1677858242,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.id, "chatcmpl-abc123");
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hello"));
        assert_eq!(chunk.choices[0].delta.role, Some(Role::Assistant));
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    #[test]
    fn test_deserialize_streaming_chunk_with_tool_call() {
        let json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion.chunk",
            "created": 1677858242,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_abc123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"loc"
                        }
                    }]
                },
                "finish_reason": null
            }]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(json).unwrap();
        let tool_calls = chunk.choices[0].delta.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls[0].index, 0);
        assert_eq!(tool_calls[0].id.as_deref(), Some("call_abc123"));
        let func = tool_calls[0].function.as_ref().unwrap();
        assert_eq!(func.name.as_deref(), Some("get_weather"));
    }

    #[test]
    fn test_deserialize_streaming_done_chunk() {
        let json = r#"{
            "id": "chatcmpl-abc123",
            "object": "chat.completion.chunk",
            "created": 1677858242,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].finish_reason.as_deref(), Some("stop"));
        assert!(chunk.choices[0].delta.content.is_none());
    }

    #[test]
    fn test_response_format_json_schema() {
        let rf = ResponseFormat::JsonSchema {
            json_schema: JsonSchema {
                name: "math_response".into(),
                description: None,
                schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "answer": {"type": "number"}
                    }
                })),
                strict: Some(true),
            },
        };

        let json = serde_json::to_value(&rf).unwrap();
        assert_eq!(json["type"], "json_schema");
        assert_eq!(json["json_schema"]["name"], "math_response");
        assert_eq!(json["json_schema"]["strict"], true);
    }

    #[test]
    fn test_stop_single_and_multiple() {
        let single = Stop::Single("END".into());
        let json = serde_json::to_value(&single).unwrap();
        assert_eq!(json, "END");

        let multi = Stop::Multiple(vec!["END".into(), "STOP".into()]);
        let json = serde_json::to_value(&multi).unwrap();
        assert_eq!(json, serde_json::json!(["END", "STOP"]));
    }

    #[test]
    fn test_builder_pattern() {
        let req = ChatCompletionRequest::new(
            "gpt-4o",
            vec![ChatCompletionMessageParam::User {
                content: UserContent::Text("Hello".into()),
                name: None,
            }],
        )
        .temperature(0.7)
        .max_completion_tokens(1000)
        .reasoning_effort("high")
        .top_p(0.9)
        .seed(42)
        .store(true)
        .n(2)
        .user("user-123")
        .response_format(ResponseFormat::JsonObject);

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["temperature"], 0.7);
        assert_eq!(json["max_completion_tokens"], 1000);
        assert_eq!(json["reasoning_effort"], "high");
        assert_eq!(json["top_p"], 0.9);
        assert_eq!(json["seed"], 42);
        assert_eq!(json["store"], true);
        assert_eq!(json["n"], 2);
        assert_eq!(json["user"], "user-123");
        assert_eq!(json["response_format"]["type"], "json_object");
    }
}
