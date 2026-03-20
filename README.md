# openai-oxide

Idiomatic Rust client for the OpenAI API — 1:1 parity with the [official Python SDK](https://github.com/openai/openai-python).

## Features

- Async-first (tokio + reqwest)
- Strongly typed requests and responses (serde)
- SSE streaming for Chat Completions and Responses API
- Automatic retries with exponential backoff
- Chainable builder pattern for requests
- Responses API with tool support (WebSearch, FileSearch, MCP, etc.)
- Structured outputs (JSON Schema with strict mode)
- Reasoning model support (o-series: effort, summary)
- Realtime API session creation (ephemeral tokens)
- 100% OpenAPI field coverage for Chat Completions
- Same resource structure as Python SDK: `client.chat().completions().create()`

## Feature Flags

Each API resource is behind an optional Cargo feature (all enabled by default):

```toml
# All resources (default)
openai-oxide = "0.6"

# Only chat + embeddings
openai-oxide = { version = "0.6", default-features = false, features = ["chat", "embeddings"] }
```

Available features: `chat`, `responses`, `embeddings`, `images`, `audio`, `files`, `fine-tuning`, `models`, `moderations`, `batches`, `uploads`, `beta`.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
openai-oxide = "0.6"
tokio = { version = "1", features = ["full"] }
```

```rust
use openai_oxide::{OpenAI, types::chat::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![
            ChatCompletionMessageParam::System {
                content: "You are a helpful assistant.".into(),
                name: None,
            },
            ChatCompletionMessageParam::User {
                content: UserContent::Text("Hello!".into()),
                name: None,
            },
        ],
    );

    let response = client.chat().completions().create(request).await?;
    println!("{}", response.choices[0].message.content.as_deref().unwrap_or(""));
    Ok(())
}
```

## Streaming

```rust
use futures_util::StreamExt;
use openai_oxide::{OpenAI, types::chat::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let request = ChatCompletionRequest::new(
        "gpt-4o-mini",
        vec![ChatCompletionMessageParam::User {
            content: UserContent::Text("Tell me a joke".into()),
            name: None,
        }],
    );

    let mut stream = client.chat().completions().create_stream(request).await?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
            print!("{delta}");
        }
    }
    Ok(())
}
```

## BYOT (Bring Your Own Types)

Send custom fields or get raw JSON responses using `create_raw()`:

```rust
use openai_oxide::OpenAI;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let raw = client.chat().completions().create_raw(&json!({
        "model": "gpt-4o",
        "messages": [{"role": "user", "content": "Hi"}],
        "custom_field": true
    })).await?;

    println!("{}", raw["choices"][0]["message"]["content"]);
    Ok(())
}
```

Also available on `client.responses().create_raw()` and `client.embeddings().create_raw()`.

## Image Save Helper

Save generated images directly to disk:

```rust
let resp = client.images().generate(req).await?;
if let Some(images) = &resp.data {
    images[0].save("output.png").await?;  // handles both URL and b64_json
}
```

## Configuration

```rust
use openai_oxide::{OpenAI, ClientConfig};

// From environment variable OPENAI_API_KEY
let client = OpenAI::from_env()?;

// Explicit API key
let client = OpenAI::new("sk-...");

// Full configuration
let config = ClientConfig::new("sk-...")
    .base_url("https://api.openai.com/v1")
    .timeout_secs(30)
    .max_retries(3);
let client = OpenAI::with_config(config);
```

## Implemented APIs

| API | Method | Status |
|-----|--------|--------|
| Chat Completions | `client.chat().completions().create()` | Done |
| Chat Completions (streaming) | `client.chat().completions().create_stream()` | Done |
| Responses | `client.responses().create()` / `create_stream()` | Done |
| Responses Tools | Function, WebSearch, FileSearch, CodeInterpreter, ComputerUse, Mcp | Done |
| Embeddings | `client.embeddings().create()` | Done |
| Models | `client.models().list()` / `retrieve()` / `delete()` | Done |
| Images | `client.images().generate()` / `edit()` / `create_variation()` | Done |
| Audio Transcription | `client.audio().transcriptions().create()` | Done |
| Audio Translation | `client.audio().translations().create()` | Done |
| Audio Speech (TTS) | `client.audio().speech().create()` | Done |
| Files | `client.files().create()` / `list()` / `retrieve()` / `delete()` / `content()` | Done |
| Fine-tuning | `client.fine_tuning().jobs().create()` / `list()` / `cancel()` / `list_events()` | Done |
| Moderations | `client.moderations().create()` | Done |
| Batches | `client.batches().create()` / `list()` / `retrieve()` / `cancel()` | Done |
| Uploads | `client.uploads().create()` / `cancel()` / `complete()` | Done |
| Assistants (beta) | `client.beta().assistants().create()` / `list()` / `retrieve()` / `delete()` | Done |
| Threads (beta) | `client.beta().threads().create()` / `retrieve()` / `delete()` / `messages()` | Done |
| Runs (beta) | `client.beta().runs(thread_id).create()` / `retrieve()` / `cancel()` | Done |
| Vector Stores (beta) | `client.beta().vector_stores().create()` / `list()` / `retrieve()` / `delete()` | Done |
| Realtime (beta) | `client.beta().realtime().sessions().create()` | Done |

## Development

```bash
cargo test                          # all tests
cargo test --features live-tests    # tests hitting real API (needs OPENAI_API_KEY)
cargo clippy -- -D warnings         # lint
cargo fmt -- --check                # format check
```

## License

MIT
