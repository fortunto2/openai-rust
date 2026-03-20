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

/// Reason the model stopped generating tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "stop"),
            Self::Length => write!(f, "length"),
            Self::ToolCalls => write!(f, "tool_calls"),
            Self::ContentFilter => write!(f, "content_filter"),
            Self::FunctionCall => write!(f, "function_call"),
        }
    }
}

/// Service tier used for the request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ServiceTier {
    Auto,
    Default,
    Flex,
    Scale,
    Priority,
}

/// Reasoning effort level for o-series models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

/// Search context size for web search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SearchContextSize {
    Low,
    Medium,
    High,
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
