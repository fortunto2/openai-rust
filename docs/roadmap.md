# openai-oxide — Roadmap

Goal: production-grade OpenAI Rust client. Beat async-openai (1.8K★) on type safety and ergonomics.

Reference: https://github.com/64bit/async-openai (competitor), ~/startups/shared/openai-python/ (source of truth)

## Done
- [x] Core client (config, auth, retries with backoff, error handling)
- [x] Chat Completions (create, streaming, tool calling, all fields 100%)
- [x] Responses API (create, retrieve, delete, streaming, all tool types)
- [x] Embeddings, Models, Moderations, Images, Audio, Files
- [x] Fine-tuning, Batches, Uploads
- [x] Assistants, Threads + Messages, Runs, Vector Stores
- [x] Realtime API (session creation + ephemeral token)
- [x] Predicted outputs, prompt caching, reasoning effort
- [x] Structured outputs with strict mode
- [x] OpenAPI coverage tests (88% overall)
- [x] Builder pattern for ChatCompletionRequest and ResponseCreateRequest
- [x] Pre-commit: tests + OpenAPI coverage + clippy + quality checks

## Priority 1: Type Quality (current — make Knuth proud)
- [ ] Replace ALL `serde_json::Value` in public types with typed structs/enums (currently 27 → target 0)
- [ ] Replace `String` with enums where appropriate (Role, Status, FinishReason, Object type)
- [ ] Add `#[non_exhaustive]` to all public enums
- [ ] Doc comments on every public field
- [ ] OpenAPI coverage ≥95% (currently 88%, Images at 57%)

## Priority 2: Ergonomics (match async-openai)
- [ ] Per-request customization: `.query()`, `.header()`, `.headers()` on resource groups
- [ ] `dyn Config` trait for provider-agnostic code (OpenAI, Azure, custom)
- [x] Azure OpenAI support (`AzureConfig` builder, `OpenAI::azure()`, api-key/AD token auth)
- [ ] Granular feature flags: `chat-types`, `response-types`, `embedding-types` (compile-time savings)
- [ ] BYOT (bring your own types): `_byot` methods accepting `impl Serialize` / `DeserializeOwned`
- [ ] Image save helper: `response.save("./output").await`

## Priority 3: Production Hardening
- [ ] Middleware/interceptor trait: logging, metrics, custom headers, rate limit tracking
- [ ] Rate limit info from headers: `x-ratelimit-remaining`, `x-ratelimit-reset`
- [ ] Automatic pagination for list endpoints (cursor-based iterator)
- [ ] Retry with jitter (currently fixed backoff)
- [ ] Timeout per-request override
- [ ] Webhook signature verification (for Responses API webhooks)
- [ ] Request ID tracking (`x-request-id` header)

## Priority 4: Ecosystem
- [ ] `openai-oxide-macros`: derive macro for function tool definitions
- [ ] WASM support (feature-gated, no tokio — use wasm-bindgen-futures)
- [ ] OpenRouter / Ollama / vLLM compatibility (custom base URL + model mapping)
- [ ] Integration test suite with real API (behind `live-tests` feature)
- [ ] Benchmarks vs async-openai (request/response overhead, streaming latency)
- [ ] Published docs on docs.rs with comprehensive examples

## Competitive Edge vs async-openai
| Area | async-openai | openai-oxide (target) |
|------|-------------|----------------------|
| Types | Generated from spec | Hand-crafted from Python SDK + OpenAPI validation |
| Value fields | Some `serde_json::Value` | Zero — fully typed |
| Python parity | No | Yes — same field names, same behavior |
| OpenAPI tests | No | Auto-validates against spec on every commit |
| Pre-commit | No | Tests + coverage + clippy + quality audit |

## Method

Read Python SDK source from `~/startups/shared/openai-python/src/openai/` for exact types.
Validate against `tests/openapi.yaml` on every commit.
