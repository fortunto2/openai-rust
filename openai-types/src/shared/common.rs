// Manual: canonical definitions of shared types used across domains.
// Role, FinishReason, Usage, ServiceTier, etc. — one source of truth.
// All enums have Other(String) catch-all for forward compatibility.

use serde::{Deserialize, Serialize};

// ── Enums with Other(String) catch-all ──

/// Message role in chat/thread conversations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Role {
    System,
    Developer,
    User,
    Assistant,
    Tool,
    Function,
    Other(String),
}

impl Serialize for Role {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::System => serializer.serialize_str("system"),
            Self::Developer => serializer.serialize_str("developer"),
            Self::User => serializer.serialize_str("user"),
            Self::Assistant => serializer.serialize_str("assistant"),
            Self::Tool => serializer.serialize_str("tool"),
            Self::Function => serializer.serialize_str("function"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "system" => Ok(Self::System),
            "developer" => Ok(Self::Developer),
            "user" => Ok(Self::User),
            "assistant" => Ok(Self::Assistant),
            "tool" => Ok(Self::Tool),
            "function" => Ok(Self::Function),
            _ => Ok(Self::Other(s)),
        }
    }
}

/// Reason the model stopped generating tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
    Other(String),
}

impl Serialize for FinishReason {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Stop => serializer.serialize_str("stop"),
            Self::Length => serializer.serialize_str("length"),
            Self::ToolCalls => serializer.serialize_str("tool_calls"),
            Self::ContentFilter => serializer.serialize_str("content_filter"),
            Self::FunctionCall => serializer.serialize_str("function_call"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for FinishReason {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "stop" => Ok(Self::Stop),
            "length" => Ok(Self::Length),
            "tool_calls" => Ok(Self::ToolCalls),
            "content_filter" => Ok(Self::ContentFilter),
            "function_call" => Ok(Self::FunctionCall),
            _ => Ok(Self::Other(s)),
        }
    }
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "stop"),
            Self::Length => write!(f, "length"),
            Self::ToolCalls => write!(f, "tool_calls"),
            Self::ContentFilter => write!(f, "content_filter"),
            Self::FunctionCall => write!(f, "function_call"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

/// Service tier used for the request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServiceTier {
    Auto,
    Default,
    Flex,
    Scale,
    Priority,
    Other(String),
}

impl Serialize for ServiceTier {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Default => serializer.serialize_str("default"),
            Self::Flex => serializer.serialize_str("flex"),
            Self::Scale => serializer.serialize_str("scale"),
            Self::Priority => serializer.serialize_str("priority"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceTier {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "auto" => Ok(Self::Auto),
            "default" => Ok(Self::Default),
            "flex" => Ok(Self::Flex),
            "scale" => Ok(Self::Scale),
            "priority" => Ok(Self::Priority),
            _ => Ok(Self::Other(s)),
        }
    }
}

// ReasoningEffort: canonical definition is in _gen.rs (auto-generated, has full variant set).

/// Search context size for web search.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SearchContextSize {
    Low,
    Medium,
    High,
    Other(String),
}

impl Serialize for SearchContextSize {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Low => serializer.serialize_str("low"),
            Self::Medium => serializer.serialize_str("medium"),
            Self::High => serializer.serialize_str("high"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for SearchContextSize {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Ok(Self::Other(s)),
        }
    }
}

/// Sort order for paginated list endpoints.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SortOrder {
    Asc,
    Desc,
    Other(String),
}

impl Serialize for SortOrder {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Asc => serializer.serialize_str("asc"),
            Self::Desc => serializer.serialize_str("desc"),
            Self::Other(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for SortOrder {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Ok(Self::Other(s)),
        }
    }
}

// ── Structs ──

/// Token usage information returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Usage {
    #[serde(default)]
    pub prompt_tokens: Option<i64>,
    #[serde(default)]
    pub completion_tokens: Option<i64>,
    #[serde(default)]
    pub total_tokens: Option<i64>,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

impl Usage {
    /// Number of prompt tokens served from cache (0 if no cache hit).
    pub fn cached_tokens(&self) -> i64 {
        self.prompt_tokens_details
            .as_ref()
            .and_then(|d| d.cached_tokens)
            .unwrap_or(0)
    }

    /// Cache hit ratio as percentage (0-100).
    pub fn cache_hit_pct(&self) -> u64 {
        let input = self.prompt_tokens.unwrap_or(0) as u64;
        let cached = self.cached_tokens() as u64;
        if input > 0 { (cached * 100) / input } else { 0 }
    }
}

/// Detailed breakdown of prompt token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct PromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<i64>,
    #[serde(default)]
    pub audio_tokens: Option<i64>,
}

/// Detailed breakdown of completion token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
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

// ── Generic helpers ──

/// A value that is either "auto" or a fixed number.
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

/// Token limit: either "inf" (unlimited) or a fixed integer.
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
