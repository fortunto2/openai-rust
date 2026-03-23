# WebSocket Sessions

Persistent WebSocket connections eliminate per-request TLS handshakes and HTTP overhead, achieving 37% faster round-trip times for agent loops and multi-turn conversations.

See the official [Realtime API guide](https://platform.openai.com/docs/guides/realtime) for session configuration.

## Rust

```rust
{{#include ../../../examples/websocket.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example websocket --features websocket`

## When to Use WebSockets

- Agent loops with 3+ sequential LLM calls
- Real-time conversational UIs
- High-throughput batch processing where latency matters

## Known Issues

### Decimal temperature causes silent close (code=1000)

**Status:** OpenAI bug as of March 2026

Sending `temperature` as a decimal (e.g. `0.7`, `1.2`) over WebSocket causes the server to immediately close the connection with code=1000 and an empty reason — no error event is returned. Integer values (`0`, `1`, `2`) work fine. The same decimal values work normally over HTTP.

**Workaround:** Omit `temperature` from WebSocket requests (the API uses model default ~1.0), or round to integer.

**Tracking:** [OpenAI Community #1375536](https://community.openai.com/t/1375536)
