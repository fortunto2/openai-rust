// Responses API types — canonical definitions live in openai-types/src/responses/.
// This module re-exports everything from openai-types and adds crate-local types.

use serde::{Deserialize, Serialize};

// Re-export all types from openai-types responses module.
// This includes: common (Role, ImageDetail, ReasoningEffort, ReasoningSummary, ServiceTier),
// create (Reasoning, ReasoningArgs, ResponseCreateRequest, CreateResponse, CreateResponseArgs,
//         ResponseTextConfig, ResponseTextFormat),
// input (ResponseInput, ResponseInputItem, EasyInputMessage, MessageType, EasyInputContent,
//        InputContent, InputTextContent, InputImageContent, InputItem, Item,
//        FunctionCallOutputItemParam, FunctionCallOutput, IncludeEnum, InputParam),
// output (ResponseOutputContent, ResponseOutputItem, OutputItem, FunctionToolCall,
//         ReasoningItem, SummaryPart, SummaryTextContent, ReasoningContent, FunctionCall),
// response (ResponseError, IncompleteDetails, ResponseAnnotation, InputTokensDetails,
//           OutputTokensDetails, InputTokenDetails, OutputTokenDetails, ResponseUsage, Response),
// streaming (all event types + ResponseStreamEvent),
// tools (FunctionTool, ResponseRankingOptions, ResponseTool, ResponseToolChoice,
//        ResponseToolChoiceFunction, ToolChoiceParam, ToolChoiceOptions, ToolChoiceFunction),
// + 281 generated types from _gen.rs.
pub use openai_types::responses::*;

// Re-export Role from common so `types::responses::Role` works.
// This shadows the openai_types::responses::common::Role (they're the same type
// since responses::common re-exports from shared).
pub use super::common::ReasoningEffort;
pub use super::common::Role;

// ── Crate-local types (depend on main crate types) ──

/// Tool definition for the Responses API (standalone enum).
///
/// Typed variant of `ResponseTool` for use in `CreateResponse` builder.
/// This type lives here rather than in openai-types because it references
/// `crate::types::chat::WebSearchUserLocation`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum Tool {
    /// Function tool.
    #[serde(rename = "function")]
    Function(FunctionTool),
    /// Web search tool.
    #[serde(rename = "web_search")]
    WebSearch {
        /// Search context size.
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        /// User location.
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<crate::types::chat::WebSearchUserLocation>,
    },
    /// File search tool.
    #[serde(rename = "file_search")]
    FileSearch {
        /// Vector store IDs.
        vector_store_ids: Vec<String>,
        /// Max results.
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<i64>,
        /// Ranking options.
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_options: Option<ResponseRankingOptions>,
    },
    /// Code interpreter tool.
    #[serde(rename = "code_interpreter")]
    CodeInterpreter {
        /// Container ID.
        #[serde(skip_serializing_if = "Option::is_none")]
        container: Option<String>,
    },
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
            effort: Some(ReasoningEffort::High),
            summary: Some(ReasoningSummary::Auto),
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
        assert_eq!(reasoning.effort, Some(ReasoningEffort::High));
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
            "content_index": 0,
            "item_id": "item_1",
            "sequence_number": 1
        }"#;
        let event: ResponseStreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type(), "response.output_text.delta");
        match event {
            ResponseStreamEvent::ResponseOutputTextDelta(evt) => {
                assert_eq!(evt.delta, "Hello");
                assert_eq!(evt.output_index, 0);
                assert_eq!(evt.content_index, 0);
            }
            other => panic!("expected ResponseOutputTextDelta, got: {other:?}"),
        }
    }

    #[test]
    fn test_deserialize_stream_event_completed() {
        let json = r#"{
            "type": "response.completed",
            "sequence_number": 5,
            "response": {
                "id": "resp-1",
                "object": "response",
                "created_at": 1.0,
                "model": "gpt-4o",
                "output": [],
                "status": "completed"
            }
        }"#;
        let event: ResponseStreamEvent = serde_json::from_str(json).unwrap();
        match event {
            ResponseStreamEvent::ResponseCompleted(evt) => {
                assert_eq!(evt.response.id, "resp-1");
                assert_eq!(evt.sequence_number, 5);
            }
            other => panic!("expected ResponseCompleted, got: {other:?}"),
        }
    }

    #[test]
    fn test_deserialize_stream_event_unknown_type() {
        let json = r#"{"type": "response.some_future_event", "foo": "bar"}"#;
        let event: ResponseStreamEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type(), "response.some_future_event");
        assert!(matches!(event, ResponseStreamEvent::Other(_)));
    }

    #[test]
    fn test_builder_pattern() {
        let req = ResponseCreateRequest::new("o3")
            .input("Explain quantum computing")
            .instructions("Be concise")
            .temperature(0.5)
            .max_output_tokens(2048)
            .reasoning(Reasoning {
                effort: Some(ReasoningEffort::High),
                summary: Some(ReasoningSummary::Concise),
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
