// Chat completion types — re-exported from openai-types.
// Manual overrides (request, response, streaming, tools) live in openai-types/src/chat/manual.rs
// Generated types (_gen.rs) supplement with verbose params, deleted responses, etc.

pub use openai_types::chat::*;

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
        assert_eq!(resp.choices[0].finish_reason, FinishReason::Stop);
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
        assert_eq!(resp.choices[0].finish_reason, FinishReason::ToolCalls);
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
                        detail: Some(ImageDetail::High),
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
        assert_eq!(chunk.choices[0].finish_reason, Some(FinishReason::Stop));
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
        .reasoning_effort(ReasoningEffort::High)
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
