---
title: "Show HN: openai-oxide – Rust OpenAI client I use across terminal, iOS, browser, Node, and Python"
url: https://github.com/fortunto2/openai-oxide
---

## HN first comment

Hey HN. I'm a solo dev and I accidentally built a cross-platform OpenAI SDK.

It started with a voice assistant. I needed low-latency audio streaming with the OpenAI Realtime API, and Python wasn't cutting it. WebSocket connections kept dropping, garbage collection pauses were audible in the audio stream. So I wrote a small Rust client. Just enough to hold a WebSocket open and send/receive JSON.

Then I started building a coding agent (think Cursor but in the terminal). It needed structured outputs — the model returns typed JSON that I deserialize into Rust structs, not strings I parse with regex. It needed streaming with early function call parsing — start executing `read_file` the moment the model finishes generating its arguments, don't wait for the whole response. I added those features to the same Rust crate.

Then came the iOS app. I'm building a video montage tool that analyzes your camera roll using Apple Vision (face detection, scene classification, body pose on the Neural Engine) and auto-generates montages. The AI part — scene selection, voiceover timing, timeline planning — is an agent loop that makes 5-10 LLM calls per montage. I needed the same structured outputs, the same retry logic, the same provider switching (OpenAI, OpenRouter, Gemini, Ollama) on iOS. I didn't want to rewrite any of it in Swift.

So I compiled the Rust crate to a static library and used UniFFI to auto-generate Swift bindings. The types defined in Rust (`TranscriptSummaryPlan`, `SceneSelection`, `TimelineEdit`) become native Swift types. The agent loop runs in Rust. Swift handles the UI and Apple's native frameworks. Provider switching works from the iOS Settings screen — same `OxideClient` underneath, different base URL.

At that point I realized I had a full OpenAI SDK that runs on five platforms:

```
openai-oxide (Rust core)
├── sgr-agent (my agent framework)
│   ├── rust-code     — coding agent, TUI (terminal)
│   └── va-agent      — video montage agent (iOS via UniFFI)
├── voice-assistant    — WebSocket, realtime
├── Node.js bindings   — napi-rs
├── Python bindings    — PyO3
└── WASM               — browsers, Dioxus, edge runtimes
```

**The part I didn't expect:** the cross-platform constraint made the code better. When your HTTP client has to work on iOS (no tokio runtime customization), in WASM (no filesystem, no threads), and in Python (GIL-aware), you end up with cleaner abstractions than if you only targeted one platform.

### Technical stuff that might interest HN

**WebSocket mode for the Responses API.** Most people know about OpenAI's Realtime API (audio/multimodal WebSocket). Fewer know there's a separate WebSocket endpoint at `wss://api.openai.com/v1/responses` for text-based agent loops. The server caches your conversation context in memory per-connection, so continuations skip the context reload. OpenAI says ~40% faster for 20+ tool calls. I measured 29-44% on small n. The official Python/Node SDKs don't wrap this endpoint yet. I built a connection pool (`WsPool`) around it — idle timeouts, per-key capacity limits, connection reuse.

**The type sync problem.** OpenAI has 1100+ types across 24 API domains. They change constantly. I wrote `py2rust.py` — it reads the official Python SDK's Pydantic models and generates Rust serde structs. Two-pass resolver handles cross-file type references. Machine-generated files (`_gen.rs`) get overwritten each sync; hand-written overrides (enums, custom builders) live in separate files and survive. `make sync-types` when OpenAI ships something new. The types crate is standalone with zero deps beyond serde — you can use it without my HTTP client.

**Structured outputs that flow across the FFI boundary.** `parse::<T>()` takes a Rust struct with `#[derive(JsonSchema)]`, generates a JSON schema, sends it to the API, gets back guaranteed-valid JSON, and deserializes it. One call. The key insight: these same Rust types become Swift types via UniFFI and JavaScript types via napi-rs. So the model's structured response is typed end-to-end from the API to the UI layer, with no manual JSON parsing anywhere in the chain.

**This is part of a bigger machine.** I'm building a [solo-factory](https://github.com/fortunto2/solo-factory) — a toolkit for launching startups solo. It includes Claude Code skills for planning, building, deploying, and reviewing projects. openai-oxide and sgr-agent are the AI infrastructure layer. Every new product I start gets the same Rust core, same agent framework, same type-safe LLM integration. The video app, the coding agent, the voice assistant — they all started from the same scaffold, and they all share the same battle-tested LLM client underneath.

The solo-factory approach means I'm not building one product — I'm building the tooling that lets me ship products faster. openai-oxide is one of those tools that turned out useful beyond my own projects.

### What I'd like to discuss

- Has anyone else gone the "one Rust core, thin bindings everywhere" route for a client SDK? Curious about long-term maintenance.
- The UniFFI → Swift path works but feels underexplored. Anyone using it in production iOS apps?
- Is OpenAI's WebSocket mode for the Responses API on anyone's radar? Documented but rarely mentioned.

Links: [GitHub](https://github.com/fortunto2/openai-oxide) · [crates.io](https://crates.io/crates/openai-oxide) · [coding agent](https://github.com/fortunto2/rust-code) · [npm](https://www.npmjs.com/package/openai-oxide) · [PyPI](https://pypi.org/project/openai-oxide/)
