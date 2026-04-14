//! # openai-oxide
//!
//! Idiomatic Rust client for the OpenAI API — 1:1 parity with the official Python SDK.
//!
//! ## Quick Start
//!
//! ```no_run
//! use openai_oxide::{OpenAI, types::chat::*};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), openai_oxide::OpenAIError> {
//!     let client = OpenAI::from_env()?;
//!
//!     let request = ChatCompletionRequest::new(
//!         "gpt-4o-mini",
//!         vec![
//!             ChatCompletionMessageParam::System {
//!                 content: "You are a helpful assistant.".into(),
//!                 name: None,
//!             },
//!             ChatCompletionMessageParam::User {
//!                 content: UserContent::Text("Hello!".into()),
//!                 name: None,
//!             },
//!         ],
//!     );
//!
//!     let response = client.chat().completions().create(request).await?;
//!     println!("{}", response.choices[0].message.content.as_deref().unwrap_or(""));
//!     Ok(())
//! }
//! ```

// When `reqwest-012` feature is enabled, alias reqwest012 → reqwest.
// Code uses `reqwest::` everywhere — this makes 0.12 transparent.
// Consumer adds: openai-oxide = { features = ["reqwest-012"] }
// Cargo unifies with their workspace's reqwest 0.12, avoiding dual versions.
#[cfg(feature = "reqwest-012")]
extern crate reqwest012 as reqwest;

pub mod anthropic;
pub mod azure;
pub mod client;
pub mod cloudflare;
pub mod config;
pub mod error;
#[cfg(feature = "responses")]
pub mod hedged;
#[cfg(not(target_arch = "wasm32"))]
pub mod middleware;
pub mod openrouter;
pub mod pagination;
#[cfg(feature = "structured")]
pub mod parsing;
#[cfg(not(target_arch = "wasm32"))]
pub mod rate_limit;
pub mod request_options;
pub mod resources;
pub(crate) mod runtime;
pub mod schema;
pub mod stream_helpers;
pub mod streaming;
pub mod types;

/// Direct access to the standalone types crate (1100+ OpenAI API types).
///
/// Same types as `openai_oxide::types::*`, but also usable independently:
/// ```toml
/// [dependencies]
/// openai-types = { version = "0.1", features = ["chat", "responses"] }
/// ```
pub use openai_types;
#[cfg(feature = "websocket")]
pub mod websocket;
#[cfg(feature = "websocket")]
pub mod ws_pool;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn ensure_tls_provider() {
    use std::sync::Once;

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

pub use azure::AzureConfig;
pub use client::OpenAI;
pub use config::ClientConfig;
pub use error::OpenAIError;
#[cfg(feature = "responses")]
pub use hedged::{hedged_request, hedged_request_n, speculative};
pub use pagination::Paginator;
pub use request_options::RequestOptions;
pub use streaming::SseStream;
#[cfg(feature = "websocket")]
pub use websocket::{WsEventStream, WsSession};
