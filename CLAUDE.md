# CLAUDE.md — openai-oxide

Idiomatic Rust client for the OpenAI API. Published on [crates.io](https://crates.io/crates/openai-oxide). 1:1 parity with [openai-python](https://github.com/openai/openai-python).

## Crate Name

- **crates.io:** `openai-oxide`
- **Rust import:** `use openai_oxide::...`
- **GitHub repo:** `fortunto2/openai-rust` (repo name kept as-is)

## Goal

Replicate the official Python SDK in Rust:
- Same resource structure: `client.chat().completions().create(params).await`
- Same parameter names and types
- Streaming support (SSE)
- All endpoints (see plan for remaining work)
- Async-first (tokio + reqwest)
- Strongly typed responses (serde)

## Tech Stack

| Component | Technology |
|-----------|-----------|
| HTTP | reqwest (rustls-tls) |
| Async | tokio |
| Serialization | serde + serde_json |
| Streaming | reqwest streaming + SSE parsing |
| Errors | thiserror |
| Builder | typed builder pattern (no derive macro) |
| Testing | cargo test + mockito (HTTP mocking) |

## Architecture

```
src/
  lib.rs              — pub mod, re-exports
  azure.rs            — AzureConfig builder, from_env(), build() → OpenAI client
  client.rs           — OpenAI client (api_key, base_url, org, retries, Beta struct, with_options, azure())
  error.rs            — OpenAIError enum
  config.rs           — ClientConfig (timeouts, retries, base_url, default_headers, default_query, azure auth mode)
  request_options.rs  — RequestOptions (per-request headers, query, extra_body, timeout)
  streaming.rs        — SSE stream parser
  resources/
    audio.rs          — transcriptions, translations, speech
    batches.rs        — batch create/list/retrieve/cancel
    beta/             — assistants, threads, runs, vector_stores (v2 header)
    chat/             — chat.completions.create() + create_stream()
    embeddings.rs     — embeddings.create()
    files.rs          — file CRUD + content download
    fine_tuning.rs    — fine_tuning.jobs CRUD + events
    images.rs         — generate, edit, create_variation
    models.rs         — list, retrieve, delete
    moderations.rs    — moderations.create()
    responses.rs      — responses create/retrieve/delete
    uploads.rs        — upload create/cancel/complete
  types/
    audio.rs          — Transcription, Translation, Speech types + AudioVoice, AudioResponseFormat, SpeechResponseFormat enums
    batch.rs          — Batch types + BatchStatus enum
    beta.rs           — Assistant, Thread, Run, VectorStore types + RunStatus, VectorStoreStatus enums
    chat.rs           — ChatCompletionRequest, ChatCompletionResponse, ... + ImageDetail, FunctionCallOption enums
    common.rs         — Usage, Role, FinishReason, ServiceTier, ReasoningEffort, SearchContextSize, AutoOrFixed<T>, MaxResponseTokens
    embedding.rs      — Embedding types + EncodingFormat enum
    file.rs           — FileObject, FileDeleted types + FilePurpose, FileStatus enums
    fine_tuning.rs    — FineTuningJob types + FineTuningStatus, FineTuningEventLevel enums
    image.rs          — Image types + ImageQuality, ImageSize, ImageStyle, ImageOutputFormat, ImageBackground, ImageModeration enums
    model.rs          — Model types
    moderation.rs     — Moderation types
    responses.rs      — Response types
    realtime.rs       — Realtime session types + RealtimeAudioFormat, TurnDetectionType, Eagerness enums
    upload.rs         — Upload types + UploadStatus enum
```

## Implemented APIs

| API | Method | Status |
|-----|--------|--------|
| Chat Completions | `client.chat().completions().create()` | Done |
| Chat Completions (streaming) | `client.chat().completions().create_stream()` | Done |
| Responses | `client.responses().create()` / `retrieve()` / `delete()` | Done |
| Responses (streaming) | `client.responses().create_stream()` | Done |
| Embeddings | `client.embeddings().create()` | Done |
| Models | `client.models().list()` / `retrieve()` / `delete()` | Done |
| Images | `client.images().generate()` / `edit()` / `create_variation()` | Done |
| Audio | `client.audio().transcriptions()` / `translations()` / `speech()` | Done |
| Files | `client.files().create()` / `list()` / `retrieve()` / `delete()` / `content()` | Done |
| Fine-tuning | `client.fine_tuning().jobs().create()` / `list()` / `cancel()` / `list_events()` | Done |
| Moderations | `client.moderations().create()` | Done |
| Batches | `client.batches().create()` / `list()` / `retrieve()` / `cancel()` | Done |
| Uploads | `client.uploads().create()` / `cancel()` / `complete()` | Done |
| Assistants (beta) | `client.beta().assistants()` CRUD | Done |
| Threads (beta) | `client.beta().threads()` CRUD + messages | Done |
| Runs (beta) | `client.beta().runs(thread_id)` CRUD | Done |
| Vector Stores (beta) | `client.beta().vector_stores()` CRUD | Done |
| Realtime (beta) | `client.beta().realtime().sessions().create()` | Done |

**Current version:** v0.5.0 on crates.io

Remaining (experimental/newer): Evals, Skills, Videos, Containers, legacy Completions.

## Essential Commands

```bash
make check                          # fmt + clippy + test
make test                           # all tests
cargo test --features "live-tests"  # tests hitting real API (needs OPENAI_API_KEY)
cargo clippy -- -D warnings
cargo fmt -- --check
```

## Reference

**Python SDK (LOCAL — use Read tool, NOT WebFetch):**
`~/startups/shared/openai-python/src/openai/`
- `resources/` — one module per API resource
- `types/` — Pydantic models for requests/responses
- READ these files directly before implementing anything

**OpenAPI spec (in repo):** `tests/openapi.yaml`
- Official OpenAI spec with code samples
- Use for validation tests and fixture generation

**RULE:** For EVERY type, Read the Python source first. Copy field names exactly. Do not guess.

## Don't

- Invent new API names — match Python SDK exactly
- Use derive macros for builders — manual impl for flexibility
- Skip error handling — every API error type must be covered
- Add async-std support — tokio only
- Leave `serde_json::Value` where a typed struct should be
- Use `String` where an enum is appropriate (roles, statuses, formats)
- Duplicate code — extract shared patterns into helpers

## Do

- TDD: write test with expected request/response JSON before implementing
- Use mockito for HTTP mocking (no real API calls in default tests)
- Use serde(rename) to match OpenAI's snake_case JSON
- Support both `OPENAI_API_KEY` env var and explicit key
- Make all response fields `Option` where the API may omit them

## Code Quality Standard

This crate aims for production-grade quality. Every change must pass this bar:

**Types:**
- No `serde_json::Value` in public types — always a typed struct or enum
- No `String` where a finite set exists — use enums (`Role`, `FinishReason`, `Status`)
- All enums: `#[serde(rename_all = "snake_case")]` + `#[non_exhaustive]` for forward compat
- All optional fields: `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`
- All structs: doc comments on every public field explaining what it does

**API design:**
- Builder methods return `&mut Self` for chaining, not consume self
- Every resource method has a doc example (even if `ignore` for no API key)
- Error types are precise — `ApiError { status, type_, message, code }`, not `String`
- Streaming returns `impl Stream<Item = Result<T, E>>` — never collects internally

**Architecture (async patterns):**

```rust
// 1. Config trait — provider-agnostic (OpenAI, Azure, custom)
pub trait Config: Send + Sync {
    fn base_url(&self) -> &str;
    fn auth_header(&self) -> HeaderMap;
    fn api_version(&self) -> Option<&str>;  // Azure needs this
}
// Client is generic: Client<C: Config> — supports dyn dispatch too

// 2. Middleware trait (Tower-style, but simpler)
pub trait Middleware: Send + Sync {
    fn on_request(&self, req: &mut reqwest::RequestBuilder);
    fn on_response(&self, resp: &reqwest::Response);
    fn on_error(&self, err: &OpenAIError);
}
// Chain: LoggingMiddleware, MetricsMiddleware, RateLimitTracker

// 3. Streaming — zero-copy SSE with backpressure
// SseStream<T> implements futures::Stream + Unpin
// Consumer controls pace via .next().await — no internal buffering
// Supports: ChatCompletionChunk, ResponseStreamEvent, RunStreamEvent

// 4. Pagination — async iterator
pub struct Paginator<T> { /* cursor, has_more, fetch_next() */ }
impl<T> Stream for Paginator<T> { /* auto-fetches next page */ }
// Usage: client.files().list_all().collect::<Vec<_>>().await

// 5. Resource access — zero-cost, borrows client
// client.chat() returns Chat<'_> (borrows, no clone, no Arc)
// Resources are thin wrappers — all state lives in Client

// 6. Retry — configurable strategy
pub trait RetryPolicy: Send + Sync {
    fn should_retry(&self, attempt: u32, error: &OpenAIError) -> Option<Duration>;
}
// Default: ExponentialBackoff { max_retries: 2, jitter: true }
// Per-request override: client.chat().with_retry(NoRetry).completions().create(req)
```

**Tests:**
- Every endpoint: at least one mockito test with realistic fixture JSON
- Every type: at least one deserialization test with a full real-world response
- `tests/openapi_coverage.rs` must show ≥90% field coverage
- Pre-commit runs all tests + coverage + clippy + fmt

**Patterns:**
- DRY: if 3+ resources share a pattern (list/retrieve/delete), extract a macro or trait
- Pagination: list endpoints return a typed `ListResponse<T>` with `has_more` + cursor
- Multipart: use a shared helper for file upload endpoints (audio, files, images)
- `#[must_use]` on all builders — compiler warns if you forget `.await`
- All public types: `Clone + Debug + Send + Sync` (thread-safe by default)

**On every iteration:**
- Run `cargo test --test openapi_coverage -- --nocapture` to see current gaps
- Read Python SDK source for the specific types being worked on
- If coverage dropped — fix before committing
- `cargo clippy -- -D warnings` must be clean — zero warnings, not just zero errors
