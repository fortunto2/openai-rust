// Realtime types — re-exported from openai-types.

pub use openai_types::realtime::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_session_create_request() {
        let req = SessionCreateRequest::new()
            .model("gpt-4o-realtime-preview")
            .voice("alloy")
            .instructions("You are a helpful assistant.")
            .modalities(vec!["text".into(), "audio".into()]);

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "gpt-4o-realtime-preview");
        assert_eq!(json["voice"], "alloy");
        assert_eq!(json["instructions"], "You are a helpful assistant.");
        assert_eq!(json["modalities"], serde_json::json!(["text", "audio"]));
    }

    #[test]
    fn test_serialize_request_with_tools() {
        let mut req = SessionCreateRequest::new().model("gpt-4o-realtime-preview");
        req.tools = Some(vec![RealtimeTool {
            type_: "function".into(),
            name: "get_weather".into(),
            description: Some("Get weather for a location".into()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            })),
        }]);
        req.turn_detection = Some(TurnDetection {
            type_: TurnDetectionType::ServerVad,
            threshold: Some(0.5),
            prefix_padding_ms: Some(300),
            silence_duration_ms: Some(500),
            create_response: Some(true),
            interrupt_response: Some(true),
            eagerness: None,
        });

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["tools"][0]["name"], "get_weather");
        assert_eq!(json["turn_detection"]["type"], "server_vad");
        assert_eq!(json["turn_detection"]["threshold"], 0.5);
    }

    #[test]
    fn test_deserialize_session_create_response() {
        let json = r#"{
            "client_secret": {
                "value": "ek-abc123xyz",
                "expires_at": 1700000000
            },
            "model": "gpt-4o-realtime-preview",
            "voice": "alloy",
            "modalities": ["text", "audio"],
            "temperature": 0.8,
            "input_audio_format": "pcm16",
            "output_audio_format": "pcm16",
            "tools": [{
                "type": "function",
                "name": "get_weather",
                "description": "Get weather",
                "parameters": {"type": "object"}
            }],
            "turn_detection": {
                "type": "server_vad",
                "threshold": 0.5,
                "prefix_padding_ms": 300,
                "silence_duration_ms": 500
            }
        }"#;

        let resp: SessionCreateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.client_secret.value, "ek-abc123xyz");
        assert_eq!(resp.client_secret.expires_at, 1700000000);
        assert_eq!(resp.model, Some("gpt-4o-realtime-preview".into()));
        assert_eq!(resp.voice, Some("alloy".into()));
        assert_eq!(resp.temperature, Some(0.8));
        let tools = resp.tools.unwrap();
        assert_eq!(tools[0].name, "get_weather");
        let turn = resp.turn_detection.unwrap();
        assert_eq!(turn.type_, TurnDetectionType::ServerVad);
        assert_eq!(turn.threshold, Some(0.5));
    }

    #[test]
    fn test_serialize_transcription_session_request() {
        let req = TranscriptionSessionCreateRequest::new()
            .input_audio_format(RealtimeAudioFormat::Pcm16)
            .transcription("gpt-4o-transcribe", "en")
            .turn_detection(TurnDetection {
                type_: TurnDetectionType::ServerVad,
                threshold: Some(0.5),
                prefix_padding_ms: Some(300),
                silence_duration_ms: Some(500),
                create_response: None,
                interrupt_response: None,
                eagerness: None,
            })
            .noise_reduction(NoiseReductionType::NearField);

        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input_audio_format"], "pcm16");
        assert_eq!(
            json["input_audio_transcription"]["model"],
            "gpt-4o-transcribe"
        );
        assert_eq!(json["input_audio_transcription"]["language"], "en");
        assert_eq!(json["turn_detection"]["type"], "server_vad");
        assert_eq!(json["input_audio_noise_reduction"]["type"], "near_field");
    }
}
