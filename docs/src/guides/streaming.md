# Streaming

Stream tokens and events as they are generated, reducing time-to-first-token (TTFT) and enabling real-time UI updates.

See the official [Streaming documentation](https://platform.openai.com/docs/api-reference/streaming) for event types and behavior.

## Stream Helpers (recommended)

High-level wrapper with typed events and automatic accumulation — no manual chunk stitching.

```rust
{{#include ../../../examples/chat_stream.rs}}
```

### Event Types

| Event | When | Fields |
|-------|------|--------|
| `Chunk` | Every SSE chunk | Raw `ChatCompletionChunk` |
| `ContentDelta` | New text fragment | `delta`, `snapshot` (accumulated) |
| `ContentDone` | Text complete | `content` (full text) |
| `ToolCallDelta` | Argument fragment | `index`, `name`, `arguments_delta`, `arguments_snapshot` |
| `ToolCallDone` | Tool call complete | `index`, `call_id`, `name`, `arguments` |
| `RefusalDelta/Done` | Model refuses | `delta`/`refusal` |
| `Done` | Stream finished | `finish_reason` |

## Node.js (drop-in replacement)

Same syntax as official `openai` package — `for await` over stream:

```javascript
{{#include ../../../openai-oxide-node/examples/demo-compat.js}}
```

## Python (drop-in replacement)

Same syntax as official `openai` package — `async for` over stream:

```python
{{#include ../../../openai-oxide-python/examples/demo.py}}
```

## Responses API Streaming

Typed events for the Responses API:

```rust
use futures_util::StreamExt;
use openai_oxide::types::responses::{ResponseCreateRequest, ResponseStreamEvent};

let mut stream = client.responses()
    .create_stream(ResponseCreateRequest::new("gpt-5.4-mini").input("Hi"))
    .await?;

while let Some(Ok(event)) = stream.next().await {
    match event {
        ResponseStreamEvent::OutputTextDelta { delta, .. } => print!("{delta}"),
        ResponseStreamEvent::ResponseCompleted { response } => {
            println!("\nDone: {}", response.output_text());
        }
        _ => {}
    }
}
```

## Next Steps

- [Function Calling](./function-calling.md) — Stream with early tool-call parsing
- [WebSocket Sessions](./websockets.md) — Even lower latency with persistent connections
- [Structured Output](./structured-output.md) — Type-safe responses
