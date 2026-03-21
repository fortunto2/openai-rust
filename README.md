<p align="center">
  <h1 align="center">openai-oxide</h1>
  <p align="center">
    The fastest OpenAI client for Rust. Beats Python on 10 out of 13 benchmarks.
  </p>
  <p align="center">
    <a href="https://crates.io/crates/openai-oxide"><img src="https://img.shields.io/crates/v/openai-oxide.svg" alt="crates.io"></a>
    <a href="https://crates.io/crates/openai-oxide"><img src="https://img.shields.io/crates/d/openai-oxide.svg" alt="downloads"></a>
    <a href="https://docs.rs/openai-oxide"><img src="https://docs.rs/openai-oxide/badge.svg" alt="docs.rs"></a>
    <a href="https://github.com/fortunto2/openai-rust/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
    <a href="https://github.com/fortunto2/openai-rust"><img src="https://img.shields.io/github/stars/fortunto2/openai-rust?style=social" alt="GitHub stars"></a>
  </p>
</p>

Full [Responses API](https://platform.openai.com/docs/api-reference/responses) + [Chat Completions](https://platform.openai.com/docs/api-reference/chat) + 20 endpoints. [WebSocket mode](https://platform.openai.com/docs/guides/websocket-mode), hedged requests, streaming FC early parse — features no other client has.

```toml
[dependencies]
openai-oxide = "0.9"
tokio = { version = "1", features = ["full"] }
```

```rust
let client = OpenAI::from_env()?;

// Simple
let r = client.responses().create(
    ResponseCreateRequest::new("gpt-5.4").input("Hello!")
).await?;
println!("{}", r.output_text());

// WebSocket (persistent connection, -22% latency)
let mut ws = client.ws_session().await?;
let r = ws.send(ResponseCreateRequest::new("gpt-5.4").input("Hello!")).await?;

// Hedged (send 2, take fastest — P99 reduced 50-96%)
let r = hedged_request(&client, request, Some(Duration::from_secs(2))).await?;

// Parallel fan-out (3 answers for the price of 1 round-trip)
let (r1, r2, r3) = tokio::join!(
    async { c1.responses().create(req1).await },
    async { c2.responses().create(req2).await },
    async { c3.responses().create(req3).await },
);

// Streaming FC early parse (-38% tool call latency)
let mut handle = client.responses().create_stream_fc(request).await?;
while let Some(fc) = handle.recv().await {
    execute_tool(&fc.name, &fc.arguments).await;  // starts BEFORE response.completed
}
```

## Benchmarks

All clients use the Responses API, GPT-5.4, warm connections, 5 iterations, median.

### 4 clients compared

| Test | openai-oxide | [genai](https://crates.io/crates/genai) 0.6 | [async-openai](https://crates.io/crates/async-openai) 0.33 | [Python openai](https://pypi.org/project/openai/) 2.29 |
|------|:-----------:|:---------:|:-----------------:|:-----------:|
| Plain text | **922ms** | 948ms | 968ms | 966ms |
| Structured output | 1404ms | 1428ms | 3407ms | **1258ms** |
| Function calling | **975ms** | 1044ms | 1244ms | 1039ms |
| Multi-turn (2 reqs) | **2042ms** | 2303ms | 2289ms | 2188ms |
| Web search | **2969ms** | — | — | 3176ms |
| Nested structured | 5013ms | — | — | **4286ms** |
| Agent loop (FC+result+JSON) | **3933ms** | — | — | 4113ms |
| Rapid-fire (5 calls) | **4521ms** | — | — | 4646ms |
| Prompt-cached | **4433ms** | — | — | 4712ms |

### Unique to oxide

| Test | oxide | Python | Speedup |
|------|------:|-------:|--------:|
| Streaming TTFT | **588ms** | 659ms | **11%** |
| Stream FC early parse | **909ms** | — | **-38% vs normal FC** |
| Parallel 3x fan-out | **926ms** | 1462ms | **37%** |
| Hedged 2x race | **893ms** | 958ms | **7%** |
| WebSocket plain text | **721ms** | — | **-22% vs HTTP** |
| WebSocket multi-turn | **1650ms** | — | **-19% vs HTTP** |

**Wins 10/13 vs Python.** Reproduce: `cargo run --example benchmark --features responses --release`

### 10 techniques that make it fast

| # | Technique | Savings |
|---|-----------|---------|
| 1 | **WebSocket mode** — persistent `wss://`, no per-turn HTTP overhead | -20-25% |
| 2 | **Stream FC early parse** — yield tool calls on `arguments.done` | -38% FC |
| 3 | **Parallel fan-out** — `tokio::join!` + HTTP/2 multiplex | 3x = 1x |
| 4 | **Hedged requests** — send 2, take fastest, cancel loser | P99 -50-96% |
| 5 | **HTTP/2 keep-alive while idle** — connections never go cold | -200ms |
| 6 | **Adaptive flow control** — auto-tuned HTTP/2 windows | Throughput |
| 7 | **Streaming TTFT** — first token in ~588ms | -36% |
| 8 | **Prompt cache key** — server-side prefix caching | Up to -80% |
| 9 | **Fast-path retry** — no loop overhead for success | -5-15ms |
| 10 | **gzip + from_slice** — compressed, zero intermediate alloc | Bandwidth |

## Features

- **Async-first** — [tokio](https://tokio.rs/) + [reqwest](https://crates.io/crates/reqwest) 0.13, HTTP/2
- **Strongly typed** — [serde](https://serde.rs/), every field documented
- **Streaming** — SSE with zero-copy parser (no external deps)
- **[WebSocket](https://platform.openai.com/docs/guides/websocket-mode)** — persistent `wss://` for agent loops (opt-in: `websocket` feature)
- **Hedged requests** — inspired by [Google's "The Tail at Scale"](https://research.google/pubs/the-tail-at-scale/)
- **[Responses API](https://platform.openai.com/docs/api-reference/responses)** — create, stream, retrieve, delete, tools (WebSearch, FileSearch, MCP, Function, CodeInterpreter, ComputerUse, ImageGeneration)
- **[Chat Completions](https://platform.openai.com/docs/api-reference/chat)** — full parity with [Python SDK](https://github.com/openai/openai-python)
- **[Structured outputs](https://platform.openai.com/docs/guides/structured-outputs)** — JSON Schema with strict mode
- **Reasoning models** — [o-series](https://platform.openai.com/docs/guides/reasoning) (effort, summary)
- **BYOT** — `create_raw()` for custom types / raw JSON
- **Pagination** — auto cursor-based streaming
- **Retries** — exponential backoff with Retry-After
- **[Azure](https://learn.microsoft.com/en-us/azure/ai-services/openai/)** — full Azure OpenAI support
- **195 tests** — unit + integration + [OpenAPI](https://github.com/openai/openai-openapi) coverage

## Feature Flags

```toml
openai-oxide = "0.9"                                            # all APIs (default)
openai-oxide = { version = "0.9", features = ["websocket"] }    # + WebSocket mode
openai-oxide = { version = "0.9", features = ["simd"] }         # + simd-json deser
```

API features (all default): `chat`, `responses`, `embeddings`, `images`, `audio`, `files`, `fine-tuning`, `models`, `moderations`, `batches`, `uploads`, `beta`.

## WASM Support

Compiles to `wasm32-unknown-unknown` — run in browsers, Cloudflare Workers, Deno Deploy.

```toml
# Cargo.toml for WASM
[dependencies]
openai-oxide = { version = "0.9", default-features = false, features = ["chat", "responses"] }
```

**Unlike [async-openai](https://github.com/64bit/async-openai) which disables streaming, retry, and all advanced features on WASM**, oxide keeps everything working:

| Feature | oxide WASM | async-openai WASM |
|---------|:----------:|:-----------------:|
| Chat / Responses API | **yes** | yes |
| Streaming SSE | **yes** | no |
| Stream FC early parse | **yes** | no |
| Hedged requests | **yes** | no |
| Parallel fan-out | **yes** | no |
| Speculative execution | **yes** | no |
| Retry with backoff | **yes** | no |

Cross-platform via [`runtime.rs`](src/runtime.rs) — one codebase, zero duplication:
- `sleep()` → tokio on native, [gloo-timers](https://crates.io/crates/gloo-timers) on WASM
- `spawn()` → tokio::spawn on native, [wasm-bindgen-futures](https://crates.io/crates/wasm-bindgen-futures) spawn_local on WASM
- `timeout()` → tokio::time::timeout on native, pass-through on WASM (browser handles)

## Quick Start

```rust
use openai_oxide::{OpenAI, types::responses::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;

    let response = client.responses().create(
        ResponseCreateRequest::new("gpt-5.4")
            .input("Explain quantum computing in one sentence.")
            .max_output_tokens(100)
    ).await?;

    println!("{}", response.output_text());
    Ok(())
}
```

## WebSocket Mode

Persistent connection — no TLS handshake per request. Ideal for agent loops.

```rust
let client = OpenAI::from_env()?;
let mut session = client.ws_session().await?;

// All calls go through the same wss:// connection
let r1 = session.send(
    ResponseCreateRequest::new("gpt-5.4").input("My name is Rustam.").store(true)
).await?;

let r2 = session.send(
    ResponseCreateRequest::new("gpt-5.4")
        .input("What's my name?")
        .previous_response_id(&r1.id)
).await?;

session.close().await?;
```

## Streaming FC Early Parse

Start executing tools ~400ms earlier. Safe — `arguments.done` guarantees complete JSON.

```rust
let mut handle = client.responses().create_stream_fc(request).await?;

while let Some(fc) = handle.recv().await {
    // Fires on arguments.done, NOT response.completed
    let result = execute_tool(&fc.name, &fc.arguments).await;
}

if let Some(err) = handle.error().await {
    eprintln!("Error: {err}");
}
```

## Hedged Requests

Send duplicates, take the fastest. Costs 2-7% extra tokens, reduces P99 by 50-96%.

```rust
use openai_oxide::hedged_request;
use std::time::Duration;

// Send 2 copies with 1.5s hedge delay, return whichever finishes first
let response = hedged_request(&client, request, Some(Duration::from_secs(2))).await?;
```

## Parallel Fan-Out

HTTP/2 multiplexing — 3 requests on 1 connection, wall time = slowest single request.

```rust
let (c1, c2, c3) = (client.clone(), client.clone(), client.clone());
let (r1, r2, r3) = tokio::join!(
    async { c1.responses().create(req1).await },
    async { c2.responses().create(req2).await },
    async { c3.responses().create(req3).await },
);
// 926ms for all 3 (vs 1462ms in Python)
```

## Implemented APIs

| API | Method |
|-----|--------|
| Chat Completions | `client.chat().completions().create()` / `create_stream()` |
| Responses | `client.responses().create()` / `create_stream()` / `create_stream_fc()` |
| Responses Tools | Function, WebSearch, FileSearch, CodeInterpreter, ComputerUse, Mcp, ImageGeneration |
| WebSocket | `client.ws_session()` — send / send_stream / warmup / close |
| Hedged | `hedged_request()` / `hedged_request_n()` / `speculative()` |
| Embeddings | `client.embeddings().create()` |
| Models | `client.models().list()` / `retrieve()` / `delete()` |
| Images | `client.images().generate()` / `edit()` / `create_variation()` |
| Audio | `client.audio().transcriptions()` / `translations()` / `speech()` |
| Files | `client.files().create()` / `list()` / `retrieve()` / `delete()` / `content()` |
| Fine-tuning | `client.fine_tuning().jobs().create()` / `list()` / `cancel()` / `list_events()` |
| Moderations | `client.moderations().create()` |
| Batches | `client.batches().create()` / `list()` / `retrieve()` / `cancel()` |
| Uploads | `client.uploads().create()` / `cancel()` / `complete()` |
| Pagination | `list_page()` / `list_auto()` — cursor-based, async stream |
| Assistants (beta) | Full CRUD + threads + runs + vector stores |
| Realtime (beta) | `client.beta().realtime().sessions().create()` |

## Configuration

```rust
use openai_oxide::{OpenAI, ClientConfig};

let client = OpenAI::from_env()?;                              // OPENAI_API_KEY env
let client = OpenAI::new("sk-...");                             // explicit key
let client = OpenAI::with_config(                               // full config
    ClientConfig::new("sk-...").base_url("https://...").timeout_secs(30).max_retries(3)
);
let client = OpenAI::azure(AzureConfig::new()                  // Azure
    .azure_endpoint("https://my.openai.azure.com").azure_deployment("gpt-4").api_key("...")
)?;
```

## Development

```bash
cargo test                                                      # 195 tests
cargo test --features live-tests                                # real API
cargo clippy -- -D warnings                                     # lint
cargo run --example benchmark --features responses --release    # benchmark
python3 examples/bench_python.py                                # Python comparison
```

## Used in

- **[sgr-agent](https://github.com/fortunto2/rust-code)** — LLM agent framework with structured output, function calling, agent loops, and 3-backend support (oxide / genai / async-openai). `openai-oxide` is the default backend for OpenAI models via `Llm::new()`. [![crates.io](https://img.shields.io/crates/v/sgr-agent.svg)](https://crates.io/crates/sgr-agent)
- **[rust-code](https://github.com/fortunto2/rust-code)** — AI-powered TUI coding agent. Uses `sgr-agent` + `openai-oxide` for GPT models.

## See also

- [openai-python](https://github.com/openai/openai-python) — Official Python SDK (our benchmark baseline)
- [async-openai](https://github.com/64bit/async-openai) — Alternative Rust client (mature, 1800+ stars)
- [genai](https://github.com/jeremychone/rust-genai) — Multi-provider Rust client (Gemini, Anthropic, OpenAI)

## License

[MIT](LICENSE)
