// Shared types — canonical definitions live in openai-types/src/shared/common.rs.
// Re-exported here for backward compatibility.

pub use openai_types::shared::{
    AutoOrFixed, CompletionTokensDetails, FinishReason, ListResponse, MaxResponseTokens,
    PromptTokensDetails, Role, SearchContextSize, ServiceTier, SortOrder, Usage,
};

// ReasoningEffort comes from _gen.rs in shared (has full variant set including None, Minimal, Xhigh)
pub use openai_types::shared::ReasoningEffort;

/// Macro to create an OpenAI API enum with forward-compatible `Other(String)` variant.
///
/// Syntax: `VariantName = "json_value"`
///
/// Example:
/// ```ignore
/// openai_enum! {
///     /// Message role
///     pub enum Role {
///         System = "system",
///         Developer = "developer",
///         InProgress = "in_progress",  // auto-handles snake_case
///         FineTune = "fine-tune",      // handles hyphens
///     }
/// }
/// ```
#[macro_export]
macro_rules! openai_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$var_meta:meta])*
                $variant:ident = $json:literal
            ),*$(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$var_meta])*
                $variant,
            )*
            /// Catch-all for unknown variants (forward compatibility).
            Other(String),
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $(Self::$variant => serializer.serialize_str($json),)*
                    Self::Other(s) => serializer.serialize_str(s),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                match s.as_str() {
                    $($json => Ok(Self::$variant),)*
                    _ => Ok(Self::Other(s)),
                }
            }
        }
    };
}

/// Helper function to serialize the `Other(String)` variant.
pub fn serialize_other<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(value)
}
