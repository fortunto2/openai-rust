# Streaming

Stream tokens and events as they are generated, reducing time-to-first-token (TTFT) and enabling real-time UI updates.

See the official [Streaming documentation](https://platform.openai.com/docs/api-reference/streaming) for event types and behavior.

## Stream Helpers (recommended)

High-level wrapper with typed events and automatic accumulation — no manual chunk stitching.

```rust
use futures_util::StreamExt;
use openai_oxide::{OpenAI, types::chat::*};
use openai_oxide::stream_helpers::ChatStreamEvent;

#[tokio::main]
async fn main() -> Result<(), openai_oxide::OpenAIError> {
    let client = OpenAI::from_env()?;
    let request = ChatCompletionRequest::new(
        "gpt-5.4-mini",
        vec![ChatCompletionMessageParam::User {
            content: UserContent::Text("Count to 5".into()),
            name: None,
        }],
    );

    let mut stream = client.chat().completions()
        .create_stream_helper(request).await?;

    while let Some(event) = stream.next().await {
        match event? {
            ChatStreamEvent::ContentDelta { delta, .. } => print!("{delta}"),
            ChatStreamEvent::ToolCallDone { name, arguments, .. } => {
                println!("\nTool call: {name}({arguments})");
            }
            ChatStreamEvent::Done { .. } => break,
            _ => {}
        }
    }
    Ok(())
}
```

### `get_final_completion()`

Consume the stream and return the assembled response — useful when you want streaming latency but don't need intermediate events:

```rust
let stream = client.chat().completions()
    .create_stream_helper(request).await?;
let completion = stream.get_final_completion().await?;
println!("{}", completion.choices[0].message.content.as_deref().unwrap_or(""));
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

## Raw SSE Stream

For full control, use the low-level stream directly:

```rust
{{#include ../../../examples/chat_stream.rs}}
```

Run: `OPENAI_API_KEY=sk-... cargo run --example chat_stream`

## Responses API Streaming

Typed events for the Responses API:

```rust
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
