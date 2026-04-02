---
title: "I ported the OpenAI Python SDK to Rust in 5 days with Claude Code. Here's what I learned."
published: false
tags: rust, openai, ai, webassembly
canonical_url: https://github.com/fortunto2/openai-oxide
---

I needed a fast OpenAI client for a realtime voice agent project. The official Python SDK is great, but I needed Rust — for WebSocket audio streaming, edge deployment to Cloudflare Workers, and sub-second latency in agentic loops with dozens of tool calls.

So I ported it. 500+ commits, 5 days for the initial version, 100+ API methods. The first day — 120 commits — was mostly Claude Code translating types from Python to Rust while I set up pre-commit hooks, WASM checks, and benchmarks. The rest was architecture decisions, performance tuning, Node/Python bindings, and a standalone types crate with 1100+ auto-synced types.

The result: [openai-oxide](https://github.com/fortunto2/openai-oxide) — a Rust client that matches the official Python SDK's API surface, with features like persistent WebSockets, structured outputs, and WASM deployment that aren't available in other Rust clients.

## Why Not Just Use What Exists?

My goal was a Rust client with complete 1:1 parity with the official Python SDK — all endpoints, plus Rust-specific features like WASM deployment, persistent WebSockets for the Responses API, and structured outputs with auto-generated schemas.

The types and HTTP layer were ported from the Python SDK. But OpenAI also has a [WebSocket mode](https://platform.openai.com/docs/guides/websocket-mode) for the Responses API — a server-side feature at `wss://api.openai.com/v1/responses` where you keep one persistent connection open for multi-turn agent loops. The endpoint exists and is documented, but the official Python and Node SDKs haven't added a convenience wrapper for it yet (their WebSocket support covers only the Realtime API for audio/multimodal). We implemented the client for this endpoint directly from the OpenAI docs.

In the Rust ecosystem, [async-openai](https://crates.io/crates/async-openai) is the closest — good type coverage and active maintenance. I actually found it after I'd mostly finished the initial version. But at the time of building, no single Rust crate offered WebSocket sessions for the Responses API, structured outputs (`parse::<T>()` with auto-generated JSON schema), stream helpers with typed events, and WASM compilation — together in one package. That's the gap we filled.

## 1100+ Types, Auto-Synced from Python SDK

One of the biggest challenges was type coverage. OpenAI's API surface is enormous — 24 domains, hundreds of nested types that change regularly. We solved this by building [openai-types](https://crates.io/crates/openai-types), a standalone crate auto-generated from the Python SDK via a custom `py2rust.py` tool.

```bash
make sync-types  # re-generates from ~/openai-python/src/openai/types/
```

The mechanism: `_gen.rs` files are machine-owned (overwritten on every sync), while manual `.rs` files contain hand-crafted overrides (enums, builders, Option fields) that are never touched. This gives us Python SDK parity on types without manual maintenance — when OpenAI adds a new field, `py2rust` picks it up automatically.

```rust
use openai_types::chat::ChatCompletion;
use openai_types::responses::{Response, ResponseCreateRequest};
use openai_types::shared::ReasoningEffort;
```

The types crate has zero runtime dependencies beyond `serde` and can be used independently — useful if you're building your own HTTP layer or working with OpenAI's API through a different transport.

## Persistent WebSockets

Keep one `wss://` connection open for the entire agent cycle. Both HTTP and WebSocket reuse TCP+TLS connections (reqwest uses connection pooling, so there's no per-request TLS handshake either way), but WebSocket eliminates HTTP/2 frame negotiation entirely.

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

One derive, both directions — the same `#[derive(JsonSchema)]` generates response schemas and tool parameter definitions. No manual JSON, no drift between types and schemas.

## Zero-Copy SSE Streaming

Time-to-first-token matters for UX. Our SSE parser avoids intermediate allocations and sets anti-buffering headers that prevent reverse proxies from holding back chunks:

```
Accept: text/event-stream
Cache-Control: no-cache
```

Without these, Cloudflare and nginx buffer streaming responses, adding 50-200ms to TTFT.

On mock benchmarks (localhost, no network), SSE processing is 2.5x faster than the official JS SDK: 312µs vs 783µs for 114 real agent chunks (p<0.001). On live API calls, the difference is masked by 200ms+ network latency — but it compounds in agent loops with many streaming rounds.

## Stream Helpers

Raw SSE chunks require manual stitching — tracking content deltas, assembling tool call arguments by index, detecting completion. We provide typed events:

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

Streaming, structured outputs, retry logic — all work in WASM. [Live demo](https://cloudflare-worker-dioxus.nameless-sunset-8f24.workers.dev).

## HTTP Tuning Defaults

These are standard reqwest builder options, but neither async-openai nor genai enable them by default:

| Optimization | What it does |
|:---|:---|
| gzip compression | ~30% smaller responses |
| TCP_NODELAY | Disables Nagle's algorithm |
| HTTP/2 keep-alive (20s ping) | Prevents idle connection drops |
| HTTP/2 adaptive window | Auto-tunes flow control |
| Connection pool (4/host) | Better parallel throughput |

Will these make your API calls faster? Probably not noticeably — server-side latency dominates. But they prevent edge cases (stale connections, buffering delays) that bite you in production. [Source](https://github.com/fortunto2/openai-oxide/blob/main/src/client.rs#L85).

## Benchmarks — What's Real and What's Noise

We spent a lot of time on benchmarks and learned an important lesson: **on single API calls, SDK choice doesn't matter for speed.** Network latency (200ms-2s) is 100-1000x larger than SDK overhead (0.1-5ms). At n=5 with live API calls, differences under 15% are within API jitter and not statistically significant.

### Live API (gpt-5.4) — honest results

**Rust ecosystem** (n=5, median of 3 runs):

| Test | openai-oxide | async-openai | genai | Note |
|:---|:---|:---|:---|:---|
| Plain text | 1011ms | **960ms** | **835ms** | oxide slower |
| Function calling | **1192ms** | 1748ms | **1030ms** | genai fastest |
| Streaming TTFT | **645ms** | 685ms | 670ms | within noise |

On single HTTP requests, oxide is not faster than async-openai or genai. All three SDKs are within API variance.

**Node.js** (n=10, median):

| Test | openai-oxide | official openai | Note |
|:---|:---|:---|:---|
| Multi-turn (2 reqs) | **~2200ms** | ~2500ms | oxide +12% |
| Streaming TTFT | **~600ms** | ~670ms | oxide consistently faster |
| Structured output | ~1400ms | ~1350ms | within noise |
| Function calling | ~1220ms | ~1210ms | within noise |

**Python** (n=5, median of 3 runs):

| Test | openai-oxide | official openai | Note |
|:---|:---|:---|:---|
| Multi-turn (2 reqs) | **2260ms** | 3089ms | +27% |
| Prompt-cached | **4425ms** | 5564ms | +20% |
| Plain text | **845ms** | 997ms | +15% |
| Structured output | 1367ms | 1379ms | within noise |

### SDK overhead — where oxide actually shines

The interesting story is pure SDK overhead, isolated with a localhost mock server. No network, no model inference — just request building, JSON serialization, response parsing, SSE chunk processing. Fixtures from a real coding agent session (320 messages, 42 tools, 718KB request body).

| Test | openai-oxide | official JS SDK | oxide faster | sig |
|:---|:---|:---|:---|:---|
| Tiny req → Tiny resp | 112µs | 431µs | **+74%** | *** |
| Heavy 657KB → Real resp | 2.3ms | 2.7ms | **+16%** | *** |
| SSE stream (114 chunks) | 312µs | 783µs | **+60%** | *** |
| Agent 20x sequential | 1.9ms | 4.6ms | **+58%** | *** |

*50 iterations, 20 warmup, Welch's t-test — all p<0.001.*

**What this means:** if your bottleneck is API latency (most use cases), SDK choice doesn't matter for speed. If you're building high-throughput pipelines, local proxies, or processing many requests with fast backends — oxide saves real time. The value proposition for most users is **API completeness** (WebSocket, structured outputs, WASM, stream helpers) and **type safety** (1100+ types), not raw speed.

Full reproducible benchmarks: `node --expose-gc benchmarks/bench_science.js`

## Drop-in Replacement

For existing codebases — change one import:

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

This library started as a need for a fast OpenAI client for my realtime TTS voice agent project. The official Python SDK worked, but I needed Rust-level performance for WebSocket audio streaming and edge deployment.

The entire crate — 100+ API methods, typed streaming, structured outputs, WASM support, Node/Python bindings — was built in a few days using a harness engineering approach with [Claude Code](https://claude.ai/claude-code) and my own toolkit:

1. **Setup**: configured pre-commit hooks (tests, clippy, WASM check, secret scan), OpenAPI spec as ground truth, Python SDK source as reference
2. **Planning**: used [solo-factory](https://github.com/fortunto2/solo-factory) skills (`/plan`, `/build`) with [solograph](https://github.com/fortunto2/solograph) for code intelligence — MCP server that indexes the codebase and provides semantic search across projects
3. **Building**: initial scaffold via Ralph Loop (autonomous agent loop), then manual refinement — architecture decisions, API design, performance tuning
4. **Type sync**: built `py2rust.py` to auto-convert Python Pydantic models to Rust serde structs — 1100+ types across 24 domains, with a two-pass resolver for cross-file references
5. **Quality gates**: every commit runs tests + clippy + WASM compilation check + doc coverage. Pre-commit catches regressions before they land

The key insight: treat the Python SDK as a spec, not as code to port line-by-line. The agent handles mechanical translation (types, error mapping, serialization); you focus on Rust-specific wins (zero-copy, tagged enums, WASM cfg gates).

A harder lesson: **benchmarks are treacherous.** We went through multiple rounds of removing claims that looked impressive but weren't statistically significant at n=5 with live API calls. The honest answer is that SDK overhead is negligible compared to server latency for most use cases. The real wins are in features (WebSocket, structured outputs, WASM) and type safety (1100+ auto-synced types), not milliseconds on single requests.

## Try It

```bash
cargo add openai-oxide tokio --features tokio/full
```

- GitHub: [fortunto2/openai-oxide](https://github.com/fortunto2/openai-oxide)
- crates.io: [openai-oxide](https://crates.io/crates/openai-oxide) + [openai-types](https://crates.io/crates/openai-types)
- npm: [openai-oxide](https://www.npmjs.com/package/openai-oxide)
- PyPI: [openai-oxide](https://pypi.org/project/openai-oxide/)
- Docs: [fortunto2.github.io/openai-oxide](https://fortunto2.github.io/openai-oxide/)
