// Beta API types — re-exported from openai-types.
// Manual overrides (Assistants, Threads, Runs, Vector Stores) live in openai-types/src/beta/manual.rs
// Generated types (_gen.rs) supplement with streaming events, tool resources, etc.

pub use openai_types::beta::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_assistant_create() {
        let req = AssistantCreateRequest::new("gpt-4o");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o");
    }

    #[test]
    fn test_serialize_assistant_with_tools() {
        let mut req = AssistantCreateRequest::new("gpt-4o");
        req.tools = Some(vec![
            BetaTool::CodeInterpreter,
            BetaTool::FileSearch { file_search: None },
            BetaTool::Function {
                function: BetaFunctionDef {
                    name: "get_weather".into(),
                    description: Some("Get weather".into()),
                    parameters: Some(serde_json::json!({"type": "object"})),
                },
            },
        ]);
        let json = serde_json::to_value(&req).unwrap();
        let tools = json["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0]["type"], "code_interpreter");
        assert_eq!(tools[1]["type"], "file_search");
        assert_eq!(tools[2]["type"], "function");
        assert_eq!(tools[2]["function"]["name"], "get_weather");
    }

    #[test]
    fn test_deserialize_assistant() {
        let json = r#"{
            "id": "asst_abc123",
            "object": "assistant",
            "created_at": 1699009709,
            "model": "gpt-4o",
            "tools": [{"type": "code_interpreter"}]
        }"#;
        let asst: Assistant = serde_json::from_str(json).unwrap();
        assert_eq!(asst.id, "asst_abc123");
        assert_eq!(asst.tools.len(), 1);
        assert!(matches!(asst.tools[0], BetaTool::CodeInterpreter));
    }

    #[test]
    fn test_deserialize_assistant_with_function_tool() {
        let json = r#"{
            "id": "asst_abc123",
            "object": "assistant",
            "created_at": 1699009709,
            "model": "gpt-4o",
            "tools": [{
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get current weather",
                    "parameters": {"type": "object", "properties": {"city": {"type": "string"}}}
                }
            }]
        }"#;
        let asst: Assistant = serde_json::from_str(json).unwrap();
        match &asst.tools[0] {
            BetaTool::Function { function } => {
                assert_eq!(function.name, "get_weather");
            }
            _ => panic!("expected function tool"),
        }
    }

    #[test]
    fn test_deserialize_thread() {
        let json = r#"{
            "id": "thread_abc123",
            "object": "thread",
            "created_at": 1699012949
        }"#;
        let thread: Thread = serde_json::from_str(json).unwrap();
        assert_eq!(thread.id, "thread_abc123");
    }

    #[test]
    fn test_deserialize_run() {
        let json = r#"{
            "id": "run_abc123",
            "object": "thread.run",
            "created_at": 1699012949,
            "thread_id": "thread_abc123",
            "assistant_id": "asst_abc123",
            "status": "completed",
            "tools": []
        }"#;
        let run: Run = serde_json::from_str(json).unwrap();
        assert_eq!(run.status, RunStatus::Completed);
    }

    #[test]
    fn test_deserialize_run_with_tools() {
        let json = r#"{
            "id": "run_abc123",
            "object": "thread.run",
            "created_at": 1699012949,
            "thread_id": "thread_abc123",
            "assistant_id": "asst_abc123",
            "status": "completed",
            "tools": [
                {"type": "code_interpreter"},
                {"type": "file_search", "file_search": {"max_num_results": 10}}
            ]
        }"#;
        let run: Run = serde_json::from_str(json).unwrap();
        assert_eq!(run.tools.len(), 2);
        match &run.tools[1] {
            BetaTool::FileSearch { file_search } => {
                assert_eq!(file_search.as_ref().unwrap().max_num_results, Some(10));
            }
            _ => panic!("expected file_search tool"),
        }
    }

    #[test]
    fn test_deserialize_message_with_annotations() {
        let json = r#"{
            "id": "msg_abc123",
            "object": "thread.message",
            "created_at": 1699012949,
            "thread_id": "thread_abc123",
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": {
                    "value": "See file [1].",
                    "annotations": [{
                        "type": "file_citation",
                        "text": "[1]",
                        "start_index": 9,
                        "end_index": 12,
                        "file_citation": {
                            "file_id": "file-abc123",
                            "quote": "relevant text"
                        }
                    }]
                }
            }]
        }"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        let text = msg.content[0].text.as_ref().unwrap();
        assert_eq!(text.annotations.len(), 1);
        assert_eq!(text.annotations[0].type_, "file_citation");
        let citation = text.annotations[0].file_citation.as_ref().unwrap();
        assert_eq!(citation.file_id, "file-abc123");
    }

    #[test]
    fn test_deserialize_vector_store() {
        let json = r#"{
            "id": "vs_abc123",
            "object": "vector_store",
            "created_at": 1699012949,
            "name": "My Store",
            "status": "completed",
            "file_counts": {
                "in_progress": 0,
                "completed": 5,
                "failed": 0,
                "cancelled": 0,
                "total": 5
            }
        }"#;
        let vs: VectorStore = serde_json::from_str(json).unwrap();
        assert_eq!(vs.id, "vs_abc123");
        assert_eq!(vs.file_counts.unwrap().completed, 5);
    }
}
