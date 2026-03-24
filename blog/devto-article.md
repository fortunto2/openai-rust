---
title: "Squeezing Every Millisecond from the OpenAI API in Rust"
published: false
tags: rust, openai, performance, webassembly
canonical_url: https://github.com/fortunto2/openai-oxide
---

When you're building an AI agent that makes 20-50 tool calls per cycle, every round-trip matters. A 300ms overhead per call compounds to 6-15 seconds of pure waste. We built openai-oxide to eliminate this.

## The Problem

The official Python and Node SDKs are solid — they reuse HTTP/2 connections, have WebSocket support for the Realtime API, and cover all endpoints. But they don't compile to WASM, and their WebSocket mode is only for the Realtime API (audio/multimodal), not for regular text-based Responses API calls.

In the Rust ecosystem, you pick async-openai for types or genai for multi-provider support — but no single crate gives you persistent WebSocket sessions for the Responses API, structured outputs with auto-generated schemas, stream helpers, and WASM deployment in one package.

For an agentic loop where the model calls `read_file`, `search_code`, `edit_file`, `run_tests` in sequence — you want all of this together. That's what we built.

## Persistent WebSockets

The biggest win: keep one `wss://` connection open for the entire agent cycle.

```rust
let mut session = client.ws_session().await?;

// 50 tool calls — zero TLS overhead after the first
for _ in 0..50 {
    let response = session.send(request).await?;
    // execute tool, feed result back
}

session.close().await?;
```

Benchmark: 10 sequential tool calls complete **40% faster** than HTTP REST on the same model.

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

Without these, Cloudflare and nginx buffer streaming responses, adding 50-200ms to TTFT. With them: **530ms TTFT** on gpt-5.4.

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
openai-oxide = { version = "0.9", default-features = false, features = ["chat", "responses"] }
worker = "0.7"
```

Streaming, structured outputs, retry logic — all work in WASM. [Live demo](https://cloudflare-worker-dioxus.nameless-sunset-8f24.workers.dev).

## HTTP Optimizations That Nobody Else Does

We checked — neither async-openai nor genai enable these by default:

| Optimization | Impact |
|:---|:---|
| gzip compression | ~30% smaller responses |
| TCP_NODELAY | Lower latency (disables Nagle) |
| HTTP/2 keep-alive (20s ping) | Prevents idle connection drops |
| HTTP/2 adaptive window | Auto-tunes flow control |
| Connection pool (4/host) | Better parallel performance |

These are all standard reqwest builder options. [Source](https://github.com/fortunto2/openai-oxide/blob/main/src/client.rs#L85).

## Benchmarks

Median of 3 runs, 5 iterations each, gpt-5.4:

### Rust (Responses API)

| Test | openai-oxide | async-openai | genai |
|:---|:---|:---|:---|
| Streaming TTFT | **645ms** | 685ms | 670ms |
| Function calling | **1192ms** | 1748ms | 1030ms |
| WebSocket plain text | **710ms** | N/A | N/A |

### Node.js — oxide wins 8/8

| Test | openai-oxide | official openai | |
|:---|:---|:---|:---|
| Structured output | **1370ms** | 1765ms | +22% |
| Multi-turn | **2283ms** | 2859ms | +20% |
| Streaming TTFT | **534ms** | 580ms | +8% |

### Python — oxide wins 10/12

| Test | openai-oxide | official openai | |
|:---|:---|:---|:---|
| Multi-turn | **2260ms** | 3089ms | +27% |
| Prompt-cached | **4425ms** | 5564ms | +20% |
| Plain text | **845ms** | 997ms | +15% |

Full reproducible benchmarks: `cargo run --example benchmark --features responses --release`

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

## Try It

```bash
cargo add openai-oxide
```

- GitHub: [fortunto2/openai-oxide](https://github.com/fortunto2/openai-oxide)
- crates.io: [openai-oxide](https://crates.io/crates/openai-oxide)
- npm: [openai-oxide](https://www.npmjs.com/package/openai-oxide)
- PyPI: [openai-oxide](https://pypi.org/project/openai-oxide/)
- Docs: [fortunto2.github.io/openai-oxide](https://fortunto2.github.io/openai-oxide/)
