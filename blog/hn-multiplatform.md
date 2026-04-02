---
title: "One Rust crate powers my coding agent, video analyzer, and voice assistant across 5 platforms"
---

I'm building three products that all need LLM calls: a TUI coding agent, an iOS video analyzer, and a realtime voice assistant. Different platforms, different latency requirements, different OpenAI APIs. Same Rust core.

```
openai-oxide (Rust — HTTP client, WebSocket pool, SSE streaming, structured outputs)
├── openai-types (1100+ types, auto-synced from Python SDK)
│
├── sgr-agent (Schema-Guided Reasoning — agent framework)
│   │   Sessions, tool loops, structured outputs, multi-provider
│   │
│   ├── rust-code (TUI coding agent — macOS/Linux terminal)
│   │   └── [planned] WASM build → same agent in the browser
│   │
│   └── va-agent (video montage agent — iOS via UniFFI)
│       ├── iOS SwiftUI app (Apple Vision + AVFoundation)
│       └── OxideClient → OpenAI / OpenRouter / Gemini / Ollama
│
├── voice-assistant (WebSocket mode, <100ms per turn)
│
├── openai-oxide-node (napi-rs — Node.js/TypeScript)
└── openai-oxide-python (PyO3 — Python)
```

## Why one crate

Official OpenAI SDKs exist for Python, Node, and Go. Each is a separate codebase. When OpenAI adds a feature, each team implements it independently. Behavior drifts.

With one Rust core:
- Same HTTP tuning (gzip, TCP_NODELAY, HTTP/2 keep-alive, connection pooling) on every platform
- Same WebSocket connection pool and retry logic everywhere
- Same streaming parser, same structured output schema generation
- Fix a bug once, all platforms get it

The bindings are thin. Node: napi-rs (`.node` addon). Python: PyO3 (`.so`). iOS: UniFFI (auto-generated Swift types). Browser: WASM (`wasm32-unknown-unknown`).

## sgr-agent: the framework layer

[sgr-agent](https://github.com/fortunto2/rust-code/tree/master/crates/sgr-agent) (Schema-Guided Reasoning) is an agent framework built on openai-oxide. It handles the parts that are common across all my products: session management, tool dispatch loops, structured output parsing, multi-provider routing (OpenAI, OpenRouter, Gemini, Ollama through one `OxideClient`).

The "schema-guided" part: every agent interaction uses `parse::<T>()` to get typed Rust structs back from the model. The agent defines its reasoning steps, tool calls, and outputs as Rust types with `#[derive(JsonSchema)]`. The framework generates the JSON schema, sends it to the API, and deserializes the response. No string parsing, no JSON key lookups.

sgr-agent also uses `create_stream_fc()` for early function call parsing: start executing a tool the moment the model finishes generating its arguments, before the stream closes.

## The coding agent

[rust-code](https://github.com/fortunto2/rust-code) is a TUI coding agent that reads your codebase, plans edits, calls tools (`read_file`, `edit_file`, `run_tests`), and iterates. It's built on sgr-agent.

Because openai-oxide compiles to WASM, the same agent code can run in a browser. Same tool loop, same streaming, same structured outputs. Only the tool implementations change (filesystem → browser APIs).

## The video analyzer: Rust core on iOS

I'm building an iOS app for automatic video montage. It analyzes your video library using Apple Vision (Neural Engine for face detection, body pose, scene classification) and generates montages with AI-powered scene selection.

The AI brain is a Rust crate (`va-agent`) that uses sgr-agent → openai-oxide. It compiles to a static library and talks to the SwiftUI app via UniFFI (auto-generated Swift bindings from Rust types).

The architecture:

```
Swift (SwiftUI)              Rust (va-agent)
┌─────────────────┐          ┌──────────────────────────────┐
│ Vision analyzer  │          │ sgr-agent (session + tools)  │
│ AVFoundation     │◄─uniffi─►│ openai-oxide (LLM calls)     │
│ PhotoKit         │          │ scene selection, transcript   │
│ Settings UI      │          │ timeline planning             │
└─────────────────┘          └──────────────────────────────┘
```

The provider is configurable from the iOS Settings screen: OpenAI, OpenRouter, Gemini, or local Ollama. All go through the same `OxideClient`. openai-oxide treats any OpenAI-compatible endpoint the same.

Structured outputs drive the agent loop. When the model selects scenes or plans voiceover timing, it returns typed Rust structs:

```rust
#[derive(Deserialize, JsonSchema)]
struct TranscriptSummaryPlan {
    segments: Vec<Segment>,
    voiceover_script: String,
    target_duration_secs: f32,
}
```

Defined once in Rust, auto-exposed to Swift via UniFFI, validated by the LLM via JSON schema. No manual JSON parsing on either side.

The agent runs an autonomous loop (up to 10 steps) with tools for video analysis, face detection, timeline assembly, and rendering. Tool dispatch, structured outputs, and retries live in Rust. Swift handles UI and Apple's native APIs.

## The voice assistant: WebSocket latency

OpenAI has a [WebSocket mode](https://platform.openai.com/docs/guides/websocket-mode) for the Responses API. One persistent `wss://` connection stays open. The server caches context locally so continuations are faster.

```rust
let mut session = client.ws_session().await?;

loop {
    let transcript = listen_to_mic().await;
    let response = session.send(
        ResponseCreateRequest::new("gpt-5.4")
            .input(&transcript)
            .previous_response_id(&last_id)
    ).await?;
    speak(response.output_text()).await;
    last_id = response.id.clone();
}
```

OpenAI reports [up to ~40% faster](https://platform.openai.com/docs/guides/websocket-mode) for 20+ tool call chains. Our measurements (29-44%, n=5) match.

The connection pool (`WsPool`) manages sessions with idle timeouts and per-key caps. For voice, you want the connection warm before the user starts speaking.

## When SDK overhead starts to matter

On today's OpenAI API (200ms-2s per call), SDK overhead is <1%. But:

- **Fast inference** (Cerebras, Groq, local models) returns in 10-50ms. SDK overhead becomes 5-30% of wall time.
- **Agent farms** with hundreds of concurrent sessions create thousands of requests/sec. Per-request overhead compounds.
- **Voice apps**: every millisecond is perceptible. Rust's 0.1-0.5ms vs Python's 1-5ms per call matters at the margin.

Mock benchmarks isolating SDK overhead: 2-3x lower than the official JS SDK across all payload sizes (p<0.001, Welch's t-test). SSE streaming: 283µs vs 742µs for 114 chunks. As inference gets faster, this gap becomes the bottleneck.

## The type problem

OpenAI's API: 1100+ types, 24 domains, changes regularly. We built [openai-types](https://crates.io/crates/openai-types), auto-generated from the Python SDK via `py2rust.py`:

```bash
make sync-types  # two-pass resolver for cross-file type references
```

`_gen.rs` files get overwritten on sync. Manual overrides (enums, builders) live in separate files, never touched. New OpenAI field → one command.

Zero dependencies beyond `serde`. Use it standalone if you're building your own transport.

## What works today

| Platform | How | Status | Example |
|----------|-----|--------|---------|
| **Rust** | native | stable | [rust-code](https://github.com/fortunto2/rust-code) (coding agent) |
| **iOS/macOS** | UniFFI → Swift | in dev | Video montage app (private, Apple Vision + LLM agent) |
| **Node.js** | napi-rs | stable | npm bindings, Zod structured outputs |
| **Python** | PyO3 | stable | PyPI bindings, Pydantic v2 |
| **Browser/Edge** | WASM | stable | [Dioxus demo](https://cloudflare-worker-dioxus.nameless-sunset-8f24.workers.dev) |
| **Android** | UniFFI → Kotlin | planned | |

## Links

- Core: [openai-oxide](https://github.com/fortunto2/openai-oxide) (GitHub) / [crates.io](https://crates.io/crates/openai-oxide) / [npm](https://www.npmjs.com/package/openai-oxide) / [PyPI](https://pypi.org/project/openai-oxide/)
- Types: [openai-types](https://crates.io/crates/openai-types) (1100+ types, standalone)
- Agent framework: [sgr-agent](https://github.com/fortunto2/rust-code/tree/master/crates/sgr-agent)
- Coding agent: [rust-code](https://github.com/fortunto2/rust-code)
- Benchmarks: [methodology and numbers](https://github.com/fortunto2/openai-oxide/tree/main/benchmarks)
