// Responses API types — mirrors openai-python types/responses/

use serde::{Deserialize, Serialize};

use super::common::Role;

// ── Request types ──

/// Input for the Responses API.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ResponseInput {
    Text(String),
    Messages(Vec<ResponseInputItem>),
}

impl From<&str> for ResponseInput {
    fn from(s: &str) -> Self {
        ResponseInput::Text(s.to_string())
    }
}

impl From<String> for ResponseInput {
    fn from(s: String) -> Self {
        ResponseInput::Text(s)
    }
}

/// An input message for the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInputItem {
    pub role: Role,
    pub content: serde_json::Value,
}

/// How the model selects tools in the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ResponseToolChoice {
    /// "none", "auto", or "required".
    Mode(String),
    /// Force a specific function by name.
    Named {
        #[serde(rename = "type")]
        type_: String,
        function: ResponseToolChoiceFunction,
    },
}

/// Specifies which function to call in tool choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseToolChoiceFunction {
    pub name: String,
}

/// Request body for `POST /responses`.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseCreateRequest {
    /// Model to use.
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

    /// Temperature (0–2).
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
    pub metadata: Option<std::collections::HashMap<String, String>>,

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
}

impl ResponseCreateRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: None,
            instructions: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            previous_response_id: None,
            temperature: None,
            top_p: None,
            max_output_tokens: None,
            truncation: None,
            reasoning: None,
            store: None,
            metadata: None,
            include: None,
            stream: None,
            service_tier: None,
            user: None,
            text: None,
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

    /// Set the temperature (0–2).
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
}

/// Reasoning configuration for o-series models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reasoning {
    /// Effort level: "none", "minimal", "low", "medium", "high", "xhigh".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    /// Summary mode: "auto", "concise", "detailed".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Text output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTextConfig {
    /// Format configuration (text, json_object, or json_schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ResponseTextFormat>,
    /// Verbosity level for the response.
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
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

/// Tool types for the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ResponseTool {
    /// Function tool.
    #[serde(rename = "function")]
    Function {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    /// Web search tool.
    #[serde(rename = "web_search")]
    WebSearch {
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<serde_json::Value>,
    },
    /// File search tool.
    #[serde(rename = "file_search")]
    FileSearch {
        vector_store_ids: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_options: Option<serde_json::Value>,
    },
    /// Code interpreter tool.
    #[serde(rename = "code_interpreter")]
    CodeInterpreter {
        #[serde(skip_serializing_if = "Option::is_none")]
        container: Option<serde_json::Value>,
    },
    /// Computer use tool.
    #[serde(rename = "computer")]
    ComputerUse {},
    /// MCP tool.
    #[serde(rename = "mcp")]
    Mcp {
        server_label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        server_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        require_approval: Option<serde_json::Value>,
    },
    /// Image generation tool.
    #[serde(rename = "image_generation")]
    ImageGeneration {
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        quality: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<String>,
    },
}

// ── Response types ──

/// An error returned when the model fails to generate a Response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseError {
    /// The error code (e.g. "server_error", "rate_limit_exceeded", "invalid_prompt").
    pub code: String,
    /// A human-readable description of the error.
    pub message: String,
}

/// Details about why the response is incomplete.
#[derive(Debug, Clone, Deserialize)]
pub struct IncompleteDetails {
    /// The reason: "max_output_tokens" or "content_filter".
    #[serde(default)]
    pub reason: Option<String>,
}

/// An annotation on response output text (e.g. URL citation, file citation).
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseAnnotation {
    /// Annotation type (e.g. "url_citation", "file_citation", "file_path").
    #[serde(rename = "type")]
    pub type_: String,
    /// Start index in the text.
    #[serde(default)]
    pub start_index: Option<i64>,
    /// End index in the text.
    #[serde(default)]
    pub end_index: Option<i64>,
    /// URL for url_citation annotations.
    #[serde(default)]
    pub url: Option<String>,
    /// Title for url_citation annotations.
    #[serde(default)]
    pub title: Option<String>,
    /// File ID for file_citation/file_path annotations.
    #[serde(default)]
    pub file_id: Option<String>,
}

/// Output item in a response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseOutputItem {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub content: Option<Vec<ResponseOutputContent>>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Content block within an output item.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseOutputContent {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub annotations: Option<Vec<ResponseAnnotation>>,
}

/// Usage for the Responses API.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseUsage {
    #[serde(default)]
    pub input_tokens: Option<i64>,
    #[serde(default)]
    pub output_tokens: Option<i64>,
    #[serde(default)]
    pub total_tokens: Option<i64>,
    #[serde(default)]
    pub input_tokens_details: Option<InputTokensDetails>,
    #[serde(default)]
    pub output_tokens_details: Option<OutputTokensDetails>,
}

/// Input token usage details.
#[derive(Debug, Clone, Deserialize)]
pub struct InputTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<i64>,
}

/// Output token usage details.
#[derive(Debug, Clone, Deserialize)]
pub struct OutputTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: Option<i64>,
}

/// Response from `POST /responses`.
#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created_at: f64,
    pub model: String,
    pub output: Vec<ResponseOutputItem>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub error: Option<ResponseError>,
    #[serde(default)]
    pub incomplete_details: Option<IncompleteDetails>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub top_p: Option<f64>,
    #[serde(default)]
    pub max_output_tokens: Option<i64>,
    #[serde(default)]
    pub previous_response_id: Option<String>,
    #[serde(default)]
    pub usage: Option<ResponseUsage>,
    #[serde(default)]
    pub tools: Option<Vec<ResponseTool>>,
    #[serde(default)]
    pub tool_choice: Option<ResponseToolChoice>,
    #[serde(default)]
    pub parallel_tool_calls: Option<bool>,
    #[serde(default)]
    pub truncation: Option<String>,
    #[serde(default)]
    pub reasoning: Option<Reasoning>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub text: Option<ResponseTextConfig>,
    #[serde(default)]
    pub completed_at: Option<f64>,
    #[serde(default)]
    pub background: Option<bool>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub top_logprobs: Option<i64>,
    #[serde(default)]
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
}

// ── Streaming types ──

/// A streaming event from the Responses API.
/// Events are prefixed with `event:` in SSE and have a `data:` JSON payload.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseStreamEvent {
    /// Event type, e.g. "response.created", "response.output_text.delta".
    #[serde(rename = "type")]
    pub type_: String,
    /// The full payload (varies per event type).
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_response_create_request() {
        let mut req = ResponseCreateRequest::new("gpt-4o");
        req.input = Some("Hello".into());
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
        assert_eq!(json["input"], "Hello");
    }

    #[test]
    fn test_serialize_request_with_tools() {
        let mut req = ResponseCreateRequest::new("gpt-4o");
        req.input = Some("Search for Rust tutorials".into());
        req.tools = Some(vec![
            ResponseTool::WebSearch {
                search_context_size: Some("medium".into()),
                user_location: None,
            },
            ResponseTool::Function {
                name: "get_weather".into(),
                description: Some("Get weather".into()),
                parameters: Some(serde_json::json!({"type": "object"})),
                strict: Some(true),
            },
        ]);
        req.reasoning = Some(Reasoning {
            effort: Some("high".into()),
            summary: Some("auto".into()),
        });
        req.truncation = Some("auto".into());
        req.include = Some(vec!["file_search_call.results".into()]);

        let json = serde_json::to_value(&req).unwrap();
        let tools = json["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0]["type"], "web_search");
        assert_eq!(tools[1]["type"], "function");
        assert_eq!(tools[1]["name"], "get_weather");
        assert_eq!(tools[1]["strict"], true);
        assert_eq!(json["reasoning"]["effort"], "high");
        assert_eq!(json["truncation"], "auto");
    }

    #[test]
    fn test_serialize_request_with_mcp_tool() {
        let mut req = ResponseCreateRequest::new("gpt-4o");
        req.input = Some("Hello".into());
        req.tools = Some(vec![ResponseTool::Mcp {
            server_label: "my-server".into(),
            server_url: Some("https://example.com/mcp".into()),
            allowed_tools: None,
            require_approval: Some(serde_json::json!("never")),
        }]);

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["tools"][0]["type"], "mcp");
        assert_eq!(json["tools"][0]["server_label"], "my-server");
    }

    #[test]
    fn test_serialize_tool_choice() {
        let mode = ResponseToolChoice::Mode("auto".into());
        let json = serde_json::to_value(&mode).unwrap();
        assert_eq!(json, "auto");

        let named = ResponseToolChoice::Named {
            type_: "function".into(),
            function: ResponseToolChoiceFunction {
                name: "get_weather".into(),
            },
        };
        let json = serde_json::to_value(&named).unwrap();
        assert_eq!(json["type"], "function");
        assert_eq!(json["function"]["name"], "get_weather");
    }

    #[test]
    fn test_serialize_text_format() {
        let fmt = ResponseTextFormat::JsonSchema {
            name: "math_result".into(),
            description: None,
            schema: Some(
                serde_json::json!({"type": "object", "properties": {"answer": {"type": "number"}}}),
            ),
            strict: Some(true),
        };
        let json = serde_json::to_value(&fmt).unwrap();
        assert_eq!(json["type"], "json_schema");
        assert_eq!(json["name"], "math_result");
        assert_eq!(json["strict"], true);

        let text = ResponseTextFormat::Text;
        let json = serde_json::to_value(&text).unwrap();
        assert_eq!(json["type"], "text");
    }

    #[test]
    fn test_deserialize_response() {
        let json = r#"{
            "id": "resp-abc123",
            "object": "response",
            "created_at": 1677610602.0,
            "model": "gpt-4o",
            "output": [{
                "type": "message",
                "id": "msg-abc123",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "Hello! How can I help?",
                    "annotations": []
                }]
            }],
            "status": "completed",
            "usage": {
                "input_tokens": 10,
                "output_tokens": 6,
                "total_tokens": 16
            }
        }"#;

        let resp: Response = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "resp-abc123");
        assert_eq!(resp.output.len(), 1);
        assert_eq!(resp.output_text(), "Hello! How can I help?");
        let usage = resp.usage.as_ref().unwrap();
        assert_eq!(usage.total_tokens, Some(16));
    }

    #[test]
    fn test_deserialize_full_response() {
        let json = r#"{
            "id": "resp-abc123",
            "object": "response",
            "created_at": 1677610602.0,
            "model": "o3",
            "output": [{
                "type": "message",
                "id": "msg-abc123",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "Result",
                    "annotations": []
                }]
            }],
            "status": "completed",
            "service_tier": "default",
            "truncation": "auto",
            "reasoning": {"effort": "high", "summary": "auto"},
            "parallel_tool_calls": true,
            "max_output_tokens": 4096,
            "completed_at": 1677610605.0,
            "tools": [{"type": "web_search"}],
            "tool_choice": "auto",
            "instructions": "Be helpful",
            "text": {"format": {"type": "text"}},
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50,
                "total_tokens": 150,
                "input_tokens_details": {"cached_tokens": 20},
                "output_tokens_details": {"reasoning_tokens": 30}
            }
        }"#;

        let resp: Response = serde_json::from_str(json).unwrap();
        assert_eq!(resp.service_tier, Some("default".into()));
        assert_eq!(resp.truncation, Some("auto".into()));
        let reasoning = resp.reasoning.as_ref().unwrap();
        assert_eq!(reasoning.effort, Some("high".into()));
        assert_eq!(resp.parallel_tool_calls, Some(true));
        assert_eq!(resp.completed_at, Some(1677610605.0));
        assert_eq!(resp.instructions, Some("Be helpful".into()));
        // tool_choice echoed back as "auto"
        assert!(resp.tool_choice.is_some());
        // text config echoed back
        let text = resp.text.as_ref().unwrap();
        assert!(text.format.is_some());
        let usage = resp.usage.as_ref().unwrap();
        let input_details = usage.input_tokens_details.as_ref().unwrap();
        assert_eq!(input_details.cached_tokens, Some(20));
        let output_details = usage.output_tokens_details.as_ref().unwrap();
        assert_eq!(output_details.reasoning_tokens, Some(30));
    }

    #[test]
    fn test_deserialize_response_with_error() {
        let json = r#"{
            "id": "resp-err",
            "object": "response",
            "created_at": 1677610602.0,
            "model": "gpt-4o",
            "output": [],
            "status": "failed",
            "error": {
                "code": "server_error",
                "message": "Internal server error"
            },
            "incomplete_details": {
                "reason": "content_filter"
            }
        }"#;

        let resp: Response = serde_json::from_str(json).unwrap();
        let err = resp.error.as_ref().unwrap();
        assert_eq!(err.code, "server_error");
        assert_eq!(err.message, "Internal server error");
        let details = resp.incomplete_details.as_ref().unwrap();
        assert_eq!(details.reason, Some("content_filter".into()));
    }

    #[test]
    fn test_deserialize_response_with_annotations() {
        let json = r#"{
            "id": "resp-ann",
            "object": "response",
            "created_at": 1677610602.0,
            "model": "gpt-4o",
            "output": [{
                "type": "message",
                "id": "msg-1",
                "role": "assistant",
                "status": "completed",
                "content": [{
                    "type": "output_text",
                    "text": "According to [1]...",
                    "annotations": [{
                        "type": "url_citation",
                        "start_index": 14,
                        "end_index": 17,
                        "url": "https://example.com",
                        "title": "Example"
                    }]
                }]
            }],
            "status": "completed"
        }"#;

        let resp: Response = serde_json::from_str(json).unwrap();
        let content = resp.output[0].content.as_ref().unwrap();
        let annotations = content[0].annotations.as_ref().unwrap();
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].type_, "url_citation");
        assert_eq!(annotations[0].url, Some("https://example.com".into()));
        assert_eq!(annotations[0].start_index, Some(14));
    }

    #[test]
    fn test_deserialize_stream_event() {
        let json = r#"{
            "type": "response.output_text.delta",
            "delta": "Hello",
            "output_index": 0,
            "content_index": 0
        }"#;
        let event: ResponseStreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.type_, "response.output_text.delta");
        assert_eq!(event.data["delta"], "Hello");
    }

    #[test]
    fn test_builder_pattern() {
        let req = ResponseCreateRequest::new("o3")
            .input("Explain quantum computing")
            .instructions("Be concise")
            .temperature(0.5)
            .max_output_tokens(2048)
            .reasoning(Reasoning {
                effort: Some("high".into()),
                summary: Some("concise".into()),
            })
            .truncation("auto")
            .store(true)
            .tool_choice(ResponseToolChoice::Mode("auto".into()))
            .previous_response_id("resp-prev");

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "o3");
        assert_eq!(json["input"], "Explain quantum computing");
        assert_eq!(json["instructions"], "Be concise");
        assert_eq!(json["temperature"], 0.5);
        assert_eq!(json["max_output_tokens"], 2048);
        assert_eq!(json["reasoning"]["effort"], "high");
        assert_eq!(json["reasoning"]["summary"], "concise");
        assert_eq!(json["truncation"], "auto");
        assert_eq!(json["store"], true);
        assert_eq!(json["tool_choice"], "auto");
        assert_eq!(json["previous_response_id"], "resp-prev");
    }
}
