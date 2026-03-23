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

/// Enforce OpenAI's strict JSON schema rules recursively.
fn ensure_strict(value: &mut serde_json::Value) {
    if let serde_json::Value::Object(map) = value {
        // Add additionalProperties: false to all objects
        if map.get("type").and_then(|t| t.as_str()) == Some("object") {
            map.entry("additionalProperties")
                .or_insert(serde_json::Value::Bool(false));

            // Make all properties required
            if let Some(props) = map.get("properties").and_then(|p| p.as_object()) {
                let keys: Vec<String> = props.keys().cloned().collect();
                map.insert("required".to_string(), serde_json::json!(keys));
            }
        }

        // Remove null defaults (not meaningful in strict mode)
        if map.get("default") == Some(&serde_json::Value::Null) {
            map.remove("default");
        }

        // Recurse into all values
        for v in map.values_mut() {
            ensure_strict(v);
        }
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
}
