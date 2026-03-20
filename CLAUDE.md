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
  client.rs           — OpenAI client (api_key, base_url, org, retries, Beta struct)
  error.rs            — OpenAIError enum
  config.rs           — ClientConfig (timeouts, retries, base_url)
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
    audio.rs          — Transcription, Translation, Speech types
    batch.rs          — Batch types
    beta.rs           — Assistant, Thread, Run, VectorStore types
    chat.rs           — ChatCompletionRequest, ChatCompletionResponse, ...
    common.rs         — Usage, shared types
    embedding.rs      — Embedding types
    file.rs           — FileObject, FileDeleted types
    fine_tuning.rs    — FineTuningJob types
    image.rs          — Image types
    model.rs          — Model types
    moderation.rs     — Moderation types
    responses.rs      — Response types
    upload.rs         — Upload types
```

## Implemented APIs

| API | Method | Status |
|-----|--------|--------|
| Chat Completions | `client.chat().completions().create()` | Done |
| Chat Completions (streaming) | `client.chat().completions().create_stream()` | Done |
| Responses | `client.responses().create()` / `retrieve()` / `delete()` | Done |
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

Study the Python SDK at https://github.com/openai/openai-python:
- `src/openai/resources/` — one module per API resource
- `src/openai/types/` — Pydantic models for requests/responses

## Don't

- Invent new API names — match Python SDK exactly
- Use derive macros for builders — manual impl for flexibility
- Skip error handling — every API error type must be covered
- Add async-std support — tokio only

## Do

- TDD: write test with expected request/response JSON before implementing
- Use mockito for HTTP mocking (no real API calls in default tests)
- Use serde(rename) to match OpenAI's snake_case JSON
- Support both `OPENAI_API_KEY` env var and explicit key
- Make all response fields `Option` where the API may omit them
