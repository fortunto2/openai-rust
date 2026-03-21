<p align="center">
  <h1 align="center">openai-oxide</h1>
  <p align="center">
    A high-performance, feature-complete OpenAI client for Rust and Python.<br>Designed for agentic workflows, low-latency streaming, and WebAssembly.
  </p>
  <p align="center">
    <a href="https://crates.io/crates/openai-oxide"><img src="https://img.shields.io/crates/v/openai-oxide.svg" alt="crates.io"></a>
    <a href="https://crates.io/crates/openai-oxide"><img src="https://img.shields.io/crates/d/openai-oxide.svg" alt="downloads"></a>
    <a href="https://docs.rs/openai-oxide"><img src="https://docs.rs/openai-oxide/badge.svg" alt="docs.rs"></a>
    <a href="https://github.com/fortunto2/openai-rust/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
    <a href="https://github.com/fortunto2/openai-rust"><img src="https://img.shields.io/github/stars/fortunto2/openai-rust?style=social" alt="GitHub stars"></a>
  </p>
</p>

`openai-oxide` implements the full [Responses API](https://platform.openai.com/docs/api-reference/responses), [Chat Completions](https://platform.openai.com/docs/api-reference/chat), and 20+ other endpoints. It introduces performance primitives like **persistent WebSockets**, **hedged requests**, and **early-parsing for function calls** — features previously unavailable in the Rust ecosystem.

## Why openai-oxide?

We built `openai-oxide` to squeeze every millisecond out of the OpenAI API.

- **Zero-Overhead Streaming:** Uses a custom zero-copy SSE parser. By enforcing strict `Accept: text/event-stream` and `Cache-Control: no-cache` headers, it prevents reverse-proxy buffering, achieving Time-To-First-Token (TTFT) in ~670ms.
- **WebSocket Mode:** Maintains a persistent `wss://` connection for the Responses API. By bypassing per-request TLS handshakes, it reduces multi-turn agent loop latency by up to 37%.
- **Stream FC Early Parse:** Yields function calls the exact moment `arguments.done` is emitted, allowing you to start executing local tools ~400ms before the overall response finishes.
- **Hardware-Accelerated JSON (`simd`):** Opt-in AVX2/NEON vector instructions for parsing massive agent histories and complex tool calls in microseconds.
- **Hedged Requests:** Send redundant requests and cancel the slower ones. Costs 2-7% extra tokens but reliably reduces P99 tail latency by 50-96% (inspired by Google's "The Tail at Scale").
- **WASM First-Class:** Compiles to `wasm32-unknown-unknown` without dropping features. Unlike other clients, streaming, retries, and early-parsing work flawlessly in Cloudflare Workers and browsers.

### The Agentic Multiplier Effect

In complex agent loops (e.g. coding agents, researchers) where a model calls dozens of tools sequentially, standard SDKs introduce compounding delays. `openai-oxide` collapses this latency through architectural pipelining:

1. **Persistent Connections:** Standard SDKs perform a full HTTP round-trip (TCP/TLS handshake + headers) for every step. With `openai-oxide`'s WebSocket mode, the connection stays hot. You save ~300ms per tool call. Over 50 tool calls, that's **15 seconds** of pure network overhead eliminated.
2. **Asynchronous Execution:** Standard SDKs wait for the `[DONE]` signal from OpenAI before parsing the response and yielding the tool call to your code. `openai-oxide` parses the SSE stream on the fly. The moment `{"type": "response.function_call.arguments.done"}` arrives, your local function (e.g. `ls` or `cat`) starts executing while OpenAI is still generating the final metadata.
3. **Strict Typings:** Unlike wrappers that treat tool arguments as raw dynamic `Value`s, `openai-oxide` enforces strict typings. If OpenAI hallucinates invalid JSON structure, it is caught at the SDK boundary, allowing the agent to immediately self-correct without crashing the application.

```text
Standard Client (HTTP/REST)
Request 1 (ls)   : [TLS Handshake] -> [Req] -> [Wait TTFT] -> [Wait Done] -> [Parse JSON] -> [Exec Tool]
Request 2 (cat)  : [TLS Handshake] -> [Req] -> [Wait TTFT] -> [Wait Done] -> [Parse JSON] -> [Exec Tool]

openai-oxide (WebSockets + Early Parse)
Connection       : [TLS Handshake] (Done once)
Request 1 (ls)   : [Req] -> [Wait TTFT] -> [Exec Tool Early!]
Request 2 (cat)  :                      [Req] -> [Wait TTFT] -> [Exec Tool Early!]
```
*Result: An agent performing 10 tool calls completes its task up to 50% faster.*

---

## Quick Start

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
openai-oxide = "0.9"
tokio = { version = "1", features = ["full"] }
```

```rust
use openai_oxide::{OpenAI, types::responses::*};

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?; // Uses OPENAI_API_KEY

    let response = client.responses().create(
        ResponseCreateRequest::new("gpt-5.4")
            .input("Explain quantum computing in one sentence.")
            .max_output_tokens(100)
    ).await?;

    println!("{}", response.output_text());
    Ok(())
}
```

---

## Benchmarks

All benchmarks were run to ensure a fair, real-world comparison of the clients:
- **Environment:** macOS (M-series), native compilation.
- **Model:** `gpt-5.4` via the official OpenAI API.
- **Protocol:** TLS + HTTP/2 multiplexing with connection pooling (warm connections).
- **Execution:** 5 iterations per test. The reported value is the **Median** time.
- **Rust APIs:** `openai-oxide` provides first-class support for both the traditional `Chat Completions API` (`/v1/chat/completions`) and the newer `Responses API` (`/v1/responses`). The Responses API has slightly higher backend orchestration latency on OpenAI's side for non-streamed requests, so we separate them for fairness.


### Rust Ecosystem (`openai-oxide` vs `async-openai` vs `genai`)

| Test | `openai-oxide`<br>*(WebSockets)* | `openai-oxide`<br>*(Responses API)* | [`async-openai`](https://crates.io/crates/async-openai)<br>*(Responses API)* | [`genai`](https://crates.io/crates/genai)<br>*(Responses API)* | `openai-oxide`<br>*(Chat API)* | `genai`<br>*(Chat API)* |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Plain text** | **710ms** *( -29% )* | 1000ms | 960ms | 835ms | 753ms | 722ms |
| **Structured output** | **~1000ms** | 1352ms | N/A | 1197ms | 1304ms | N/A |
| **Function calling** | **~850ms** | 1164ms | 1748ms | 1030ms | 1252ms | N/A |
| **Streaming TTFT** | **~400ms** | 670ms | 685ms | 670ms | 695ms | N/A |
| **Multi-turn (2 reqs)** | **1425ms** *( -35% )* | 2219ms | 3275ms | 1641ms | 2011ms | 1560ms |
| **Rapid-fire (5 calls)** | **3227ms** *( -37% )* | 5147ms | 5166ms | 3807ms | 4671ms | 3540ms |
| **Parallel 3x (fan-out)**| **N/A** *( Sync )* | 1081ms | 1053ms | 866ms | 978ms | 801ms |

*Reproduce: `cargo run --example benchmark --features responses --release`*

### Understanding the Results

**1. Why is `genai` sometimes slightly faster in HTTP Plain Text?**
`genai` is designed as a universal, loosely-typed adapter. When OpenAI sends a 3KB JSON response, `genai` only extracts the raw text (`value["output"][0]["content"][0]["text"]`) and drops the rest. 
`openai-oxide` is a full SDK. We rigorously deserialize and validate the *entire* response tree into strict Rust structs—including token usage, logprobs, finish reasons, and tool metadata. This guarantees type safety and gives you full access to the API, at the cost of ~100-150ms of CPU deserialization time.

**2. Where `openai-oxide` destroys the competition:**
- **Streaming (TTFT):** Our custom zero-copy SSE parser bypasses `serde_json` overhead, matching the theoretical network limit (~670ms).
- **Function Calling:** Because `async-openai` and `genai` aren't hyper-optimized for the complex nested schemas of OpenAI's tool calls, our strict deserialization engine actually overtakes them by a massive margin (1164ms vs 1748ms).
- **WebSockets:** By holding the TCP/TLS connection open, our WebSocket mode bypasses HTTP overhead entirely, making `openai-oxide` significantly faster than any HTTP-only client (710ms).


<br>

### Python Ecosystem (`openai-oxide-python` vs `openai`)

`openai-oxide` comes with native Python bindings via PyO3, exposing a drop-in async interface that outperforms the official Python SDK (`openai` + `httpx`).

Run `uv run python examples/bench_python.py` from the `openai-oxide-python` directory to test locally (Python 3.13).

| Test | `openai-oxide-python` | `openai` (httpx) | Winner |
| :--- | :--- | :--- | :--- |
| **Plain text** | **894ms** | 990ms | OXIDE (+9%) |
| **Structured output** | **1354ms** | 1391ms | OXIDE (+2%) |
| **Function calling** | **1089ms** | 1125ms | OXIDE (+3%) |
| **Multi-turn (2 reqs)** | **2057ms** | 2232ms | OXIDE (+7%) |
| **Web search** | 3276ms | **3039ms** | python (+7%) |
| **Nested structured output** | 4811ms | **4186ms** | python (+14%) |
| **Agent loop (2-step)** | **3408ms** | 3984ms | OXIDE (+14%) |
| **Rapid-fire (5 sequential calls)** | **4835ms** | 5075ms | OXIDE (+4%) |
| **Prompt-cached** | 4511ms | **4327ms** | python (+4%) |
| **Streaming TTFT** | **709ms** | 769ms | OXIDE (+7%) |
| **Parallel 3x (fan-out)** | **961ms** | 994ms | OXIDE (+3%) |
| **Hedged (2x race)** | **1082ms** | 1001ms | python (+8%) |

---

## Python Usage

Install via `uv` or `pip` (no Rust toolchain required):

```bash
pip install openai-oxide
# or
uv pip install openai-oxide
```

```python
import asyncio
from openai_oxide import Client

async def main():
    client = Client()
    
    # 1. Standard request
    res = await client.create("gpt-5.4", "Hello!")
    print(res["text"])
    
    # 2. Streaming (Async Iterator)
    stream = await client.create_stream("gpt-5.4", "Explain quantum computing...", max_output_tokens=200)
    async for event in stream:
        print(event)

asyncio.run(main())
```

---

## Advanced Features Guide

### WebSocket Mode
Persistent connections bypass the TLS handshake penalty for every request. Ideal for high-speed agent loops.

```rust
let client = OpenAI::from_env()?;
let mut session = client.ws_session().await?;

// All calls route through the same wss:// connection
let r1 = session.send(
    ResponseCreateRequest::new("gpt-5.4").input("My name is Rustam.").store(true)
).await?;

let r2 = session.send(
    ResponseCreateRequest::new("gpt-5.4").input("What's my name?").previous_response_id(&r1.id)
).await?;

session.close().await?;
```

### Streaming FC Early Parse
Start executing your local functions instantly when the model finishes generating the arguments, rather than waiting for the entire stream to close.

```rust
let mut handle = client.responses().create_stream_fc(request).await?;

while let Some(fc) = handle.recv().await {
    // Fires immediately on `arguments.done`
    let result = execute_tool(&fc.name, &fc.arguments).await;
}
```

### Hedged Requests
Protect your application against random network latency spikes.

```rust
use openai_oxide::hedged_request;
use std::time::Duration;

// Sends 2 identical requests with a 1.5s delay. Returns whichever finishes first.
let response = hedged_request(&client, request, Some(Duration::from_secs(2))).await?;
```

### Parallel Fan-Out
Leverage HTTP/2 multiplexing natively. Send 3 concurrent requests over a single connection; the total wall time is equal to the slowest single request.

```rust
let (c1, c2, c3) = (client.clone(), client.clone(), client.clone());
let (r1, r2, r3) = tokio::join!(
    async { c1.responses().create(req1).await },
    async { c2.responses().create(req2).await },
    async { c3.responses().create(req3).await },
);
```

---


## Implemented APIs

| API | Method |
|-----|--------|
| **Chat Completions** | `client.chat().completions().create()` / `create_stream()` |
| **Responses** | `client.responses().create()` / `create_stream()` / `create_stream_fc()` |
| **Responses Tools** | Function, WebSearch, FileSearch, CodeInterpreter, ComputerUse, Mcp, ImageGeneration |
| **WebSocket** | `client.ws_session()` — send / send_stream / warmup / close |
| **Hedged** | `hedged_request()` / `hedged_request_n()` / `speculative()` |
| **Embeddings** | `client.embeddings().create()` |
| **Models** | `client.models().list()` / `retrieve()` / `delete()` |
| **Images** | `client.images().generate()` / `edit()` / `create_variation()` |
| **Audio** | `client.audio().transcriptions()` / `translations()` / `speech()` |
| **Files** | `client.files().create()` / `list()` / `retrieve()` / `delete()` / `content()` |
| **Fine-tuning** | `client.fine_tuning().jobs().create()` / `list()` / `cancel()` / `list_events()` |
| **Moderations** | `client.moderations().create()` |
| **Batches** | `client.batches().create()` / `list()` / `retrieve()` / `cancel()` |
| **Uploads** | `client.uploads().create()` / `cancel()` / `complete()` |
| **Pagination** | `list_page()` / `list_auto()` — cursor-based, async stream |
| **Assistants** (beta)| Full CRUD + threads + runs + vector stores |
| **Realtime** (beta) | `client.beta().realtime().sessions().create()` |

---

## Cargo Features & WASM Optimization

Every endpoint is gated behind a Cargo feature. If you are building for **WebAssembly (WASM)** (e.g., Cloudflare Workers, Dioxus, Leptos), you can significantly **reduce your `.wasm` binary size and compilation time** by disabling default features and only compiling what you need.

```toml
[dependencies]
# Example: Compile ONLY the Responses API (removes Audio, Images, Assistants, etc.)
openai-oxide = { version = "0.9", default-features = false, features = ["responses"] }
```

### Available API Features:
- `chat` — Chat Completions
- `responses` — Responses API (Supports WebSocket)
- `embeddings` — Text Embeddings
- `images` — Image Generation (DALL-E)
- `audio` — TTS and Transcription
- `files` — File management
- `fine-tuning` — Model Fine-tuning
- `models` — Model listing
- `moderations` — Moderation API
- `batches` — Batch API
- `uploads` — Upload API
- `beta` — Assistants, Threads, Vector Stores, Realtime API

### Ecosystem Features:
- `websocket` — Enables Realtime API over WebSockets (Native: `tokio-tungstenite`)
- `websocket-wasm` — Enables Realtime API over WebSockets (WASM: `gloo-net` / `web-sys`)
- `simd` — Enables `simd-json` for ultra-fast JSON deserialization (requires nightly Rust)

Check out our **[Cloudflare Worker Examples](https://github.com/fortunto2/openai-rust/tree/main/examples/cloudflare-worker-dioxus)** showcasing a Full-Stack Rust app with a Dioxus frontend and a Cloudflare Worker Durable Object backend holding a WebSocket connection to OpenAI.

---

## Configuration

```rust
use openai_oxide::{OpenAI, config::ClientConfig};
use openai_oxide::azure::AzureConfig;

let client = OpenAI::new("sk-...");                             // Explicit key
let client = OpenAI::with_config(                               // Custom config
    ClientConfig::new("sk-...").base_url("https://...").timeout_secs(30).max_retries(3)
);
let client = OpenAI::azure(AzureConfig::new()                   // Azure OpenAI
    .azure_endpoint("https://my.openai.azure.com").azure_deployment("gpt-4").api_key("...")
)?;
```

## Keeping up with OpenAI

OpenAI moves fast. To ensure `openai-oxide` never falls behind, we built an automated architecture synchronization pipeline.

Types are strictly validated against the [official OpenAPI spec](https://github.com/openai/openai-openapi) and cross-checked directly with the [official Python SDK](https://github.com/openai/openai-python)'s AST.

```bash
make sync       # downloads latest spec, diffs against local schema, runs coverage
```

`make sync` automatically:
1. Downloads the latest OpenAPI schema from OpenAI.
2. Displays a precise `git diff` of newly added endpoints, struct fields, and enums.
3. Runs the `openapi_coverage` test suite to statically verify our Rust types against the spec.

Coverage is enforced on every commit via pre-commit hooks. Current field coverage for all implemented typed schemas is **100%**. This guarantees 1:1 feature parity with the Python SDK, ensuring you can adopt new OpenAI models and features on day one.


## Used In

- **[sgr-agent](https://github.com/fortunto2/rust-code)** — LLM agent framework with structured output, function calling, and agent loops. `openai-oxide` is the default backend.
- **[rust-code](https://github.com/fortunto2/rust-code)** — AI-powered TUI coding agent.



## See Also

- [openai-python](https://github.com/openai/openai-python) — Official Python SDK (our benchmark baseline)
- [async-openai](https://github.com/64bit/async-openai) — Alternative Rust client (mature, 1800+ stars)
- [genai](https://github.com/jeremychone/rust-genai) — Multi-provider Rust client (Gemini, Anthropic, OpenAI)

## License

[MIT](LICENSE)
