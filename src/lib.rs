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

pub mod azure;
pub mod client;
pub mod config;
pub mod error;
#[cfg(feature = "responses")]
pub mod hedged;
pub mod pagination;
pub mod request_options;
pub mod resources;
pub mod streaming;
pub mod types;
#[cfg(feature = "websocket")]
pub mod websocket;

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
