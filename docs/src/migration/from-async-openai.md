# Migrating from async-openai

openai-oxide and [async-openai](https://github.com/64bit/async-openai) are both Rust OpenAI clients, but differ in API design, feature set, and architecture.

## Key Differences

| async-openai | openai-oxide |
|---|---|
| `Client::new()` | `OpenAI::from_env()?` |
| `CreateChatCompletionRequestArgs::default()...build()?` | `ChatCompletionRequest::new("model")...` |
| Derive-macro builders | Manual builder methods (no proc macros) |
| `backoff` crate for retries | Built-in configurable retry policy |
| No WebSocket support | Native WebSocket sessions |
| No WASM support | First-class WASM target |
| No hedged requests | Built-in hedged request support |

## Pattern: async-openai to openai-oxide

```rust
// async-openai
let client = Client::new();
let request = CreateChatCompletionRequestArgs::default()
    .model("gpt-5.4")
    .messages(vec![ChatCompletionRequestUserMessageArgs::default()
        .content("Hello")
        .build()?
        .into()])
    .build()?;
let response = client.chat().create(request).await?;
```

```rust
// openai-oxide
let client = OpenAI::from_env()?;
let response = client.chat().completions().create(
    ChatCompletionRequest::new("gpt-5.4")
        .messages(vec![ChatMessage::user("Hello")])
).await?;
```

The main wins from switching: simpler builder API (no `.build()?` calls), WebSocket support, WASM compatibility, hedged requests, and feature-flag granularity.
