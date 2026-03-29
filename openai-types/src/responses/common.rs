// MANUAL — hand-maintained. py2rust sync will not overwrite.
// Shared enums used across Responses API types.
// Re-exports canonical types from crate::shared where they exist.

use serde::{Deserialize, Serialize};

// Re-export canonical types from shared — single source of truth.
pub use crate::shared::{ReasoningEffort, ReasoningSummary, Role, ServiceTier};

/// Image detail level for vision inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
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
