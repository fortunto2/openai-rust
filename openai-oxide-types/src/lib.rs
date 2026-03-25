//! Typed OpenAI API models — zero dependencies beyond serde.
//!
//! This crate provides strongly-typed Rust structs and enums for the OpenAI API,
//! suitable for use without an HTTP client.

pub mod responses;

// Re-export for convenience
pub use responses::*;
