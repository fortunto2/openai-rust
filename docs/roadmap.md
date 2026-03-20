# openai-oxide — Roadmap

Goal: 100% parity with openai-python SDK. Always WebFetch Python source as reference.

## Done
- [x] Core client (config, auth, retries, error handling)
- [x] Chat Completions (create, streaming, tool calling)
- [x] Embeddings
- [x] Models (list, retrieve, delete)
- [x] Moderations
- [x] Images (generate, edit, variations)
- [x] Audio (transcriptions, translations, speech)
- [x] Files (CRUD + content)
- [x] Fine-tuning (jobs CRUD + events)
- [x] Responses API (basic create)
- [x] Assistants (CRUD)
- [x] Threads + Messages
- [x] Runs + submit_tool_outputs
- [x] Vector Stores
- [x] Batches
- [x] Uploads

## Priority 1: Advanced Features (current)
- [ ] OpenAPI spec validation tests — fetch openapi.yaml, auto-generate test fixtures, validate all types deserialize correctly
- [ ] Chat Completions: add ALL missing fields from Python SDK (prediction, reasoning_effort, audio, modalities, seed, logprobs, logit_bias, n, presence/frequency_penalty, service_tier, store)
- [ ] Chat Completions: Usage details (cached_tokens, reasoning_tokens, audio_tokens, prediction_tokens)
- [ ] Responses API: full tool types (web_search, file_search, code_interpreter, computer_use, mcp)
- [ ] Responses API: streaming with all event types
- [ ] Responses API: conversation chaining (previous_response_id)
- [ ] Structured Outputs: strict mode for response_format and tools
- [ ] Realtime API: session creation + ephemeral token
- [ ] Builder pattern for ChatCompletionRequest and ResponseCreateRequest

## Priority 2: Quality
- [ ] OpenAPI fixture tests for every endpoint (deserialize real response JSON)
- [ ] Field coverage report: auto-compare Python SDK vs Rust types
- [ ] Integration test suite (behind live-tests feature flag)
- [ ] Examples: tool_calling, structured_output, responses_api, multi-turn chat

## Priority 3: Ecosystem
- [ ] Middleware/interceptor support (logging, metrics, custom headers)
- [ ] Rate limit tracking from response headers
- [ ] Automatic pagination for list endpoints
- [ ] Retry with jitter
- [ ] Timeout per-request override
- [ ] Azure OpenAI support (different base URL + auth)

## Method

For EVERY item: WebFetch the Python SDK source first, copy field names exactly. Do not guess.
Reference: https://github.com/openai/openai-python/tree/main/src/openai
OpenAPI spec: https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml
