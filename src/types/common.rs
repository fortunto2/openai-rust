// Shared types (Usage, Role, etc.)

use serde::{Deserialize, Serialize};

/// Message role in chat/thread conversations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Role {
    System,
    Developer,
    User,
    Assistant,
    Tool,
    Function,
}

/// Token usage information returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
    /// Detailed breakdown of prompt tokens.
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    /// Detailed breakdown of completion tokens.
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Detailed breakdown of prompt token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<i64>,
    #[serde(default)]
    pub audio_tokens: Option<i64>,
}

/// Detailed breakdown of completion token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: Option<i64>,
    #[serde(default)]
    pub audio_tokens: Option<i64>,
    #[serde(default)]
    pub accepted_prediction_tokens: Option<i64>,
    #[serde(default)]
    pub rejected_prediction_tokens: Option<i64>,
}
