// Moderation types — re-exported from openai-types.
pub use openai_types::moderation::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_moderation_request() {
        let req = ModerationRequest::new("I want to harm someone");
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"], "I want to harm someone");
        assert!(json.get("model").is_none());
    }

    #[test]
    fn test_deserialize_moderation_response() {
        let json = r#"{
            "id": "modr-abc123",
            "model": "text-moderation-007",
            "results": [{
                "flagged": true,
                "categories": {
                    "harassment": true,
                    "harassment/threatening": false,
                    "hate": false,
                    "hate/threatening": false,
                    "self-harm": false,
                    "self-harm/instructions": false,
                    "self-harm/intent": false,
                    "sexual": false,
                    "sexual/minors": false,
                    "violence": true,
                    "violence/graphic": false
                },
                "category_scores": {
                    "harassment": 0.85,
                    "harassment/threatening": 0.02,
                    "hate": 0.001,
                    "hate/threatening": 0.0001,
                    "self-harm": 0.0001,
                    "self-harm/instructions": 0.0001,
                    "self-harm/intent": 0.0001,
                    "sexual": 0.0001,
                    "sexual/minors": 0.0001,
                    "violence": 0.75,
                    "violence/graphic": 0.001
                }
            }]
        }"#;

        let resp: ModerationCreateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "modr-abc123");
        assert_eq!(resp.results.len(), 1);
        assert!(resp.results[0].flagged);
        assert!(resp.results[0].categories.harassment);
        assert!(resp.results[0].categories.violence);
        assert!(resp.results[0].category_scores.harassment > 0.5);
    }
}
