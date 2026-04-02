---
title: "I ported the OpenAI Python SDK to Rust in 5 days with Claude Code. Here's what I learned."
published: false
tags: rust, openai, ai, webassembly
canonical_url: https://github.com/fortunto2/openai-oxide
---

I needed a fast OpenAI client for a realtime voice agent project. The official Python SDK is great, but I needed Rust for WebSocket audio streaming, edge deployment to Cloudflare Workers, and sub-second latency in agentic loops.

So I ported it. 500+ commits, 5 days for the initial version, 100+ API methods. Day one (120 commits) was mostly Claude Code translating types from Python to Rust while I set up pre-commit hooks, WASM checks, and benchmarks. The rest was architecture decisions, performance tuning, Node/Python bindings, and a standalone types crate with 1100+ auto-synced types.

The result: [openai-oxide](https://github.com/fortunto2/openai-oxide), a Rust client that matches the official Python SDK's API surface, with persistent WebSockets, structured outputs, and WASM deployment that aren't available in other Rust clients.

## Why Not Just Use What Exists?

My goal was a Rust client with complete 1:1 parity with the official Python SDK. All endpoints, plus WASM deployment, persistent WebSockets for the Responses API, and structured outputs with auto-generated schemas.

The types and HTTP layer were ported from the Python SDK. But OpenAI also has a [WebSocket mode](https://platform.openai.com/docs/guides/websocket-mode) for the Responses API, a server-side feature at `wss://api.openai.com/v1/responses` where you keep one persistent connection open for multi-turn agent loops. The endpoint exists and is documented, but the official Python and Node SDKs haven't added a convenience wrapper for it yet (their WebSocket support covers only the Realtime API for audio/multimodal). We implemented the client for this endpoint directly from the OpenAI docs.

In the Rust ecosystem, [async-openai](https://crates.io/crates/async-openai) is the closest. Good type coverage and active maintenance. I actually found it after I'd mostly finished the initial version. But at the time of building, no single Rust crate offered WebSocket sessions for the Responses API, `parse::<T>()` with auto-generated JSON schema, and WASM compilation together. That's the gap we filled.

## 1100+ Types, Auto-Synced from Python SDK

Type coverage was the hardest part. OpenAI's API surface spans 24 domains with hundreds of nested types that change regularly. We solved this by building [openai-types](https://crates.io/crates/openai-types), a standalone crate auto-generated from the Python SDK via a custom `py2rust.py` tool.

```bash
make sync-types  # re-generates from ~/openai-python/src/openai/types/
```

The mechanism: `_gen.rs` files are machine-owned (overwritten on every sync), while manual `.rs` files contain hand-crafted overrides (enums, builders, Option fields) that are never touched. This gives us Python SDK parity on types without manual maintenance. When OpenAI adds a new field, `py2rust` picks it up automatically.

```rust
use openai_types::chat::ChatCompletion;
use openai_types::responses::{Response, ResponseCreateRequest};
use openai_types::shared::ReasoningEffort;
```

The types crate has zero runtime dependencies beyond `serde` and can be used on its own if you're building your own HTTP layer.

## Persistent WebSockets

Keep one `wss://` connection open for the entire agent cycle. Both HTTP and WebSocket reuse TCP+TLS connections (reqwest pools them), but the server caches context locally for WebSocket connections, keeping the previous response state in memory for faster continuations.

```rust
let mut session = client.ws_session().await?;

// All calls route through the same wss:// connection
for _ in 0..50 {
    let response = session.send(request).await?;
    // execute tool, feed result back
}

session.close().await?;
```

Our preliminary measurements (gpt-5.4, warm connections, n=5):
- **Plain text:** 710ms WS vs 1011ms HTTP (29% faster)
- **Rapid-fire (5 calls):** 3227ms vs 5807ms (44% faster)

This aligns with [OpenAI's own documentation](https://platform.openai.com/docs/guides/websocket-mode): *"For rollouts with 20+ tool calls, we have seen up to roughly 40% faster end-to-end execution."*

*Our numbers are preliminary at n=5, but the direction matches OpenAI's published benchmarks.*

## Structured Outputs Without Boilerplate

Every Rust OpenAI client supports `response_format: json_schema`. But you have to build the schema by hand:

```rust
// Other clients: manual schema construction
let schema = json!({
    "type": "object",
    "properties": {
        "answer": {"type": "string"},
        "confidence": {"type": "number"}
    },
    "required": ["answer", "confidence"],
    "additionalProperties": false
});
```

With openai-oxide, derive the schema from your types:

```rust
#[derive(Deserialize, JsonSchema)]
struct Answer {
    answer: String,
    confidence: f64,
}

let result = client.chat().completions()
    .parse::<Answer>(request).await?;

println!("{}", result.parsed.unwrap().answer);
```

One derive, both directions. The same `#[derive(JsonSchema)]` generates response schemas and tool parameter definitions. No manual JSON, no drift between types and schemas.

## SSE Streaming

Time-to-first-token matters for UX. Our SSE parser uses incremental buffered line extraction and sets standard anti-buffering headers that prevent reverse proxies from holding back chunks:

```
Accept: text/event-stream
Cache-Control: no-cache
```

Without these, Cloudflare and nginx buffer streaming responses, adding 50-200ms to TTFT.

On mock benchmarks (localhost, no network), SSE processing via our Node napi-rs bindings is 2.6x faster than the official JS SDK: 283µs vs 742µs for 114 real agent chunks (p<0.001). On live API calls, the difference is masked by 200ms+ network latency, but it compounds in agent loops with many streaming rounds.

## Stream Helpers

Raw SSE chunks require manual stitching: tracking content deltas, assembling tool call arguments by index, detecting completion. We provide typed events:

```rust
let mut stream = client.chat().completions()
    .create_stream_helper(request).await?;

while let Some(event) = stream.next().await {
    match event? {
        ChatStreamEvent::ContentDelta { delta, snapshot } => {
            print!("{delta}"); // snapshot has full text so far
        }
        ChatStreamEvent::ToolCallDone { name, arguments, .. } => {
            execute_tool(&name, &arguments).await;
        }
        _ => {}
    }
}
```

Or just get the final result: `stream.get_final_completion().await?`

## WASM Support

The entire client compiles to `wasm32-unknown-unknown` and runs in Cloudflare Workers:

```toml
[dependencies]
openai-oxide = { version = "0.11", default-features = false, features = ["chat", "responses"] }
worker = "0.7"
```

Streaming, structured outputs, and JSON request retries work in WASM. Limitations: no multipart uploads, no streaming retries (yet). [Live demo](https://cloudflare-worker-dioxus.nameless-sunset-8f24.workers.dev).

## HTTP Tuning Defaults

These are standard reqwest builder options, enabled by default in openai-oxide:

| Optimization | What it does |
|:---|:---|
| gzip compression | ~30% smaller responses |
| TCP_NODELAY | Disables Nagle's algorithm |
| HTTP/2 keep-alive (20s ping) | Prevents idle connection drops |
| HTTP/2 adaptive window | Auto-tunes flow control |
| Connection pool (4/host) | Better parallel throughput |

Will these make your API calls faster? Probably not. Server-side latency dominates. But they prevent edge cases (stale connections, buffering delays) that bite you in production. [Source](https://github.com/fortunto2/openai-oxide/blob/main/src/client.rs#L85).

## Benchmarks — What's Real and What's Noise

After many rounds of benchmarking: **on today's API latencies, SDK choice doesn't matter for single calls.** Network latency (200ms-2s) dwarfs SDK overhead (0.1-5ms). At n=5, differences under 15% are API jitter.

### Live API (gpt-5.4) — honest results

**Rust ecosystem** (n=5, median of 3 runs):

| Test | openai-oxide | async-openai | genai | Note |
|:---|:---|:---|:---|:---|
| Plain text | 1011ms | **960ms** | **835ms** | oxide slower |
| Function calling | **1192ms** | 1748ms | **1030ms** | genai fastest |
| Streaming TTFT | **645ms** | 685ms | 670ms | within noise |

No single SDK consistently wins at n=5. oxide takes function calling and streaming, genai wins plain text (it skips full deserialization).

**Node.js** (n=5, median of 3 runs):

| Test | openai-oxide | official openai | Note |
|:---|:---|:---|:---|
| Plain text | 1075ms | 1311ms | -18% |
| Structured output | 1370ms | 1765ms | -22% |
| Multi-turn (2 reqs) | 2283ms | 2859ms | -20% |
| Streaming TTFT | 534ms | 580ms | within noise |

**Python** (n=5, median of 3 runs):

| Test | openai-oxide | official openai | Note |
|:---|:---|:---|:---|
| Multi-turn (2 reqs) | **2260ms** | 3089ms | +27% |
| Prompt-cached | **4425ms** | 5564ms | +20% |
| Plain text | **845ms** | 997ms | +15% |
| Structured output | 1367ms | 1379ms | within noise |

### SDK overhead — where oxide actually shines

The interesting part is pure SDK overhead, isolated with a localhost mock server. No network, no model inference. Just request building, JSON serialization, response parsing, SSE chunk processing. Fixtures from a real coding agent session (320 messages, 42 tools, 718KB request body).

| Test | openai-oxide | official JS SDK | oxide faster | sig |
|:---|:---|:---|:---|:---|
| Tiny req → Tiny resp | 172µs | 443µs | **+61%** | *** |
| Heavy 657KB → Real resp | 4.9ms | 6.2ms | **+21%** | *** |
| SSE stream (114 chunks) | 283µs | 742µs | **+62%** | *** |
| Agent 20x sequential | 2.1ms | 5.4ms | **+61%** | *** |

*50 iterations, 20 warmup, Welch's t-test — all p<0.001.*

**What this means today:** on OpenAI's API (200ms-2s), SDK overhead is <1% of wall time. But the picture changes with fast inference providers (Cerebras, Groq, local models returning in 10-50ms) and agent farms running hundreds of parallel sessions. At those speeds, SDK overhead becomes 5-30% of wall time, and the 2-3x gap compounds.

The value right now is **API completeness** (WebSocket with connection pool, structured outputs, WASM, stream helpers), **type safety** (1100+ auto-synced types), and the trajectory: as APIs get faster, the Rust overhead advantage grows.

Full reproducible benchmarks: `node --expose-gc benchmarks/bench_science.js`

## Drop-in Replacement

For existing codebases, change one import:

**Python:**
```python
# from openai import AsyncOpenAI
from openai_oxide.compat import AsyncOpenAI

# rest of code unchanged
client = AsyncOpenAI()
r = await client.chat.completions.create(model="gpt-5.4-mini", messages=[...])
```

**Node.js:**
```javascript
// const OpenAI = require('openai');
const { OpenAI } = require('openai-oxide/compat');

// rest of code unchanged
const client = new OpenAI();
```

## How This Was Built

This started as a need for a fast OpenAI client for a realtime TTS voice agent project. The Python SDK worked, but I needed Rust for WebSocket audio streaming and edge deployment.

The whole thing (100+ API methods, typed streaming, structured outputs, WASM, Node/Python bindings) was built in a few days using [Claude Code](https://claude.ai/claude-code) and my own toolkit:

1. **Setup**: configured pre-commit hooks (tests, clippy, WASM check, secret scan), OpenAPI spec as ground truth, Python SDK source as reference
2. **Planning**: [solo-factory](https://github.com/fortunto2/solo-factory) skills (`/plan`, `/build`) with [solograph](https://github.com/fortunto2/solograph) for code intelligence (MCP server that indexes the codebase and provides semantic search)
3. **Building**: initial scaffold via Ralph Loop (autonomous agent loop), then manual refinement. Architecture decisions, API design, performance tuning.
4. **Type sync**: built `py2rust.py` to auto-convert Python Pydantic models to Rust serde structs. 1100+ types across 24 domains, two-pass resolver for cross-file references.
5. **Quality gates**: every commit runs tests + clippy + WASM compilation check + doc coverage. Pre-commit catches regressions before they land

The key insight: treat the Python SDK as a spec, not as code to port line-by-line. The agent handles mechanical translation (types, error mapping, serialization); you focus on Rust-specific wins (tagged enums, feature gates, WASM cfg).

A harder lesson: **benchmarks are treacherous.** We went through multiple rounds removing claims that weren't statistically significant at n=5. The real story is not about milliseconds on single requests. It's about what happens at scale: structured outputs with schema generation on every call, hundreds of parallel agent sessions, function calling chains with 20+ tool invocations. That's where Rust's lack of GC pauses and lower per-call overhead start to compound.

## One Crate, Every Platform

The biggest payoff from writing the core in Rust: it runs everywhere.

| Platform | Binding | Status |
|----------|---------|--------|
| Rust | native | stable |
| Node.js / TypeScript | napi-rs | stable |
| Python | PyO3 + maturin | stable |
| Browser / Cloudflare Workers | WASM | stable |
| iOS / macOS | UniFFI (Swift) | planned |
| Android | UniFFI (Kotlin) | planned |

Same HTTP tuning, WebSocket pool, streaming parser, and retry logic on every platform. No reimplementation, no behavior drift.

This also means the crate works as **agent infrastructure**. [sgr-agent](https://github.com/fortunto2/rust-code/tree/master/crates/sgr-agent) is an LLM agent framework built on openai-oxide that runs as a [TUI coding agent](https://github.com/fortunto2/rust-code) today and can compile to WASM for browser-based agents tomorrow. The same agent code, the same OpenAI layer, different targets.

## Try It

```bash
cargo add openai-oxide tokio --features tokio/full
```

- GitHub: [fortunto2/openai-oxide](https://github.com/fortunto2/openai-oxide)
- crates.io: [openai-oxide](https://crates.io/crates/openai-oxide) + [openai-types](https://crates.io/crates/openai-types)
- npm: [openai-oxide](https://www.npmjs.com/package/openai-oxide)
- PyPI: [openai-oxide](https://pypi.org/project/openai-oxide/)
- Docs: [fortunto2.github.io/openai-oxide](https://fortunto2.github.io/openai-oxide/)
