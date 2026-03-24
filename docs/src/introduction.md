# openai-oxide

A high-performance, feature-complete OpenAI client for **Rust**, **Node.js**, and **Python**.

openai-oxide implements the full [Responses API](https://platform.openai.com/docs/api-reference/responses), [Chat Completions](https://platform.openai.com/docs/api-reference/chat), and 20+ other endpoints with performance primitives like persistent WebSockets, hedged requests, and early-parsing for function calls.

## Why openai-oxide?

- **Zero-Overhead Streaming** — Custom zero-copy SSE parser, TTFT ~670ms
- **WebSocket Mode** — Persistent `wss://` connections, 37% faster agent loops
- **Stream FC Early Parse** — Execute tools ~400ms before response finishes
- **SIMD JSON** — Opt-in AVX2/NEON for microsecond parsing
- **Hedged Requests** — 50-96% P99 tail latency reduction
- **WASM First-Class** — Full streaming in Cloudflare Workers and browsers

## Packages

| Package | Registry | Install |
|---------|----------|---------|
| `openai-oxide` | [crates.io](https://crates.io/crates/openai-oxide) | `cargo add openai-oxide` |
| `openai-oxide` | [npm](https://www.npmjs.com/package/openai-oxide) | `npm install openai-oxide` |
| `openai-oxide` | [PyPI](https://pypi.org/project/openai-oxide/) | `pip install openai-oxide` |

## OpenAI Compatibility

Parameter names match the official Python SDK exactly. If the [OpenAI docs](https://platform.openai.com/docs) show `model="gpt-5.4"`, use `.model("gpt-5.4")` in Rust or `{model: "gpt-5.4"}` in Node.js.

See the [OpenAI Docs Mapping](./openai-mapping.md) for a complete cross-reference.
