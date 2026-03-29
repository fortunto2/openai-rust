<p align="center">
  <img src="docs/logo.png" alt="openai-oxide" width="480">
  <br>
  <p align="center">
    A high-performance, feature-complete OpenAI client for <strong>Rust</strong>, <strong>Node.js</strong>, and <strong>Python</strong>.<br>Designed for agentic workflows, low-latency streaming, and WebAssembly.
  </p>
  <p align="center">
    <a href="https://crates.io/crates/openai-oxide"><img src="https://img.shields.io/crates/v/openai-oxide.svg" alt="crates.io"></a>
    <a href="https://www.npmjs.com/package/openai-oxide"><img src="https://img.shields.io/npm/v/openai-oxide.svg" alt="npm"></a>
    <a href="https://pypi.org/project/openai-oxide/"><img src="https://img.shields.io/pypi/v/openai-oxide.svg" alt="PyPI"></a>
    <a href="https://docs.rs/openai-oxide"><img src="https://docs.rs/openai-oxide/badge.svg" alt="docs.rs"></a>
    <a href="https://fortunto2.github.io/openai-oxide/"><img src="https://img.shields.io/badge/docs-mdbook-blue.svg" alt="Guide"></a>
    <a href="https://socket.dev/npm/package/openai-oxide"><img src="https://badge.socket.dev/npm/package/openai-oxide" alt="Socket"></a>
    <a href="https://github.com/fortunto2/openai-oxide/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT"></a>
    <a href="https://github.com/fortunto2/openai-oxide"><img src="https://img.shields.io/github/stars/fortunto2/openai-oxide?style=social" alt="GitHub stars"></a>
  </p>
</p>

`openai-oxide` implements the full [Responses API](https://platform.openai.com/docs/api-reference/responses), [Chat Completions](https://platform.openai.com/docs/api-reference/chat), and 20+ other endpoints. It introduces performance primitives like **persistent WebSockets**, **hedged requests**, **early-parsing for function calls**, and **type-safe Structured Outputs** — features previously unavailable in the Rust ecosystem.

## Why openai-oxide?

We built `openai-oxide` to squeeze every millisecond out of the OpenAI API.

- **Structured Outputs — `parse::<T>()`:** Auto-generates JSON schema from Rust types via `schemars` and deserializes the response in one call — `parse::<MyStruct>()`. Works with Chat and Responses APIs. Node (Zod) and Python (Pydantic v2) bindings included.
- **Stream Helpers:** High-level `ChatStreamEvent` with automatic text/tool-call accumulation, typed `ContentDelta`/`ToolCallDone` events, `get_final_completion()`, and `current_content()` snapshots. No manual chunk stitching.
- **Streaming:** SSE parser with anti-buffering headers (`Accept: text/event-stream`, `Cache-Control: no-cache`). On mock benchmarks, per-chunk processing is 2.6x faster than the official JS SDK (283µs vs 742µs for 114 chunks, p<0.001).
- **WebSocket Mode:** Persistent `wss://` connection for the Responses API. Measured 29-44% faster on multi-turn benchmarks vs HTTP (warm connections).
- **Stream FC Early Parse:** Yields function calls the exact moment `arguments.done` is emitted, letting you start executing local tools before the overall response finishes.
- **Hardware-Accelerated JSON (`simd`):** Opt-in AVX2/NEON vector instructions for parsing massive agent histories and complex tool calls in microseconds.
- **Hedged Requests:** Send redundant requests and cancel the slower ones. Trades extra tokens for lower tail latency (technique from Google's "The Tail at Scale").
- **Webhook Verification:** HMAC-SHA256 signature verification with timestamp replay protection — production-ready webhook handling out of the box.
- **HTTP Tuning:** gzip, TCP_NODELAY, HTTP/2 keep-alive with adaptive window, connection pooling — enabled by default. Neither async-openai nor genai set these.
- **WASM First-Class:** Compiles to `wasm32-unknown-unknown` without dropping features. Streaming, retries, and early-parsing work flawlessly in Cloudflare Workers and browsers. [Live demo](https://cloudflare-worker-dioxus.nameless-sunset-8f24.workers.dev).

### WebSocket Mode for Agent Loops

In multi-turn agent loops, WebSocket mode avoids per-request HTTP/2 framing and header overhead. Both HTTP and WebSocket reuse the same TCP+TLS connection (no per-request handshake), but WebSocket eliminates HTTP/2 frame negotiation.

```text
Standard Client (HTTP/2, warm connection)
Request 1 (ls)   : [HTTP/2 frames] -> [Wait TTFT] -> [Wait Done] -> [Parse] -> [Exec Tool]
Request 2 (cat)  : [HTTP/2 frames] -> [Wait TTFT] -> [Wait Done] -> [Parse] -> [Exec Tool]

openai-oxide (WebSocket + Early Parse)
Connection       : [TLS Handshake] (Done once)
Request 1 (ls)   : [Send JSON] -> [Wait TTFT] -> [Exec Tool Early!]
Request 2 (cat)  :             [Send JSON] -> [Wait TTFT] -> [Exec Tool Early!]
```

Measured savings on warm connections (gpt-5.4, median of medians):
- **Plain text:** 710ms WS vs 1011ms HTTP (29% faster)
- **Multi-turn (2 reqs):** 1425ms vs 2362ms (40% faster)
- **Rapid-fire (5 calls):** 3227ms vs 5807ms (44% faster)

The gap likely reflects both reduced framing overhead and different server-side routing for the WebSocket endpoint. Early parse (yielding tool calls before `[DONE]`) provides additional savings in streaming mode.

---

## Installation

### Rust
```bash
cargo add openai-oxide tokio --features tokio/full
```

### Node.js / TypeScript
```bash
npm install openai-oxide
# or
pnpm add openai-oxide
# or
yarn add openai-oxide
```
Supported platforms: macOS (x64, arm64), Linux (x64, arm64, glibc & musl), Windows (x64).

### Python
```bash
pip install openai-oxide
# or
uv pip install openai-oxide
```

| Package | Registry | Link |
|---------|----------|------|
| `openai-oxide` | crates.io | [crates.io/crates/openai-oxide](https://crates.io/crates/openai-oxide) |
| `openai-oxide` | npm | [npmjs.com/package/openai-oxide](https://www.npmjs.com/package/openai-oxide) |
| `openai-oxide` | PyPI | [pypi.org/project/openai-oxide](https://pypi.org/project/openai-oxide/) |
| `openai-oxide-macros` | crates.io | [crates.io/crates/openai-oxide-macros](https://crates.io/crates/openai-oxide-macros) |

---

## Quick Start

### Rust

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

### Node.js

```javascript
const { Client } = require("openai-oxide");

const client = new Client(); // Uses OPENAI_API_KEY
const text = await client.createText("gpt-5.4-mini", "Hello from Node!");
console.log(text);
```

### Python

```python
import asyncio, json
from openai_oxide import Client

async def main():
    client = Client()  # Uses OPENAI_API_KEY
    res = json.loads(await client.create("gpt-5.4-mini", "Hello from Python!"))
    print(res["text"])

asyncio.run(main())
```

---

## Benchmarks

- **Environment:** macOS (M-series), release mode.
- **Model:** `gpt-5.4` via the official OpenAI API.
- **Protocol:** TLS + HTTP/2 with connection pooling (warm connections).
- **Methodology:** 5 iterations per test, median. Date: 2026-03-29.

### Rust Ecosystem

`openai-oxide` vs [`async-openai`](https://crates.io/crates/async-openai) 0.34 vs [`genai`](https://crates.io/crates/genai) 0.6-beta. All via Responses API (genai uses Chat API — it's a multi-provider adapter).

| Test | `openai-oxide` | `async-openai` | `genai` |
| :--- | :--- | :--- | :--- |
| **Plain text** | 1068ms | 995ms | **845ms** |
| **Structured output** | 1430ms | N/A | N/A |
| **Function calling** | 1153ms | 1108ms | N/A |
| **Multi-turn (2 reqs)** | 2266ms | 1866ms | N/A |
| **Streaming TTFT** | **584ms** | N/A | N/A |
| **Parallel 3x (fan-out)** | **1057ms** | N/A | N/A |

On single HTTP calls, all three SDKs are **within API variance** — server latency (800-1100ms) is 100-1000x larger than SDK overhead. genai is slightly faster on plain text because it skips full response deserialization (extracts text only).

**Where oxide stands out** is features, not single-call speed:

| Feature | `openai-oxide` | `async-openai` 0.34 | `genai` 0.6 |
| :--- | :---: | :---: | :---: |
| Streaming SSE (584ms TTFT) | **yes** | no | no |
| WebSocket Responses API | **yes** | no | no |
| Structured `parse::<T>()` | **yes** | no | no |
| Parallel fan-out (1057ms) | **yes** | manual | manual |
| WASM support | **yes** | no | no |
| Node.js / Python bindings | **yes** | no | no |
| Hedged requests | **yes** | no | no |
| Stream FC early parse | **yes** | no | no |

*Reproduce: `cd benchmarks/rust-compare && cargo run --release`*

<br>

<!-- BENCH:python:START -->
### Python Ecosystem (`openai-oxide-python` vs `openai`)

`openai-oxide` wins **10/12** tests. Native PyO3 bindings vs `openai` (openai 2.29.0).

| Test | `openai-oxide` | `openai` | Winner |
| :--- | :--- | :--- | :--- |
| **Plain text** | **845ms** | 997ms | OXIDE (+15%) |
| **Structured output** | **1367ms** | 1379ms | OXIDE (+1%) |
| **Function calling** | **1195ms** | 1230ms | OXIDE (+3%) |
| **Multi-turn (2 reqs)** | **2260ms** | 3089ms | OXIDE (+27%) |
| **Web search** | **3157ms** | 3499ms | OXIDE (+10%) |
| **Nested structured** | 5377ms | **5339ms** | python (+1%) |
| **Agent loop (2-step)** | **4570ms** | 5144ms | OXIDE (+11%) |
| **Rapid-fire (5 calls)** | **5667ms** | 6136ms | OXIDE (+8%) |
| **Prompt-cached** | **4425ms** | 5564ms | OXIDE (+20%) |
| **Streaming TTFT** | **626ms** | 638ms | OXIDE (+2%) |
| **Parallel 3x** | 1184ms | **1090ms** | python (+9%) |
| **Hedged (2x race)** | **893ms** | 995ms | OXIDE (+10%) |

*median of medians, 3×5 iterations. Model: gpt-5.4. Date: 2026-03-24. Not re-measured since — results may have shifted.*

Reproduce: `cd openai-oxide-python && uv run python ../examples/bench_python.py`
<!-- BENCH:python:END -->

---

<!-- BENCH:node:START -->
### Node.js Ecosystem (`openai-oxide` vs `openai`)

Live API results are **noisy** — single-call differences are within API variance (n=10). Neither SDK consistently wins on single requests. The real advantages show on streaming, parallel, and WebSocket workloads.

| Test | `openai-oxide` | `openai` | Notes |
| :--- | :--- | :--- | :--- |
| **Plain text** | ~1050ms | ~1000ms | within noise |
| **Structured output** | ~1400ms | ~1350ms | within noise |
| **Function calling** | ~1220ms | ~1210ms | within noise |
| **Multi-turn (2 reqs)** | ~2200ms | ~2500ms | oxide +12% (varies) |
| **Rapid-fire (5 calls)** | ~4900ms | ~4800ms | within noise |
| **Streaming TTFT** | **~600ms** | ~670ms | oxide consistently faster |
| **Parallel 3x** | **~1000ms** | ~1060ms | oxide consistently faster |
| **WebSocket hot pair** | **~2300ms** | N/A | no official equivalent |

*10 iterations, median. Model: gpt-5.4. Date: 2026-03-29. Results vary between runs.*

Reproduce: `cd openai-oxide-node && BENCH_ITERATIONS=10 node examples/bench_node.js`
<!-- BENCH:node:END -->

### SDK Overhead (synthetic, Node.js)

The live benchmarks above include network latency and model inference, which adds noise.
To isolate **pure SDK overhead**, we also run a synthetic benchmark with a localhost mock
server (zero network, zero inference). Fixtures are captured from a real coding agent session
(320 messages, 42 tools, 718KB request body).

| Test | `openai-oxide` | `openai` npm | oxide faster | sig |
| :--- | :--- | :--- | :--- | :--- |
| Tiny req → Tiny resp | 112µs | 431µs | **+74%** | *** |
| Tiny req → Structured 5KB | 172µs | 393µs | **+56%** | *** |
| Medium 150KB → Tool call | 839µs | 1.2ms | **+29%** | *** |
| Heavy 657KB → Real agent resp | 2.3ms | 2.7ms | **+16%** | *** |
| SSE stream (114 real chunks) | 312µs | 783µs | **+60%** | *** |
| Agent 20x sequential (tiny) | 1.9ms | 4.6ms | **+58%** | *** |
| Agent 10x sequential (heavy) | 22.7ms | 26.0ms | **+13%** | *** |

*50 iterations, 20 warmup, `--expose-gc`, Welch's t-test — all p<0.001. Date: 2026-03-29.*

**oxide wins 12/12 mock tests** (was 10/12 before auto fast-path wrapper).

The wrapper auto-detects large payloads (>8KB) and routes through `JSON.stringify` → Rust, bypassing the napi object→Value copy. This fixed the heavy-payload regression: 657KB went from -7% (slower) to **+16%** (faster).

**Where oxide is faster:** everything on mock — 13-74% depending on payload size. SSE streaming 60% faster (zero-copy parser). Agent loops compound: 20 tiny calls save 2.7ms, 10 heavy calls save 3.3ms.

**Where it doesn't matter:** single API calls to OpenAI with 200ms-2s latency. SDK overhead (0.1-3ms) is <1% of wall time. Live benchmarks show no consistent winner on single requests — API variance dominates. Real advantages are streaming TTFT, parallel fan-out, and WebSocket mode.

Reproduce: `node --expose-gc benchmarks/bench_science.js`

---

## Python Usage

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


### `#[openai_tool]` Macro
Auto-generate JSON schemas for your functions.

```rust
use openai_oxide_macros::openai_tool;

#[openai_tool(description = "Get the current weather")]
fn get_weather(location: String, unit: Option<String>) -> String {
    format!("Weather in {location}")
}

// The macro generates `get_weather_tool()` which returns the `serde_json::Value` schema
let tool = get_weather_tool();
```

### Node.js / TypeScript Native Bindings
Thanks to NAPI-RS, we now provide lightning-fast Node.js bindings that execute requests and stream events directly from Rust into the V8 event loop without pure-JS blocking overhead.

```javascript
const { Client } = require("openai-oxide");

(async () => {
  const client = new Client();
  const session = await client.wsSession();
  const res = await session.send("gpt-5.4-mini", "Say hello to Rust from Node!");
  console.log(res);
  await session.close();
})();
```

At the moment, the Node bindings expose Chat Completions, Responses, streaming helpers, and WebSocket sessions. The full API matrix below refers to the Rust core crate.


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

Check out our **[Cloudflare Worker Examples](https://github.com/fortunto2/openai-oxide/tree/main/examples/cloudflare-worker-dioxus)** showcasing a Full-Stack Rust app with a Dioxus frontend and a Cloudflare Worker Durable Object backend holding a WebSocket connection to OpenAI.

---

## OpenAI Docs → openai-oxide

Use OpenAI's official guides — the same concepts apply directly. Here's how each maps to `openai-oxide`:

| OpenAI Guide | Rust | Node.js | Python |
|---|---|---|---|
| [Chat Completions](https://platform.openai.com/docs/guides/chat-completions) | `client.chat().completions().create()` | `client.createChatCompletion({...})` | `await client.create(model, input)` |
| [Responses API](https://platform.openai.com/docs/api-reference/responses) | `client.responses().create()` | `client.createText(model, input)` | `await client.create(model, input)` |
| [Streaming](https://platform.openai.com/docs/api-reference/streaming) | `client.responses().create_stream()` | `client.createStream({...}, cb)` | `await client.create_stream(model, input)` |
| [Function Calling](https://platform.openai.com/docs/guides/function-calling) | `client.responses().create_stream_fc()` | `client.createResponse({model, input, tools})` | `await client.create_with_tools(model, input, tools)` |
| [Structured Output](https://platform.openai.com/docs/guides/structured-outputs) | `client.chat().completions().parse::<T>()` | `client.createChatParsed(req, name, schema)` | `await client.create_parsed(model, input, PydanticModel)` |
| [Embeddings](https://platform.openai.com/docs/guides/embeddings) | `client.embeddings().create()` | via `createResponse()` raw | via `create_raw()` |
| [Image Generation](https://platform.openai.com/docs/guides/images) | `client.images().generate()` | via `createResponse()` raw | via `create_raw()` |
| [Text-to-Speech](https://platform.openai.com/docs/guides/text-to-speech) | `client.audio().speech().create()` | via `createResponse()` raw | via `create_raw()` |
| [Speech-to-Text](https://platform.openai.com/docs/guides/speech-to-text) | `client.audio().transcriptions().create()` | via `createResponse()` raw | via `create_raw()` |
| [Fine-tuning](https://platform.openai.com/docs/guides/fine-tuning) | `client.fine_tuning().jobs().create()` | via `createResponse()` raw | via `create_raw()` |
| [Conversations](https://platform.openai.com/docs/guides/conversational-agents) | `client.conversations()` CRUD + items | via raw | via raw |
| [Video Generation (Sora)](https://developers.openai.com/api/docs/guides/video-generation) | `client.videos()` create/edit/extend/remix | via raw | via raw |
| [Webhooks](https://platform.openai.com/docs/guides/webhooks) | `Webhooks::new(secret).verify()` | — | — |
| [Realtime API](https://platform.openai.com/docs/guides/realtime) | `client.ws_session()` | `client.wsSession()` | — |
| [Assistants](https://platform.openai.com/docs/assistants) | `client.beta().assistants()` | via raw | via raw |

> **Tip:** Parameter names match the official Python SDK exactly. If OpenAI docs show `model="gpt-5.4"`, use `.model("gpt-5.4")` in Rust or `{model: "gpt-5.4"}` in Node.js.
>
> **Note:** Node.js and Python bindings have typed helpers for Responses, Chat, Streaming, Function Calling, and Structured Output. All other endpoints are available via the raw JSON methods (`createResponse()` / `create_raw()`) which accept any OpenAI API request body.

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


## Structured Outputs

Get typed, validated responses directly from the model — no manual JSON parsing.

### Rust (feature: `structured`)

```rust
use openai_oxide::parsing::ParsedChatCompletion;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct MathAnswer {
    steps: Vec<String>,
    final_answer: String,
}

// Chat API
let result: ParsedChatCompletion<MathAnswer> = client.chat().completions()
    .parse::<MathAnswer>(request).await?;
println!("{}", result.parsed.unwrap().final_answer);

// Responses API
let result = client.responses().parse::<MathAnswer>(request).await?;
```

The SDK auto-generates a strict JSON schema from your Rust types, sends it as `response_format` (Chat) or `text.format` (Responses), and deserializes the response. The API guarantees the output matches your schema.

### Node.js

```javascript
// With raw JSON schema
const { parsed } = await client.createChatParsed(request, "MathAnswer", jsonSchema);

// With Zod (optional: npm install zod-to-json-schema)
const { zodParse } = require("openai-oxide/zod");
const Answer = z.object({ steps: z.array(z.string()), final_answer: z.string() });
const { parsed } = await zodParse(client, request, Answer);
```

### Python (Pydantic v2)

```python
from pydantic import BaseModel

class MathAnswer(BaseModel):
    steps: list[str]
    final_answer: str

result = await client.create_parsed("gpt-5.4-mini", "What is 2+2?", MathAnswer)
print(result.final_answer)  # Typed Pydantic instance, not dict
```

---

## Stream Helpers

High-level streaming with typed events and automatic delta accumulation.

```rust
use openai_oxide::stream_helpers::ChatStreamEvent;

// Option 1: Just get the final result
let stream = client.chat().completions().create_stream_helper(request).await?;
let completion = stream.get_final_completion().await?;

// Option 2: React to typed events
let mut stream = client.chat().completions().create_stream_helper(request).await?;
while let Some(event) = stream.next().await {
    match event? {
        ChatStreamEvent::ContentDelta { delta, snapshot } => {
            print!("{delta}");  // Print as it arrives
            // snapshot = full text accumulated so far
        }
        ChatStreamEvent::ToolCallDone { name, arguments, .. } => {
            // Arguments are complete — execute the tool
            execute_tool(&name, &arguments).await;
        }
        ChatStreamEvent::ContentDone { content } => {
            // Final text, fully assembled
        }
        _ => {}
    }
}
```

No manual chunk stitching. Tool call arguments are automatically assembled from index-based deltas.

---

## Webhook Verification

Verify OpenAI webhook signatures (feature: `webhooks`).

```rust
use openai_oxide::resources::webhooks::Webhooks;

let wh = Webhooks::new("whsec_your_secret")?;
let event: MyEvent = wh.unwrap(payload, signature_header, timestamp_header)?;
```

---

## Built With AI

This crate was built in days, not months — using [Claude Code](https://claude.ai/claude-code) with a harness engineering approach: pre-commit quality gates, OpenAPI spec as ground truth, official Python SDK as reference. Planning and code intelligence via [solo-factory](https://github.com/fortunto2/solo-factory) skills and [solograph](https://github.com/fortunto2/solograph) MCP server.

---

## Roadmap

Our goal is to make `openai-oxide` the universal engine for all LLM integrations across the entire software stack.

- [x] **Rust Core**: Fully typed, high-performance client (Chat, Responses, Realtime, Assistants).
- [x] **WASM Support**: First-class Cloudflare Workers & browser execution.
- [x] **Python Bindings**: Native PyO3 integration published on PyPI.
- [ ] **Tauri Integrations**: Dedicated examples/guides for building AI desktop apps with Tauri + WebSockets.
- [ ] **HTMX + Axum Examples**: Showcasing how to stream LLM responses directly to HTML with zero-JS frontends.
- [ ] **Swift Bindings (UniFFI)**: Native iOS/macOS integration for Apple ecosystem developers.
- [ ] **Kotlin Bindings (UniFFI)**: Native Android integration via JNI.
- [x] **Node.js/TypeScript Bindings (NAPI-RS)**: Native Node.js bindings for the TS ecosystem.

Want to help us get there? PRs and discussions are highly welcome!

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



## AI Agent Skills

This repo includes an [Agent Skill](https://agentskills.io/) — a portable knowledge pack that teaches AI coding assistants how to use `openai-oxide` correctly (gotchas, patterns, API reference).

Works with Claude Code, Cursor, GitHub Copilot, Gemini CLI, VS Code, and [30+ other agents](https://agentskills.io/).

```bash
# Context7
npx ctx7 skills search openai-oxide
npx ctx7 skills install /fortunto2/openai-oxide

# skills.sh
npx skills add fortunto2/openai-oxide
```

---

## See Also

- [openai-python](https://github.com/openai/openai-python) — Official Python SDK (our benchmark baseline)
- [async-openai](https://github.com/64bit/async-openai) — Alternative Rust client (mature, 1800+ stars)
- [genai](https://github.com/jeremychone/rust-genai) — Multi-provider Rust client (Gemini, Anthropic, OpenAI)

## License

[MIT](LICENSE)
