# Hedged Requests

Hedged requests race multiple identical API calls and return the first successful response. This technique reduces P99 tail latency by 50-96% at the cost of additional API usage.

This is an openai-oxide exclusive feature not available in the official SDKs.

## Rust

```rust
use openai_oxide::{OpenAI, types::responses::*};

let client = OpenAI::from_env()?;

// Race 2 identical requests, return whichever finishes first
let response = client.responses().hedged_request(
    ResponseCreateRequest::new("gpt-5.4-mini")
        .input("Quick question: what is 2+2?"),
    2, // number of concurrent requests
).await?;
```

## When to Use

- Latency-sensitive applications (real-time UIs, voice assistants)
- Short, deterministic prompts where cost of duplicates is low
- Production systems with strict P99 SLA requirements

## Trade-offs

- Uses N times the tokens (one request per hedge)
- Best for short prompts where the latency gain outweighs cost
- Not recommended for long-running completions with `max_output_tokens > 1000`
