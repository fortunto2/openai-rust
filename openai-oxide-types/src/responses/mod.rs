//! OpenAI Responses API types.
//!
//! This module contains all types needed to work with the OpenAI Responses API,
//! including request, response, streaming, tool, and input/output types.

pub mod common;
pub mod create;
pub mod input;
pub mod output;
pub mod response;
pub mod streaming;
pub mod tools;

// Re-export key types at module level for convenience
pub use common::*;
pub use create::*;
pub use input::*;
pub use output::{
    FunctionCall, FunctionToolCall, OutputItem, ReasoningContent, ReasoningItem,
    ResponseOutputContent, ResponseOutputItem, SummaryPart, SummaryTextContent,
};
pub use response::*;
pub use streaming::*;
pub use tools::*;
