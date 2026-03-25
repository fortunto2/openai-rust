---
name: openai-oxide
description: Use the openai-oxide crate to call OpenAI APIs from Rust, Node.js, and Python. Covers client setup, chat completions, streaming, Responses API, function calling, structured outputs, images, audio, embeddings, file uploads, pagination, Azure, WebSocket sessions, and WASM builds. Use when writing code that calls OpenAI, GPT, DALL-E, Whisper, or TTS APIs.
license: MIT
compatibility: Requires Rust 1.75+ with tokio. Node.js via napi-rs. Python via PyO3.
metadata:
  author: fortunto2
  version: "0.9"
---

# openai-oxide

Idiomatic Rust client for the OpenAI API with 1:1 parity with the official Python SDK. Also available as native Node.js and Python packages.

## Installation

### Rust
```bash
cargo add openai-oxide tokio --features tokio/full
```

### Node.js
```bash
npm install openai-oxide
```

### Python
```bash
pip install openai-oxide
```

## Client Setup

```rust
use openai_oxide::OpenAI;

// From environment variable OPENAI_API_KEY
let client = OpenAI::from_env()?;

// Explicit key
let client = OpenAI::new("sk-...");

// Custom config
use openai_oxide::config::ClientConfig;
let client = OpenAI::with_config(
    ClientConfig::new("sk-...")
        .base_url("https://custom-endpoint.example.com/v1")
        .timeout_secs(30)
        .max_retries(3)
);
```

## Chat Completions

```rust
use openai_oxide::{OpenAI, types::chat::*};

let client = OpenAI::from_env()?;
let request = ChatCompletionRequest::new("gpt-4o")
    .message(ChatMessage::system("You are a helpful assistant."))
    .message(ChatMessage::user("What is Rust?"));

let response = client.chat().completions().create(request).await?;
println!("{}", response.choices[0].message.content.as_deref().unwrap_or_default());
```

## Streaming

```rust
use openai_oxide::{OpenAI, types::chat::*};
use futures_util::StreamExt;

let request = ChatCompletionRequest::new("gpt-4o")
    .message(ChatMessage::user("Write a haiku about Rust."));

let mut stream = client.chat().completions().create_stream(request).await?;
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
        print!("{delta}");
    }
}
```

## Responses API

```rust
use openai_oxide::{OpenAI, types::responses::*};

let response = client.responses().create(
    ResponseCreateRequest::new("gpt-4o")
        .input("Explain quantum computing in one sentence.")
        .max_output_tokens(100)
).await?;

println!("{}", response.output_text());
```

### Streaming Responses

```rust
let mut stream = client.responses().create_stream(
    ResponseCreateRequest::new("gpt-4o").input("Hello!")
).await?;

while let Some(event) = stream.next().await {
    let event = event?;
    // Handle ResponseStreamEvent variants
}
```

## Function Calling / Tools

```rust
use openai_oxide::types::responses::*;

let request = ResponseCreateRequest::new("gpt-4o")
    .input("What is the weather in Paris?")
    .tool(Tool::function(
        "get_weather",
        Some("Get current weather for a city"),
        serde_json::json!({
            "type": "object",
            "properties": {
                "city": { "type": "string" }
            },
            "required": ["city"]
        }),
    ));
```

### Early Function Call Parsing (Streaming)

```rust
let mut handle = client.responses().create_stream_fc(request).await?;
while let Some(fc) = handle.recv().await {
    let result = execute_tool(&fc.name, &fc.arguments).await;
}
```

## Structured Output

```rust
let request = ChatCompletionRequest::new("gpt-4o")
    .message(ChatMessage::user("Extract: John is 30 years old."))
    .response_format(ResponseFormat::json_schema(serde_json::json!({
        "name": "person",
        "strict": true,
        "schema": {
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name", "age"],
            "additionalProperties": false
        }
    })));
```

With `schemars` (feature `structured`):

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct Person { name: String, age: u32 }

let person: Person = client.chat().completions().parse::<Person>(request).await?;
```

## Images

```rust
use openai_oxide::types::image::*;

let response = client.images().generate(
    ImageGenerateRequest::new("A sunset over mountains")
        .model("dall-e-3")
        .size(ImageSize::Size1024x1024)
        .quality(ImageQuality::Hd)
).await?;
```

## Audio

```rust
// Transcription
let transcription = client.audio().transcriptions().create(
    TranscriptionRequest::new("path/to/audio.mp3").model("whisper-1")
).await?;

// Text-to-Speech
let audio_bytes = client.audio().speech().create(
    SpeechRequest::new("tts-1", "Hello world!", AudioVoice::Nova)
).await?;
```

## Embeddings

```rust
use openai_oxide::types::embedding::*;

let response = client.embeddings().create(
    EmbeddingRequest::new("text-embedding-3-small", "Hello world")
).await?;
```

## Pagination

```rust
use futures_util::StreamExt;
use openai_oxide::types::file::FileListParams;

// Auto-paginate all items
let files: Vec<_> = client.files()
    .list_auto(FileListParams::new())
    .collect::<Vec<_>>()
    .await;

// Single page
let page = client.files()
    .list_page(FileListParams::new().limit(10))
    .await?;
```

## WebSocket Sessions (Persistent Connection)

```rust
let mut session = client.ws_session().await?;

let r1 = session.send(
    ResponseCreateRequest::new("gpt-4o").input("My name is Alice.").store(true)
).await?;

let r2 = session.send(
    ResponseCreateRequest::new("gpt-4o").input("What's my name?").previous_response_id(&r1.id)
).await?;

session.close().await?;
```

## Hedged Requests (Latency Optimization)

```rust
use openai_oxide::hedged_request;
use std::time::Duration;

let response = hedged_request(&client, request, Some(Duration::from_secs(2))).await?;
```

## Azure OpenAI

```rust
use openai_oxide::azure::AzureConfig;

let client = OpenAI::azure(
    AzureConfig::new()
        .azure_endpoint("https://my.openai.azure.com")
        .azure_deployment("gpt-4")
        .api_key("...")
)?;
```

## Resource Access Pattern

All resources are zero-cost borrows from the client:

| Resource | Access |
|----------|--------|
| Chat Completions | `client.chat().completions()` |
| Responses | `client.responses()` |
| Embeddings | `client.embeddings()` |
| Models | `client.models()` |
| Images | `client.images()` |
| Audio | `client.audio()` |
| Files | `client.files()` |
| Fine-tuning | `client.fine_tuning().jobs()` |
| Moderations | `client.moderations()` |
| Batches | `client.batches()` |
| Uploads | `client.uploads()` |
| Assistants | `client.beta().assistants()` |
| Threads | `client.beta().threads()` |
| Runs | `client.beta().runs(thread_id)` |
| Vector Stores | `client.beta().vector_stores()` |
| Realtime | `client.beta().realtime()` |

## Feature Flags (Cargo)

```toml
# Full (default) — all APIs
openai-oxide = "0.9"

# Minimal — only what you need
openai-oxide = { version = "0.9", default-features = false, features = ["responses"] }

# WebSocket support
openai-oxide = { version = "0.9", features = ["websocket"] }

# SIMD JSON parsing
openai-oxide = { version = "0.9", features = ["simd"] }

# WASM target
openai-oxide = { version = "0.9", default-features = false, features = ["responses", "websocket-wasm"] }
```

Available: `chat`, `responses`, `embeddings`, `images`, `audio`, `files`, `fine-tuning`, `models`, `moderations`, `batches`, `uploads`, `beta`, `websocket`, `websocket-wasm`, `simd`, `structured`, `webhooks`, `macros`.

## Error Handling

```rust
use openai_oxide::OpenAIError;

match client.chat().completions().create(request).await {
    Ok(response) => println!("{:?}", response),
    Err(OpenAIError::ApiError { status, error }) => {
        eprintln!("API error {status}: {}", error.message);
    }
    Err(e) => eprintln!("Other error: {e}"),
}
```

## Key Design Decisions

- All public types: `Clone + Debug + Send + Sync`
- Builder methods return `&mut Self` for chaining
- All optional API fields are `Option<T>`
- Enums use `#[non_exhaustive]` for forward compatibility
- Streaming returns `impl Stream<Item = Result<T, OpenAIError>>`
- Parameter names match the official Python SDK exactly
