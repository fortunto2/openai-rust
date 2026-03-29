# openai-oxide

Feature-complete OpenAI client for **Rust**, **Node.js**, and **Python**.

openai-oxide implements the full [Responses API](https://platform.openai.com/docs/api-reference/responses), [Chat Completions](https://platform.openai.com/docs/api-reference/chat), and 20+ other endpoints with persistent WebSockets, hedged requests, streaming with early-parse, and type-safe Structured Outputs.

## Why openai-oxide?

- **Streaming** — SSE parser with anti-buffering headers, 2.5x faster per-chunk vs official JS SDK
- **WebSocket Mode** — Persistent `wss://` connections, 29-44% faster on multi-turn benchmarks
- **Stream FC Early Parse** — Execute tools the moment `arguments.done` arrives
- **Structured Outputs** — `parse::<T>()` auto-generates JSON schema from Rust types
- **Hedged Requests** — Send redundant requests, cancel the slower (technique from Google's "The Tail at Scale")
- **WASM First-Class** — Full streaming in Cloudflare Workers and browsers
- **1100+ Types** — Auto-synced from Python SDK via [`openai-types`](https://crates.io/crates/openai-types) crate

## Packages

| Package | Registry | Install |
|---------|----------|---------|
| `openai-oxide` | [crates.io](https://crates.io/crates/openai-oxide) | `cargo add openai-oxide` |
| `openai-types` | [crates.io](https://crates.io/crates/openai-types) | `cargo add openai-types` |
| `openai-oxide` | [npm](https://www.npmjs.com/package/openai-oxide) | `npm install openai-oxide` |
| `openai-oxide` | [PyPI](https://pypi.org/project/openai-oxide/) | `pip install openai-oxide` |

## OpenAI Compatibility

Parameter names match the official Python SDK exactly. If the [OpenAI docs](https://platform.openai.com/docs) show `model="gpt-5.4"`, use `.model("gpt-5.4")` in Rust or `{model: "gpt-5.4"}` in Node.js.

See the [OpenAI Docs Mapping](./openai-mapping.md) for a complete cross-reference.
