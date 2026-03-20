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

/// A value that is either "auto" or a fixed number.
///
/// Used for hyperparameters like `n_epochs`, `batch_size`, `learning_rate_multiplier`.
/// Serializes as the string `"auto"` or a bare number.
#[derive(Debug, Clone, PartialEq)]
pub enum AutoOrFixed<T> {
    Auto,
    Fixed(T),
}

impl<T: Serialize> Serialize for AutoOrFixed<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Fixed(v) => v.serialize(serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for AutoOrFixed<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match &value {
            serde_json::Value::String(s) if s == "auto" => Ok(Self::Auto),
            _ => T::deserialize(value)
                .map(Self::Fixed)
                .map_err(serde::de::Error::custom),
        }
    }
}

/// Token limit that is either "inf" (unlimited) or a fixed integer.
///
/// Used for `max_response_output_tokens` in the Realtime API.
#[derive(Debug, Clone, PartialEq)]
pub enum MaxResponseTokens {
    Inf,
    Fixed(i64),
}

impl Serialize for MaxResponseTokens {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Inf => serializer.serialize_str("inf"),
            Self::Fixed(v) => serializer.serialize_i64(*v),
        }
    }
}

impl<'de> Deserialize<'de> for MaxResponseTokens {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match &value {
            serde_json::Value::String(s) if s == "inf" => Ok(Self::Inf),
            serde_json::Value::Number(n) => n
                .as_i64()
                .map(Self::Fixed)
                .ok_or_else(|| serde::de::Error::custom("expected integer")),
            _ => Err(serde::de::Error::custom("expected \"inf\" or integer")),
        }
    }
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
