// Manual: hand-crafted realtime types (session builders, enums, transcription).

use serde::{Deserialize, Serialize};

// Re-export shared types (canonical definition in shared/common.rs)
pub use crate::shared::MaxResponseTokens;

/// Audio format for realtime API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RealtimeAudioFormat {
    Pcm16,
    G711Ulaw,
    G711Alaw,
}

/// Turn detection type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TurnDetectionType {
    ServerVad,
    SemanticVad,
}

/// Eagerness level for semantic VAD.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Eagerness {
    Low,
    Medium,
    High,
    Auto,
}

/// Noise reduction type for input audio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum NoiseReductionType {
    NearField,
    FarField,
}

// -- Request types --

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

    /// Sampling temperature (0.6-1.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Maximum output tokens (1-4096 or "inf").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<MaxResponseTokens>,

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

    /// Speed of spoken response (0.25-1.5).
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
    /// VAD activation threshold (0.0-1.0, server_vad only).
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
    /// Transcription model (gpt-4o-transcribe, gpt-4o-mini-transcribe, whisper-1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Language code (ISO-639-1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Optional text to guide the model's style or continue a previous audio segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

/// Input audio noise reduction configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioNoiseReduction {
    #[serde(rename = "type")]
    pub type_: NoiseReductionType,
}

// -- Response types --

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
    pub max_response_output_tokens: Option<MaxResponseTokens>,
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

// -- Client secret config types --

/// Client secret configuration for request (controls token expiration).
#[derive(Debug, Clone, Serialize)]
pub struct ClientSecretConfig {
    /// Expiration configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<ClientSecretExpiresAt>,
}

/// Expiration anchor + duration for client secret.
#[derive(Debug, Clone, Serialize)]
pub struct ClientSecretExpiresAt {
    /// Anchor point -- only "created_at" is currently supported.
    pub anchor: String,
    /// Seconds from anchor to expiration (10-7200).
    pub seconds: i64,
}

// -- Transcription session types --

/// Request body for `POST /realtime/transcription_sessions`.
///
/// Creates a transcription-only Realtime session with an ephemeral token.
/// Unlike `/realtime/sessions`, this endpoint is specialized for STT
/// (no voice output, no response generation).
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionSessionCreateRequest {
    /// Configuration for the ephemeral token expiration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<ClientSecretConfig>,

    /// Items to include in transcription (e.g. "item.input_audio_transcription.logprobs").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// The format of input audio (pcm16, g711_ulaw, g711_alaw).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<RealtimeAudioFormat>,

    /// Configuration for input audio noise reduction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: Option<InputAudioNoiseReduction>,

    /// Configuration for input audio transcription (model + language + prompt).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<InputAudioTranscription>,

    /// Modalities: ["text"] or ["text", "audio"].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,

    /// Turn detection configuration (server_vad or semantic_vad).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
}

impl TranscriptionSessionCreateRequest {
    pub fn new() -> Self {
        Self {
            client_secret: None,
            include: None,
            input_audio_format: None,
            input_audio_noise_reduction: None,
            input_audio_transcription: None,
            modalities: None,
            turn_detection: None,
        }
    }

    pub fn input_audio_format(mut self, format: RealtimeAudioFormat) -> Self {
        self.input_audio_format = Some(format);
        self
    }

    pub fn transcription(mut self, model: impl Into<String>, language: impl Into<String>) -> Self {
        self.input_audio_transcription = Some(InputAudioTranscription {
            model: Some(model.into()),
            language: Some(language.into()),
            prompt: None,
        });
        self
    }

    pub fn turn_detection(mut self, td: TurnDetection) -> Self {
        self.turn_detection = Some(td);
        self
    }

    pub fn noise_reduction(mut self, type_: NoiseReductionType) -> Self {
        self.input_audio_noise_reduction = Some(InputAudioNoiseReduction { type_ });
        self
    }

    pub fn include(mut self, items: Vec<String>) -> Self {
        self.include = Some(items);
        self
    }

    pub fn modalities(mut self, modalities: Vec<String>) -> Self {
        self.modalities = Some(modalities);
        self
    }

    pub fn expires_in(mut self, seconds: i64) -> Self {
        self.client_secret = Some(ClientSecretConfig {
            expires_at: Some(ClientSecretExpiresAt {
                anchor: "created_at".into(),
                seconds,
            }),
        });
        self
    }
}

impl Default for TranscriptionSessionCreateRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from `POST /realtime/transcription_sessions`.
///
/// Simpler than `SessionCreateResponse` -- no model/voice/tools/instructions.
#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionSession {
    /// Ephemeral client secret for client-side authentication.
    pub client_secret: ClientSecret,
    /// The format of input audio.
    #[serde(default)]
    pub input_audio_format: Option<String>,
    /// Transcription configuration.
    #[serde(default)]
    pub input_audio_transcription: Option<InputAudioTranscription>,
    /// Modalities.
    #[serde(default)]
    pub modalities: Option<Vec<String>>,
    /// Turn detection configuration.
    #[serde(default)]
    pub turn_detection: Option<TurnDetection>,
}
