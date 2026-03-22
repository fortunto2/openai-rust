# Full-Stack Rust WASM Example

A production-ready architecture demonstrating a **100% Rust stack** using **Dioxus** on the frontend and a **Cloudflare Worker** on the backend. This example showcases the extreme flexibility of `openai-oxide` by running on the Edge and dynamically adapting to different LLM providers.

## The Power of `openai-oxide` in this Example

1. **True WASM Support (No Feature Stripping):**
   Unlike `async-openai` (which drops streaming, retries, and WebSockets when compiled to WASM), `openai-oxide` retains **100% of its features** when compiled to `wasm32-unknown-unknown` for Cloudflare Workers.
   
2. **Hybrid Connection Intelligence:**
   The backend intelligently routes your traffic based on the upstream provider you choose in the UI:
   - **OpenAI (Native):** If you use `https://api.openai.com/v1`, the Worker uses `openai-oxide`'s **WebSocket Mode (`websocket` feature)** to establish a direct `wss://` connection to the Responses API. This eliminates TLS handshake latency per turn, achieving blazing-fast **~350ms TTFT**.
   - **Custom Providers (OpenRouter, LM Studio, etc):** If you specify a custom Base URL that doesn't support the native Responses WebSocket API, `openai-oxide` automatically falls back to standard HTTP SSE (`chat/completions`) streaming. The Worker reads the HTTP stream and repacks it into WebSockets for the frontend on the fly.

3. **Built-in Prompt Caching:**
   When connecting to OpenAI, the Worker automatically injects `.prompt_cache_key("oxide-dioxus-chat")`. This means the frontend can statelessly send the *entire* chat history on every turn, and OpenAI will cache the prefix on their servers. This drastically reduces multi-turn latency (up to -80%) and cuts token costs, without requiring a complex external database.

## Architecture Highlights
- **Zero JS:** Both the UI and the Backend are written entirely in Rust.
- **Edge Performance:** Deploying to Cloudflare puts the compute physically closer to the user.
- **Durable Objects:** Uses Cloudflare Durable Objects to hold the stateful WebSocket connection between the Browser and the OpenAI Edge, bypassing the stateless limitations of standard serverless functions.
- **Live Metrics:** The frontend uses `web-sys` performance APIs to calculate and display live TTFT (Time-To-First-Token) and generation speed (tokens/sec).

## Deploy

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/fortunto2/openai-oxide/tree/main/examples/cloudflare-worker-dioxus)

*(Note: Deploying this requires a Paid Cloudflare Workers plan because it uses Durable Objects).*

## Local Dev

```bash
# Set your local env vars
cd worker
echo "OPENAI_API_KEY=sk-..." > .dev.vars

# Build the Dioxus App & run the Worker
cd ..
./build.sh
```
