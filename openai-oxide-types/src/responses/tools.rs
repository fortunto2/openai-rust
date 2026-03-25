// MANUAL — hand-maintained. py2rust sync will not overwrite.
// Tool definitions for the Responses API.

use serde::{Deserialize, Serialize};

/// A function tool definition for the Responses API.
///
/// Maps to Python SDK `FunctionTool`. Standalone struct for typed tool definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTool {
    /// The name of the function.
    pub name: String,
    /// JSON Schema object describing the parameters.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    /// Whether to enforce strict parameter validation.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    /// A description of the function.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Ranking options for file search in the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseRankingOptions {
    /// Score threshold (0.0-1.0).
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score_threshold: Option<f64>,
    /// Ranker to use: "auto" or "default-2024-11-15".
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranker: Option<String>,
}

/// Tool types for the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ResponseTool {
    /// Function tool.
    #[serde(rename = "function")]
    Function {
        /// Function name.
        name: String,
        /// A description of the function.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// JSON Schema object describing the parameters.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<serde_json::Value>,
        /// Whether to enforce strict parameter validation.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
    /// Web search tool.
    #[serde(rename = "web_search")]
    WebSearch {
        /// Search context size.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        search_context_size: Option<String>,
        /// User location.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        user_location: Option<serde_json::Value>,
    },
    /// File search tool.
    #[serde(rename = "file_search")]
    FileSearch {
        /// Vector store IDs.
        vector_store_ids: Vec<String>,
        /// Max results.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        max_num_results: Option<i64>,
        /// Ranking options.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_options: Option<ResponseRankingOptions>,
    },
    /// Code interpreter tool.
    #[serde(rename = "code_interpreter")]
    CodeInterpreter {
        /// Container ID.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        container: Option<String>,
    },
    /// Computer use tool.
    #[serde(rename = "computer")]
    ComputerUse {},
    /// MCP tool.
    #[serde(rename = "mcp")]
    Mcp {
        /// Server label.
        server_label: String,
        /// Server URL.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        server_url: Option<String>,
        /// Allowed tools.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        allowed_tools: Option<Vec<String>>,
        /// Approval config — polymorphic ("never" | filter object), kept as Value.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        require_approval: Option<serde_json::Value>,
    },
    /// Image generation tool.
    #[serde(rename = "image_generation")]
    ImageGeneration {
        /// Model to use for generation.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        /// Image quality.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        quality: Option<String>,
        /// Image size.
        #[serde(default)]
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<String>,
    },
}

/// How the model selects tools in the Responses API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ResponseToolChoice {
    /// "none", "auto", or "required".
    Mode(String),
    /// Force a specific function by name.
    Named {
        /// Type — usually "function".
        #[serde(rename = "type")]
        type_: String,
        /// The function to call.
        function: ResponseToolChoiceFunction,
    },
}

/// Specifies which function to call in tool choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseToolChoiceFunction {
    /// The function name.
    pub name: String,
}

/// How the model selects tools — typed version.
///
/// Maps to Python SDK `ToolChoiceOptions` + `ToolChoiceFunction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ToolChoiceParam {
    /// Predefined mode: none, auto, or required.
    Mode(ToolChoiceOptions),
    /// Force a specific function by name.
    Function(ToolChoiceFunction),
}

/// Predefined tool choice modes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolChoiceOptions {
    /// Do not use any tools.
    None,
    /// Let the model decide.
    Auto,
    /// Force tool use.
    Required,
}

/// Force a specific function tool by name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolChoiceFunction {
    /// The name of the function to call.
    pub name: String,
    /// The type — always "function".
    #[serde(rename = "type")]
    #[serde(default = "default_function_type")]
    pub type_: String,
}

fn default_function_type() -> String {
    "function".to_string()
}
