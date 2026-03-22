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
- [x] Type quality: 19 typed enums + 5 typed structs replacing `serde_json::Value` (type-quality track)
- [x] `#[non_exhaustive]` on all public enums
- [x] RequestOptions: per-request headers, query, extra_body, timeout via `with_options()`
- [x] Azure OpenAI support (`AzureConfig` builder, `OpenAI::azure()`, api-key/AD token auth)
- [x] Granular feature flags: 12 resource features, `default` and `full` aliases

## Priority 1: Ergonomics (ergonomics track — complete)
- [x] Feature flags (`cargo check --no-default-features` compiles)
- [x] BYOT `create_raw()` methods: Chat, Responses, Embeddings (accept `impl Serialize`, return `serde_json::Value`)
- [x] Image save helper: `Image::save(path)` — URL download + b64_json decode
- [x] `dyn Config` trait for provider-agnostic code (OpenAI, Azure, custom) — separate track

## Priority 2: Production Hardening
- [x] Middleware/interceptor trait: logging, metrics, custom headers, rate limit tracking
- [x] Rate limit info from headers: `x-ratelimit-remaining`, `x-ratelimit-reset`
- [x] Automatic pagination for list endpoints (cursor-based iterator)
- [ ] Retry with jitter (currently fixed backoff)
- [ ] Timeout per-request override
- [ ] Webhook signature verification (for Responses API webhooks)
- [ ] Request ID tracking (`x-request-id` header)

## Priority 3: Ecosystem
- [x] `openai-oxide-macros`: derive macro for function tool definitions
- [x] WASM support (feature-gated, no tokio — use wasm-bindgen-futures)
- [x] OpenRouter / Ollama / vLLM compatibility (custom base URL + model mapping)
- [ ] Integration test suite with real API (behind `live-tests` feature)
- [ ] Benchmarks vs async-openai (request/response overhead, streaming latency)
- [ ] Published docs on docs.rs with comprehensive examples

## Competitive Edge vs async-openai
| Area | async-openai | openai-oxide |
|------|-------------|----------------------|
| Types | Generated from spec | Hand-crafted from Python SDK + OpenAPI validation |
| Value fields | Some `serde_json::Value` | Fully typed (19 enums, 5 structs) |
| Python parity | No | Yes — same field names, same behavior |
| OpenAPI tests | No | Auto-validates against spec on every commit |
| Pre-commit | No | Tests + coverage + clippy + quality audit |
| Azure | No | AzureConfig builder, AD token + api-key auth |
| Feature flags | No | 12 granular resource features |
| BYOT methods | No | `create_raw()` for custom types |
| Image save | No | `Image::save(path)` with b64/URL |

## Method

Read Python SDK source from `~/startups/shared/openai-python/src/openai/` for exact types.
Validate against `tests/openapi.yaml` on every commit.
