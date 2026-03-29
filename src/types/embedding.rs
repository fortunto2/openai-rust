// Embedding types — re-exported from openai-types.
pub use openai_types::embedding::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_embedding_request() {
        let req = EmbeddingCreateRequest::new("Hello world", "text-embedding-3-small");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"], "Hello world");
        assert_eq!(json["model"], "text-embedding-3-small");
    }

    #[test]
    fn test_serialize_embedding_request_with_array() {
        let req = EmbeddingCreateRequest::new(
            vec!["Hello".to_string(), "World".to_string()],
            "text-embedding-3-small",
        );
        let json = serde_json::to_value(&req).unwrap();
        let arr = json["input"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_deserialize_embedding_response() {
        let json = r#"{
            "object": "list",
            "data": [{
                "object": "embedding",
                "embedding": [0.1, 0.2, 0.3],
                "index": 0
            }],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;
        let resp: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].embedding.as_ref().unwrap().len(), 3);
    }
}
