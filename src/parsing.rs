// Structured Outputs — parse chat completions into typed Rust structs.
//
// Mirrors Python SDK's `client.chat.completions.parse(response_format=Model)`.
//
// Requires the `structured` feature (enables `schemars` dependency).

use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::error::OpenAIError;
use crate::types::chat::{
    ChatCompletionChoice, ChatCompletionMessage, ChatCompletionResponse,
    JsonSchema as ApiJsonSchema, ResponseFormat,
};
use crate::types::common::FinishReason;

/// A chat completion with the first choice's content parsed into `T`.
///
/// Wraps the raw [`ChatCompletionResponse`] and adds a `parsed` field.
///
/// ```ignore
/// use schemars::JsonSchema;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, JsonSchema)]
/// struct MathResponse {
///     steps: Vec<Step>,
///     final_answer: String,
/// }
///
/// #[derive(Deserialize, JsonSchema)]
/// struct Step {
///     explanation: String,
///     output: String,
/// }
///
/// let result = client.chat().completions()
///     .parse::<MathResponse>(request)
///     .await?;
///
/// println!("{}", result.parsed.unwrap().final_answer);
/// ```
#[derive(Debug, Clone)]
pub struct ParsedChatCompletion<T> {
    /// The raw API response.
    pub response: ChatCompletionResponse,
    /// The deserialized content from the first choice, or `None` if the
    /// model refused or the content was empty.
    pub parsed: Option<T>,
}

impl<T> std::ops::Deref for ParsedChatCompletion<T> {
    type Target = ChatCompletionResponse;
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Generate the `response_format` parameter for structured output from a Rust type.
///
/// Applies OpenAI's strict schema rules:
/// - `additionalProperties: false` on all objects
/// - All properties marked as `required`
pub fn response_format_from_type<T: JsonSchema>() -> ResponseFormat {
    let schema = schemars::schema_for!(T);
    let mut value = serde_json::to_value(schema).unwrap_or_default();
    ensure_strict(&mut value);

    ResponseFormat::JsonSchema {
        json_schema: ApiJsonSchema {
            name: std::any::type_name::<T>()
                .rsplit("::")
                .next()
                .unwrap_or("Response")
                .to_string(),
            description: None,
            schema: Some(value),
            strict: Some(true),
        },
    }
}

/// Generate a function tool definition from a Rust type.
///
/// ```ignore
/// use openai_oxide::parsing::tool_from_type;
///
/// #[derive(Deserialize, JsonSchema)]
/// /// Get current weather for a location.
/// struct GetWeather {
///     location: String,
///     unit: Option<String>,
/// }
///
/// let tool = tool_from_type::<GetWeather>("get_weather", "Get current weather");
/// ```
pub fn tool_from_type<T: JsonSchema>(
    name: impl Into<String>,
    description: impl Into<String>,
) -> crate::types::chat::Tool {
    let schema = schemars::schema_for!(T);
    let mut value = serde_json::to_value(schema).unwrap_or_default();
    ensure_strict(&mut value);

    crate::types::chat::Tool::function(name, description, value)
}

/// Generate a strict Responses API function tool from a Rust type.
///
/// Like [`tool_from_type`] but returns [`ResponseTool`] for the Responses API.
///
/// ```ignore
/// use openai_oxide::parsing::response_tool_from_type;
///
/// #[derive(Deserialize, JsonSchema)]
/// struct SearchArgs { query: String, limit: Option<u32> }
///
/// let tool = response_tool_from_type::<SearchArgs>("search", "Search items");
/// ```
pub fn response_tool_from_type<T: JsonSchema>(
    name: impl Into<String>,
    description: impl Into<String>,
) -> crate::types::responses::ResponseTool {
    let schema = schemars::schema_for!(T);
    let mut value = serde_json::to_value(schema).unwrap_or_default();
    ensure_strict(&mut value);

    crate::types::responses::ResponseTool::Function {
        name: name.into(),
        description: Some(description.into()),
        parameters: Some(value),
        strict: Some(true),
    }
}

/// Parse a chat completion response, extracting the content from the first choice.
///
/// Returns an error if:
/// - `finish_reason` is `length` (response was truncated)
/// - `finish_reason` is `content_filter`
/// - Content is present but fails to deserialize
pub fn parse_completion<T: DeserializeOwned>(
    response: ChatCompletionResponse,
) -> Result<ParsedChatCompletion<T>, OpenAIError> {
    let choice = response.choices.first();

    if let Some(choice) = choice {
        check_finish_reason(choice)?;

        let parsed = parse_message_content::<T>(&choice.message)?;

        Ok(ParsedChatCompletion { response, parsed })
    } else {
        Ok(ParsedChatCompletion {
            response,
            parsed: None,
        })
    }
}

/// Check finish_reason for errors.
fn check_finish_reason(choice: &ChatCompletionChoice) -> Result<(), OpenAIError> {
    match choice.finish_reason {
        FinishReason::Length => Err(OpenAIError::InvalidArgument(
            "response was truncated (finish_reason=length) — increase max_completion_tokens".into(),
        )),
        FinishReason::ContentFilter => Err(OpenAIError::InvalidArgument(
            "response was filtered (finish_reason=content_filter)".into(),
        )),
        _ => Ok(()),
    }
}

/// Parse the message content into T, returning None if refusal or empty.
fn parse_message_content<T: DeserializeOwned>(
    message: &ChatCompletionMessage,
) -> Result<Option<T>, OpenAIError> {
    // Don't parse if the model refused
    if message.refusal.is_some() {
        return Ok(None);
    }

    match &message.content {
        Some(content) if !content.is_empty() => {
            let parsed: T = serde_json::from_str(content)?;
            Ok(Some(parsed))
        }
        _ => Ok(None),
    }
}

// ── Responses API structured output ──

/// A Response with its text output parsed into `T`.
#[derive(Debug, Clone)]
pub struct ParsedResponse<T> {
    /// The raw API response.
    pub response: crate::types::responses::Response,
    /// The deserialized text output, or `None` if empty/failed.
    pub parsed: Option<T>,
}

impl<T> std::ops::Deref for ParsedResponse<T> {
    type Target = crate::types::responses::Response;
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

/// Generate the `text.format` parameter for Responses API structured output.
pub fn text_format_from_type<T: JsonSchema>() -> crate::types::responses::ResponseTextFormat {
    let schema = schemars::schema_for!(T);
    let mut value = serde_json::to_value(schema).unwrap_or_default();
    ensure_strict(&mut value);

    crate::types::responses::ResponseTextFormat::JsonSchema {
        name: std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap_or("Response")
            .to_string(),
        description: None,
        schema: Some(value),
        strict: Some(true),
    }
}

/// Parse a Responses API response, extracting `output_text()` into `T`.
pub fn parse_response<T: DeserializeOwned>(
    response: crate::types::responses::Response,
) -> Result<ParsedResponse<T>, OpenAIError> {
    if response.status.as_deref() == Some("failed") {
        let msg = response
            .error
            .as_ref()
            .map(|e| e.message.clone())
            .unwrap_or_else(|| "response failed".into());
        return Err(OpenAIError::InvalidArgument(msg));
    }

    let text = response.output_text();
    let parsed = if text.is_empty() {
        None
    } else {
        Some(serde_json::from_str::<T>(&text)?)
    };

    Ok(ParsedResponse { response, parsed })
}

/// Make a JSON Schema compatible with OpenAI strict mode.
///
/// Mirrors the Python SDK's `_ensure_strict_json_schema()` from `openai/lib/_pydantic.py`.
///
/// OpenAI `strict: true` requires:
/// 1. `additionalProperties: false` on every object
/// 2. All properties listed in `required`
/// 3. Optional fields use `anyOf: [{type}, {type: null}]` (not `nullable: true`)
/// 4. `allOf` with 1 item → inline; multi-item → recurse
/// 5. `oneOf` → `anyOf`
/// 6. Recurse into `properties`, `items`, `anyOf`, `allOf`, `$defs`, `definitions`
///
/// See: <https://developers.openai.com/api/docs/guides/structured-outputs>
pub fn ensure_strict(value: &mut serde_json::Value) {
    if let serde_json::Value::Object(map) = value {
        // Recurse into $defs / definitions first (like Python SDK)
        for key in ["$defs", "definitions"] {
            if let Some(defs) = map.get_mut(key).and_then(|v| v.as_object_mut()) {
                for def in defs.values_mut() {
                    ensure_strict(def);
                }
            }
        }

        let is_object = map.get("type").and_then(|t| t.as_str()) == Some("object");

        if is_object {
            map.entry("additionalProperties")
                .or_insert(serde_json::Value::Bool(false));

            // Convert nullable properties: "nullable":true + "type":"T"
            // → {"anyOf": [{"type":"T"}, {"type":"null"}]}
            // schemars 0.8 uses "nullable" keyword; OpenAI requires anyOf pattern.
            if let Some(props) = map.get_mut("properties").and_then(|v| v.as_object_mut()) {
                for (_key, prop) in props.iter_mut() {
                    if let Some(prop_obj) = prop.as_object_mut() {
                        if prop_obj.remove("nullable").and_then(|v| v.as_bool()) == Some(true) {
                            if let Some(type_val) = prop_obj.remove("type") {
                                let desc = prop_obj.remove("description");
                                let mut wrapper = serde_json::Map::new();
                                wrapper.insert(
                                    "anyOf".into(),
                                    serde_json::json!([{"type": type_val}, {"type": "null"}]),
                                );
                                if let Some(d) = desc {
                                    wrapper.insert("description".into(), d);
                                }
                                *prop = serde_json::Value::Object(wrapper);
                            }
                        }
                    }
                }
            }

            // All properties must be required
            if let Some(props) = map.get("properties").and_then(|p| p.as_object()) {
                let keys: Vec<String> = props.keys().cloned().collect();
                map.insert("required".to_string(), serde_json::json!(keys));
            }

            // Recurse into each property
            if let Some(props) = map.get_mut("properties").and_then(|v| v.as_object_mut()) {
                for prop in props.values_mut() {
                    ensure_strict(prop);
                }
            }
        }

        // Recurse into array items
        if let Some(items) = map.get_mut("items") {
            ensure_strict(items);
        }

        // Remove null defaults (not meaningful in strict mode)
        if map.get("default") == Some(&serde_json::Value::Null) {
            map.remove("default");
        }

        // oneOf → anyOf (OpenAI strict doesn't support oneOf)
        if let Some(one_of) = map.remove("oneOf") {
            map.insert("anyOf".into(), one_of);
        }

        // anyOf: recurse into variants
        if let Some(any_of) = map.get_mut("anyOf").and_then(|v| v.as_array_mut()) {
            for variant in any_of.iter_mut() {
                ensure_strict(variant);
            }
        }

        // allOf: inline single-item, recurse multi-item (matches Python SDK)
        if let Some(all_of) = map.remove("allOf") {
            if let Some(arr) = all_of.as_array() {
                if arr.len() == 1 {
                    // Inline single allOf entry
                    if let Some(inner) = arr[0].as_object() {
                        for (k, v) in inner {
                            map.entry(k.clone()).or_insert(v.clone());
                        }
                    }
                    // Re-run strict on this node (inlined content may need processing)
                    ensure_strict(value);
                    return;
                } else {
                    // Multi allOf: keep and recurse
                    let mut all_of = all_of;
                    if let Some(arr) = all_of.as_array_mut() {
                        for entry in arr.iter_mut() {
                            ensure_strict(entry);
                        }
                    }
                    map.insert("allOf".into(), all_of);
                }
            }
        }

        // Remove top-level $schema (not needed in tool schemas)
        map.remove("$schema");
    } else if let serde_json::Value::Array(arr) = value {
        for v in arr {
            ensure_strict(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct MathStep {
        explanation: String,
        output: String,
    }

    #[derive(Debug, Deserialize, JsonSchema, PartialEq)]
    struct MathResponse {
        steps: Vec<MathStep>,
        final_answer: String,
    }

    #[test]
    fn test_response_format_generation() {
        let fmt = response_format_from_type::<MathResponse>();
        match fmt {
            ResponseFormat::JsonSchema { json_schema } => {
                assert_eq!(json_schema.name, "MathResponse");
                assert_eq!(json_schema.strict, Some(true));
                let schema = json_schema.schema.unwrap();
                assert_eq!(schema["type"], "object");
                assert_eq!(schema["additionalProperties"], false);
                // Check required fields
                let required = schema["required"].as_array().unwrap();
                assert!(required.contains(&serde_json::json!("steps")));
                assert!(required.contains(&serde_json::json!("final_answer")));
            }
            _ => panic!("expected JsonSchema variant"),
        }
    }

    #[test]
    fn test_parse_completion_success() {
        let response = ChatCompletionResponse {
            id: "test".into(),
            choices: vec![ChatCompletionChoice {
                finish_reason: FinishReason::Stop,
                index: 0,
                message: ChatCompletionMessage {
                    role: crate::types::common::Role::Assistant,
                    content: Some(
                        r#"{"steps":[{"explanation":"add","output":"4"}],"final_answer":"4"}"#
                            .into(),
                    ),
                    refusal: None,
                    tool_calls: None,
                    annotations: None,
                },
                logprobs: None,
            }],
            created: 0,
            model: "gpt-4o".into(),
            object: "chat.completion".into(),
            service_tier: None,
            system_fingerprint: None,
            usage: None,
        };

        let parsed: ParsedChatCompletion<MathResponse> = parse_completion(response).unwrap();
        let math = parsed.parsed.unwrap();
        assert_eq!(math.final_answer, "4");
        assert_eq!(math.steps.len(), 1);
        assert_eq!(math.steps[0].explanation, "add");
    }

    #[test]
    fn test_parse_completion_length_error() {
        let response = ChatCompletionResponse {
            id: "test".into(),
            choices: vec![ChatCompletionChoice {
                finish_reason: FinishReason::Length,
                index: 0,
                message: ChatCompletionMessage {
                    role: crate::types::common::Role::Assistant,
                    content: Some("partial".into()),
                    refusal: None,
                    tool_calls: None,
                    annotations: None,
                },
                logprobs: None,
            }],
            created: 0,
            model: "gpt-4o".into(),
            object: "chat.completion".into(),
            service_tier: None,
            system_fingerprint: None,
            usage: None,
        };

        let result = parse_completion::<MathResponse>(response);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("truncated"));
    }

    #[test]
    fn test_parse_completion_refusal() {
        let response = ChatCompletionResponse {
            id: "test".into(),
            choices: vec![ChatCompletionChoice {
                finish_reason: FinishReason::Stop,
                index: 0,
                message: ChatCompletionMessage {
                    role: crate::types::common::Role::Assistant,
                    content: None,
                    refusal: Some("I can't help with that".into()),
                    tool_calls: None,
                    annotations: None,
                },
                logprobs: None,
            }],
            created: 0,
            model: "gpt-4o".into(),
            object: "chat.completion".into(),
            service_tier: None,
            system_fingerprint: None,
            usage: None,
        };

        let parsed: ParsedChatCompletion<MathResponse> = parse_completion(response).unwrap();
        assert!(parsed.parsed.is_none());
        assert!(parsed.response.choices[0].message.refusal.is_some());
    }

    #[test]
    fn test_tool_from_type() {
        #[derive(JsonSchema)]
        struct GetWeather {
            location: String,
        }

        let tool = tool_from_type::<GetWeather>("get_weather", "Get weather");
        assert_eq!(tool.function.name, "get_weather");
        assert_eq!(tool.function.strict, Some(true));
        let params = tool.function.parameters.unwrap();
        assert_eq!(params["additionalProperties"], false);
    }

    #[test]
    fn test_ensure_strict_nested() {
        let mut schema = serde_json::json!({
            "type": "object",
            "properties": {
                "inner": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"}
                    }
                }
            }
        });

        ensure_strict(&mut schema);

        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["properties"]["inner"]["additionalProperties"], false);
        // Both levels have required
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("inner"))
        );
        assert!(
            schema["properties"]["inner"]["required"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("name"))
        );
    }

    #[test]
    fn test_text_format_generation() {
        let fmt = text_format_from_type::<MathResponse>();
        match fmt {
            crate::types::responses::ResponseTextFormat::JsonSchema {
                name,
                strict,
                schema,
                ..
            } => {
                assert_eq!(name, "MathResponse");
                assert_eq!(strict, Some(true));
                let schema = schema.unwrap();
                assert_eq!(schema["additionalProperties"], false);
            }
            _ => panic!("expected JsonSchema variant"),
        }
    }

    #[test]
    fn test_parse_response_success() {
        let json = r#"{
            "id": "resp-1", "object": "response", "created_at": 1.0,
            "model": "gpt-4o",
            "output": [{"type": "message", "id": "msg-1", "role": "assistant",
                "status": "completed",
                "content": [{"type": "output_text",
                    "text": "{\"steps\":[],\"final_answer\":\"42\"}"}]
            }],
            "status": "completed"
        }"#;
        let response: crate::types::responses::Response = serde_json::from_str(json).unwrap();
        let parsed: ParsedResponse<MathResponse> = parse_response(response).unwrap();
        assert_eq!(parsed.parsed.unwrap().final_answer, "42");
    }

    #[test]
    fn test_parse_response_failed() {
        let json = r#"{
            "id": "resp-err", "object": "response", "created_at": 1.0,
            "model": "gpt-4o", "output": [], "status": "failed",
            "error": {"code": "server_error", "message": "something broke"}
        }"#;
        let response: crate::types::responses::Response = serde_json::from_str(json).unwrap();
        let result = parse_response::<MathResponse>(response);
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("something broke"));
    }
}
