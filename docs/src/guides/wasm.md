# WASM / Cloudflare Workers

openai-oxide compiles to WebAssembly for use in Cloudflare Workers, Deno Deploy, and browser environments. Full streaming support is included.

## Setup

```toml
[dependencies]
openai-oxide = { version = "0.9", default-features = false, features = ["responses", "websocket-wasm"] }
```

Disable default features to exclude tokio and native TLS, which are not available in WASM.

## Cloudflare Worker Example

```rust
use openai_oxide::OpenAI;
use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let client = OpenAI::new(env.secret("OPENAI_API_KEY")?.to_string());

    let response = client.responses().create(
        ResponseCreateRequest::new("gpt-5.4-mini")
            .input("Hello from the edge!")
    ).await.map_err(|e| worker::Error::from(e.to_string()))?;

    Response::ok(response.output_text())
}
```

## Limitations

- No filesystem access (audio file uploads require bytes, not paths)
- WebSocket mode uses `websocket-wasm` feature instead of `websocket`
- SIMD feature is not available in WASM targets
