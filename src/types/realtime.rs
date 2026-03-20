// Realtime API types — mirrors openai-python types/beta/realtime/

use serde::{Deserialize, Serialize};

/// Audio format for realtime API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RealtimeAudioFormat {
    Pcm16,
    G711Ulaw,
    G711Alaw,
}

/// Turn detection type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TurnDetectionType {
    ServerVad,
    SemanticVad,
}

/// Eagerness level for semantic VAD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Eagerness {
    Low,
    Medium,
    High,
    Auto,
}

// ── Request types ──

/// Request body for `POST /realtime/sessions`.
#[derive(Debug, Clone, Serialize)]
pub struct SessionCreateRequest {
    /// The Realtime model to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// The voice the model uses to respond.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,

    /// System instructions for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// The format of input audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<RealtimeAudioFormat>,

    /// The format of output audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<RealtimeAudioFormat>,

    /// Modalities: ["text"], ["audio"], or ["text", "audio"].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,

    /// Sampling temperature (0.6–1.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Maximum output tokens (1–4096 or "inf").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<crate::types::common::MaxResponseTokens>,

    /// Tools (functions) available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<RealtimeTool>>,

    /// How the model chooses tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,

    /// Turn detection configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,

    /// Input audio transcription configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<InputAudioTranscription>,

    /// Speed of spoken response (0.25–1.5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

impl SessionCreateRequest {
    pub fn new() -> Self {
        Self {
            model: None,
            voice: None,
            instructions: None,
            input_audio_format: None,
            output_audio_format: None,
            modalities: None,
            temperature: None,
            max_response_output_tokens: None,
            tools: None,
            tool_choice: None,
            turn_detection: None,
            input_audio_transcription: None,
            speed: None,
        }
    }

    /// Set the model.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the voice.
    pub fn voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = Some(voice.into());
        self
    }

    /// Set instructions.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set modalities.
    pub fn modalities(mut self, modalities: Vec<String>) -> Self {
        self.modalities = Some(modalities);
        self
    }
}

impl Default for SessionCreateRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// A function tool for the Realtime API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeTool {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Turn detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnDetection {
    /// Turn detection type.
    #[serde(rename = "type")]
    pub type_: TurnDetectionType,
    /// VAD activation threshold (0.0–1.0, server_vad only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Audio included before speech starts (ms, server_vad only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_padding_ms: Option<i64>,
    /// Silence duration to detect end (ms, server_vad only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_duration_ms: Option<i64>,
    /// Whether to auto-generate response on VAD stop.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_response: Option<bool>,
    /// Whether to auto-interrupt on VAD start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt_response: Option<bool>,
    /// Eagerness level for semantic VAD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eagerness: Option<Eagerness>,
}

/// Input audio transcription configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioTranscription {
    /// Transcription model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Language code (ISO-639-1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

// ── Response types ──

/// Ephemeral client secret returned by session creation.
#[derive(Debug, Clone, Deserialize)]
pub struct ClientSecret {
    /// The ephemeral API key value.
    pub value: String,
    /// Expiration timestamp.
    pub expires_at: i64,
}

/// Response from `POST /realtime/sessions`.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionCreateResponse {
    /// Ephemeral client secret for client-side authentication.
    pub client_secret: ClientSecret,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub voice: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub modalities: Option<Vec<String>>,
    #[serde(default)]
    pub temperature: Option<f64>,
    #[serde(default)]
    pub max_response_output_tokens: Option<crate::types::common::MaxResponseTokens>,
    #[serde(default)]
    pub input_audio_format: Option<RealtimeAudioFormat>,
    #[serde(default)]
    pub output_audio_format: Option<RealtimeAudioFormat>,
    #[serde(default)]
    pub tools: Option<Vec<RealtimeTool>>,
    #[serde(default)]
    pub tool_choice: Option<String>,
    #[serde(default)]
    pub turn_detection: Option<TurnDetection>,
    #[serde(default)]
    pub speed: Option<f64>,
}

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
}
