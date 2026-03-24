# Structured Output

Force the model to return JSON matching a specific schema. Guarantees valid, parseable output without prompt engineering tricks.

See the official [Structured Outputs guide](https://platform.openai.com/docs/guides/structured-outputs) for schema format and limitations.

## Rust — `parse::<T>()` (recommended)

Derive `JsonSchema` on your struct and call `parse::<T>()`. The SDK auto-generates the schema and deserializes the response.

Requires feature `structured`: `cargo add openai-oxide --features structured`

```rust
{{#include ../../../examples/live_features_test.rs:1:30}}
```

## Rust — Manual Schema

For full control, construct the schema yourself:

```rust
{{#include ../../../examples/structured_output.rs}}
```

## Node.js (drop-in replacement)

Same syntax as official `openai` package — change one import:

```javascript
{{#include ../../../openai-oxide-node/examples/parsing-compat.js}}
```

## Python (drop-in replacement)

Same syntax as official `openai` package — change one import:

```python
{{#include ../../../openai-oxide-python/examples/parsing.py}}
```

## Next Steps

- [Function Calling](./function-calling.md) — Combine structured output with tool use
- [Streaming](./streaming.md) — Stream with typed events
- [Responses API](./responses-api.md) — Full parameter reference
