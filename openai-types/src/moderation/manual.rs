// Hand-crafted moderation types for openai-types.
// These types mirror the original src/types/moderation.rs but are standalone (no crate:: imports).

use serde::{Deserialize, Serialize};

// -- Request types --

/// Input for moderations: a single string or array of strings.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ModerationInput {
    String(String),
    StringArray(Vec<String>),
}

impl From<&str> for ModerationInput {
    fn from(s: &str) -> Self {
        ModerationInput::String(s.to_string())
    }
}

impl From<String> for ModerationInput {
    fn from(s: String) -> Self {
        ModerationInput::String(s)
    }
}

impl From<Vec<String>> for ModerationInput {
    fn from(v: Vec<String>) -> Self {
        ModerationInput::StringArray(v)
    }
}

/// Request body for `POST /moderations`.
#[derive(Debug, Clone, Serialize)]
pub struct ModerationRequest {
    /// Input text to classify.
    pub input: ModerationInput,

    /// Model to use (e.g. "omni-moderation-latest").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl ModerationRequest {
    pub fn new(input: impl Into<ModerationInput>) -> Self {
        Self {
            input: input.into(),
            model: None,
        }
    }
}

// -- Response types --

/// Category flags for moderation results.
#[derive(Debug, Clone, Deserialize)]
pub struct Categories {
    pub harassment: bool,
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: bool,
    pub hate: bool,
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: bool,
    #[serde(default, rename = "illicit")]
    pub illicit: Option<bool>,
    #[serde(default, rename = "illicit/violent")]
    pub illicit_violent: Option<bool>,
    #[serde(rename = "self-harm")]
    pub self_harm: bool,
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: bool,
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: bool,
    pub sexual: bool,
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: bool,
    pub violence: bool,
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: bool,
}

/// Category scores for moderation results.
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryScores {
    pub harassment: f64,
    #[serde(rename = "harassment/threatening")]
    pub harassment_threatening: f64,
    pub hate: f64,
    #[serde(rename = "hate/threatening")]
    pub hate_threatening: f64,
    #[serde(default, rename = "illicit")]
    pub illicit: Option<f64>,
    #[serde(default, rename = "illicit/violent")]
    pub illicit_violent: Option<f64>,
    #[serde(rename = "self-harm")]
    pub self_harm: f64,
    #[serde(rename = "self-harm/instructions")]
    pub self_harm_instructions: f64,
    #[serde(rename = "self-harm/intent")]
    pub self_harm_intent: f64,
    pub sexual: f64,
    #[serde(rename = "sexual/minors")]
    pub sexual_minors: f64,
    pub violence: f64,
    #[serde(rename = "violence/graphic")]
    pub violence_graphic: f64,
}

/// A single moderation result.
#[derive(Debug, Clone, Deserialize)]
pub struct Moderation {
    pub flagged: bool,
    pub categories: Categories,
    pub category_scores: CategoryScores,
}

/// Response from `POST /moderations`.
#[derive(Debug, Clone, Deserialize)]
pub struct ModerationCreateResponse {
    pub id: String,
    pub model: String,
    pub results: Vec<Moderation>,
}
