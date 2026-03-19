# openai-rust Phase 1 — Core Client + Chat API

**Status:** [ ] Not Started
**Track:** phase1
**Estimated tasks:** 12

## Context Handoff

**Intent:** Build an idiomatic Rust OpenAI client that mirrors the official Python SDK 1:1. Start with the core client infrastructure and Chat Completions API (the most-used endpoint).

**Reference:** Study https://github.com/openai/openai-python for exact API structure, parameter names, and response types. Use `WebFetch` to read the Python source files from GitHub.

**Key decisions:**
- async-first (tokio + reqwest)
- serde for all JSON (de)serialization, use `#[serde(rename_all = "snake_case")]`
- thiserror for error types
- mockito for HTTP mocking in tests (no real API calls by default)
- Builder pattern for request construction
- All response fields `Option<T>` where API may omit

---

## Phase 1: Project Setup

- [x] Task 1.1: Create `Cargo.toml` with deps: reqwest (rustls-tls, json, stream, multipart), tokio (full), serde (derive), serde_json, thiserror, tracing. Dev-deps: mockito, tokio (rt-multi-thread). Feature: `live-tests`. <!-- sha:99fdcdc -->
- [x] Task 1.2: Create `src/error.rs` — `OpenAIError` enum: ApiError { status, message, type_, code }, RequestError(reqwest::Error), JsonError(serde_json::Error), StreamError(String), InvalidArgument(String). Implement Display, Error. Add `ErrorResponse` struct matching OpenAI error JSON format. <!-- sha:99fdcdc -->

## Phase 2: Base Client

- [x] Task 2.1: Create `src/config.rs` — `ClientConfig` struct: api_key, base_url (default "https://api.openai.com/v1"), organization, project, timeout_secs, max_retries. Builder pattern. Load api_key from `OPENAI_API_KEY` env if not set. <!-- sha:99fdcdc -->
- [~] Task 2.2: Create `src/client.rs` — `OpenAI` struct wrapping reqwest::Client + config. Methods: `new(api_key)`, `with_config(config)`, `from_env()`. Internal: `get()`, `post()`, `delete()` helpers that add auth headers, parse errors. Test: construct client, verify headers.
- [ ] Task 2.3: Add retry logic to client — exponential backoff on 429/500/502/503. Configurable max_retries (default 2). Parse `Retry-After` header. Test with mockito: mock 429 → 200 sequence.

## Phase 3: Chat Completions (most important endpoint)

- [ ] Task 3.1: Study Python SDK chat types. Use WebFetch to read `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/chat/chat_completion.py` and `chat_completion_message.py` and `chat_completion_chunk.py`. Create `src/types/chat.rs` with all structs: ChatCompletionRequest, ChatCompletionResponse, ChatCompletionMessage (role, content, tool_calls, refusal), ChatCompletionChoice, CompletionUsage, ToolCall, Function. All derive Serialize+Deserialize. TDD: deserialize fixture JSON from real API response.
- [ ] Task 3.2: Add message role types and content types — `Role` enum (system, user, assistant, tool, developer), `ContentPart` enum (text, image_url, input_audio). Support both `content: "string"` and `content: [parts]` via serde untagged enum. TDD.
- [ ] Task 3.3: Create `src/resources/chat.rs` — `Chat` struct (ref to client), `Completions` struct. Method: `client.chat().completions().create(request).await -> Result<ChatCompletionResponse>`. POST to `/chat/completions`. TDD with mockito: mock response JSON, verify deserialization.
- [ ] Task 3.4: Add streaming — `create_stream()` returns `Stream<Item = Result<ChatCompletionChunk>>`. Parse SSE lines (`data: {...}`). Handle `[DONE]`. Create `src/streaming.rs` for SSE parser. TDD: feed mock SSE lines, verify chunks.
- [ ] Task 3.5: Add tool/function calling support — `Tool` struct (type, function), `FunctionDef` struct (name, description, parameters as Value). `ToolChoice` enum (auto, none, required, specific). Wire into ChatCompletionRequest. TDD with fixture.

## Phase 4: Wire + Verify

- [ ] Task 4.1: Create `src/lib.rs` — re-export OpenAI, types, errors. Create `src/types/mod.rs` and `src/resources/mod.rs`. Verify `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check` all pass. Write a doc example in lib.rs showing basic usage.
- [ ] Task 4.2: Add `examples/chat.rs` — working example (requires OPENAI_API_KEY). `examples/chat_stream.rs` — streaming example. Both gated behind `live-tests` or just compile-check.
