// Shared enums used across Responses API types.

use serde::{Deserialize, Serialize};

/// Message role in conversations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Role {
    /// System-level instructions.
    System,
    /// Developer-level instructions (newer alias for system).
    Developer,
    /// User input.
    User,
    /// Model output.
    Assistant,
    /// Tool/function output.
    Tool,
    /// Legacy function role.
    Function,
}

/// Image detail level for vision inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageDetail {
    /// Let the model decide.
    Auto,
    /// Low resolution — faster, fewer tokens.
    Low,
    /// High resolution — more detail, more tokens.
    High,
}

/// Reasoning effort level for o-series models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ReasoningEffort {
    /// Minimal reasoning.
    Low,
    /// Balanced reasoning.
    Medium,
    /// Maximum reasoning depth.
    High,
}

/// Summary mode for reasoning output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ReasoningSummary {
    /// Automatically determine summary level.
    Auto,
    /// Brief summary.
    Concise,
    /// Detailed summary.
    Detailed,
}

/// Service tier used for the request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ServiceTier {
    /// Automatic tier selection.
    Auto,
    /// Default tier.
    Default,
    /// Flexible tier (lower priority, lower cost).
    Flex,
    /// Scale tier.
    Scale,
    /// Priority tier.
    Priority,
}
