// Manual: hand-crafted embedding types (enums, builders, response structs).

use serde::{Deserialize, Serialize};

/// Encoding format for embedding output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub enum EncodingFormat {
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "base64")]
    Base64,
}

/// Input for embeddings: a single string, array of strings, or array of token arrays.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum EmbeddingInput {
    String(String),
    StringArray(Vec<String>),
    Tokens(Vec<Vec<i64>>),
}

impl From<&str> for EmbeddingInput {
    fn from(s: &str) -> Self {
        EmbeddingInput::String(s.to_string())
    }
}

impl From<String> for EmbeddingInput {
    fn from(s: String) -> Self {
        EmbeddingInput::String(s)
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        EmbeddingInput::StringArray(v)
    }
}

impl From<Vec<Vec<i64>>> for EmbeddingInput {
    fn from(v: Vec<Vec<i64>>) -> Self {
        EmbeddingInput::Tokens(v)
    }
}

/// Request body for `POST /embeddings`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct EmbeddingCreateRequest {
    /// Input text to embed.
    pub input: EmbeddingInput,

    /// Embedding model (e.g. "text-embedding-3-small").
    pub model: String,

    /// Encoding format for the embedding vectors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,

    /// Number of dimensions to return (for supported models).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<i64>,

    /// A unique identifier representing your end-user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl EmbeddingCreateRequest {
    pub fn new(input: impl Into<EmbeddingInput>, model: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            model: model.into(),
            encoding_format: None,
            dimensions: None,
            user: None,
        }
    }
}

/// Backward compatibility alias.
pub type EmbeddingRequest = EmbeddingCreateRequest;

/// OpenAPI spec name alias.
pub type CreateEmbeddingRequest = EmbeddingCreateRequest;

/// A single embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct Embedding {
    pub object: String,
    /// The embedding vector (when encoding_format is float).
    #[serde(default)]
    pub embedding: Option<Vec<f64>>,
    /// Base64-encoded embedding (when encoding_format is base64).
    #[serde(default)]
    pub b64_embedding: Option<String>,
    pub index: i64,
}

/// Response from `POST /embeddings`.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: super::Usage,
}
