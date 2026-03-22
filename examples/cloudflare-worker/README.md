# OpenAI Oxide Cloudflare Worker Example

A lightweight, zero-JS Cloudflare Worker example demonstrating how to use `openai-oxide` to build an edge API in a strict WASM environment. This example includes a simple built-in HTML playground that tests the Worker's streaming capabilities.

## The Power of `openai-oxide` in this Example

1. **True WASM Edge Compute:**
   Cloudflare Workers run in V8 Isolates and do not support standard OS sockets or native TLS (`reqwest`'s default backend). While other libraries like `async-openai` completely strip out streaming and retries when targeting WASM, `openai-oxide` uses `gloo-net` and specialized WASM targets to keep **100% of its features intact**.

2. **Native Responses API Streaming:**
   This example uses the cutting-edge `Responses API` (`ResponseCreateRequest`) to generate text. The Worker connects to OpenAI, receives the Server-Sent Events (`ResponseStreamEvent`) stream, and seamlessly pipes it back to the client directly from the Edge, maintaining perfect streaming without buffering.

3. **Zero Configuration Complexity:**
   Thanks to `openai-oxide`'s `client.chat().completions().create_stream()`, handling the async event stream inside a Cloudflare Worker `fetch` handler is reduced to a simple `while let Some(chunk) = stream.next().await` loop. No manual `fetch` boilerplate or chunk decoding required.

## Try it out
The worker exposes a `POST /chat` endpoint that accepts JSON `{"message": "Hello", "model": "gpt-4o-mini"}` and returns an SSE stream.
It also serves a simple HTML playground on `GET /`.

## Deploy

[![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/fortunto2/openai-oxide/tree/main/examples/cloudflare-worker)

*(Note: This is a lightweight, stateless example and works perfectly on the Free Cloudflare Workers tier).*

## Local Dev

```bash
# Add your API Key to a local vars file
echo "OPENAI_API_KEY=sk-..." > .dev.vars

# Run locally
npx wrangler dev
```
