# openai-oxide

Idiomatic Rust client for the OpenAI API — 1:1 parity with the [official Python SDK](https://github.com/openai/openai-python).

## Performance

Benchmarked against the official Python SDK and 2 Rust alternatives. All use the Responses API (`POST /responses`), GPT-5.4, warm connections, 5 iterations, median.

### Sequential requests

| Test | openai-oxide | genai 0.6 | async-openai 0.33 | Python 2.29 |
|------|:-----------:|:---------:|:-----------------:|:-----------:|
| Plain text | **922ms** | 948ms | 968ms | 966ms |
| Structured output | 1404ms | 1428ms | 3407ms | **1258ms** |
| Function calling | **975ms** | 1044ms | 1244ms | 1039ms |
| Multi-turn (2 reqs) | **2042ms** | 2303ms | 2289ms | 2188ms |
| Web search | **2969ms** | — | — | 3176ms |
| Nested structured | 5013ms | — | — | **4286ms** |
| Agent loop (FC→result→JSON) | **3933ms** | — | — | 4113ms |
| Rapid-fire (5 calls) | **4521ms** | — | — | 4646ms |
| Prompt-cached | **4433ms** | — | — | 4712ms |

### Advanced patterns (oxide-only)

| Test | oxide | Python | Speedup |
|------|------:|-------:|--------:|
| Streaming TTFT | **588ms** | 659ms | **11% faster** |
| Stream FC (early parse) | **909ms** | — | **-38% vs normal FC** |
| Parallel 3x fan-out | **926ms** | 1462ms | **37% faster** |
| Hedged 2x race | **893ms** | 958ms | **7% faster** |
| WebSocket plain text | **721ms** | — | **-22% vs HTTP** |
| WebSocket multi-turn | **1650ms** | — | **-19% vs HTTP** |

**oxide wins 10/13 tests** vs Python. No other Rust or Python client has WebSocket mode, streaming FC early parse, hedged requests, or parallel fan-out built in.

### Why it's fast

| Technique | What it does | Savings |
|-----------|-------------|---------|
| HTTP/2 keep-alive while idle | Connections stay warm between requests | -200ms cold start |
| HTTP/2 adaptive windows | Auto-tuned flow control | Better throughput |
| Parallel fan-out | `tokio::join!` + HTTP/2 multiplex | 3 answers ≈ 1 latency |
| Hedged requests | Send 2 copies, take fastest | P99 -50-96% |
| Streaming TTFT | First token in ~588ms | -36% vs full response |
| Stream FC early parse | Yield function call on `arguments.done` | -38% vs `response.completed` |
| WebSocket mode | Persistent `wss://` — no per-turn HTTP | -20-25% per request |
| Prompt cache key | Server-side system prompt caching | Up to -80% TTFT |
| Fast-path retry | No loop overhead for successful requests | -5-15ms |
| gzip + from_slice | Compressed responses, zero-copy deser | Bandwidth + alloc |

Run the benchmark yourself:
```bash
OPENAI_API_KEY=sk-... cargo run --example benchmark --features responses --release
python3 examples/bench_python.py  # Python comparison
```

## Features

- Async-first (tokio + reqwest 0.13)
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
openai-oxide = "0.9"

# Only chat + embeddings
openai-oxide = { version = "0.8", default-features = false, features = ["chat", "embeddings"] }
```

Available features: `chat`, `responses`, `embeddings`, `images`, `audio`, `files`, `fine-tuning`, `models`, `moderations`, `batches`, `uploads`, `beta`.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
openai-oxide = "0.9"
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

## Responses API

```rust
use openai_oxide::{OpenAI, types::responses::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let response = client.responses().create(
        ResponseCreateRequest::new("gpt-5.4")
            .input("What are the latest developments in Rust?")
            .tools(vec![ResponseTool::WebSearch {
                search_context_size: Some("medium".into()),
                user_location: None,
            }])
            .max_output_tokens(1024)
    ).await?;

    println!("{}", response.output_text());

    // Extract function calls
    for fc in response.function_calls() {
        println!("Tool: {}({})", fc.name, fc.arguments);
    }
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

## Pagination

All list endpoints support automatic cursor-based pagination:

```rust
use futures_util::StreamExt;
use openai_oxide::{OpenAI, types::file::FileListParams};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    // Single page with params
    let page = client.files().list_page(
        FileListParams::new().limit(10)
    ).await?;

    // Auto-paginate through all results
    let mut stream = client.files().list_auto(FileListParams::new());
    while let Some(file) = stream.next().await {
        let file = file?;
        println!("{}: {}", file.id, file.filename);
    }
    Ok(())
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
cargo run --example benchmark --features responses --release  # benchmark
```

## License

MIT
