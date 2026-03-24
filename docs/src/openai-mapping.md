# OpenAI Docs → openai-oxide

openai-oxide has 1:1 parity with the official Python SDK. Use [OpenAI's documentation](https://platform.openai.com/docs) as your primary reference — the same concepts, parameter names, and patterns apply.

## Endpoint Mapping

| OpenAI Guide | Rust | Node.js | Python |
|---|---|---|---|
| [Chat Completions](https://platform.openai.com/docs/guides/chat-completions) | `client.chat().completions().create()` | `client.createResponse({model, input})` | `await client.create(model, input)` |
| [Responses API](https://platform.openai.com/docs/api-reference/responses) | `client.responses().create()` | `client.createText(model, input)` | `await client.create(model, input)` |
| [Streaming](https://platform.openai.com/docs/api-reference/streaming) | `client.responses().create_stream()` | `client.createStream(model, input)` | `await client.create_stream(model, input)` |
| [Function Calling](https://platform.openai.com/docs/guides/function-calling) | `client.responses().create_stream_fc()` | `client.createResponse({model, input, tools})` | `await client.create_with_tools(model, input, tools)` |
| [Structured Output](https://platform.openai.com/docs/guides/structured-outputs) | `ResponseCreateRequest::new(model).text_format(schema)` | `client.createResponse({model, input, text})` | `await client.create_structured(model, input, name, schema)` |
| [Embeddings](https://platform.openai.com/docs/guides/embeddings) | `client.embeddings().create()` | via `createResponse()` raw | via `create_raw()` |
| [Image Generation](https://platform.openai.com/docs/guides/images) | `client.images().generate()` | via `createResponse()` raw | via `create_raw()` |
| [Text-to-Speech](https://platform.openai.com/docs/guides/text-to-speech) | `client.audio().speech().create()` | via `createResponse()` raw | via `create_raw()` |
| [Speech-to-Text](https://platform.openai.com/docs/guides/speech-to-text) | `client.audio().transcriptions().create()` | via `createResponse()` raw | via `create_raw()` |
| [Fine-tuning](https://platform.openai.com/docs/guides/fine-tuning) | `client.fine_tuning().jobs().create()` | via `createResponse()` raw | via `create_raw()` |
| [Realtime API](https://platform.openai.com/docs/guides/realtime) | `client.ws_session()` | `client.wsSession()` | — |
| [Assistants](https://platform.openai.com/docs/assistants) | `client.beta().assistants()` | via `createResponse()` raw | via `create_raw()` |

> Node.js and Python have typed helpers for the top 5 endpoints. All other endpoints work via raw JSON methods.

## Parameter Names

Parameter names match the Python SDK exactly:

| OpenAI Python | Rust | Node.js |
|---|---|---|
| `model="gpt-5.4"` | `.model("gpt-5.4")` | `{ model: "gpt-5.4" }` |
| `max_output_tokens=100` | `.max_output_tokens(100)` | `{ maxOutputTokens: 100 }` |
| `temperature=0.7` | `.temperature(0.7)` | `{ temperature: 0.7 }` |
| `stream=True` | `create_stream()` | `createStream()` |
| `store=True` | `.store(true)` | `{ store: true }` |

## openai-oxide Exclusive Features

These features are not available in the official SDKs:

| Feature | API | Description |
|---|---|---|
| WebSocket Sessions | `client.ws_session()` | Persistent connection, 37% faster agent loops |
| Hedged Requests | `hedged_request()` | Race redundant requests, cut P99 latency |
| Stream FC Early Parse | `create_stream_fc()` | Execute tools 400ms before response finishes |
| SIMD JSON | `features = ["simd"]` | AVX2/NEON accelerated parsing |
| WASM | `default-features = false` | Full streaming in Cloudflare Workers |
