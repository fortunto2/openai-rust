# Implementation Plan: Ergonomics

**Track ID:** ergonomics
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [x] Complete

## Overview

Add granular feature flags, BYOT raw methods, and image save helper to improve DX. Feature flags are the structural foundation — implemented first. BYOT and image save are additive features built on existing infrastructure.

## Phase 1: Feature Flags

Gate each API resource behind an optional Cargo feature so users can minimize compile times. Types remain always available.

### Tasks

- [x] Task 1.1: Add feature flags to `Cargo.toml` <!-- sha:596f8f3 --> — define features: `chat`, `responses`, `embeddings`, `images`, `audio`, `files`, `fine-tuning`, `models`, `moderations`, `batches`, `uploads`, `beta`. Add `default = ["chat", "responses", "embeddings", "images", "audio", "files", "fine-tuning", "models", "moderations", "batches", "uploads", "beta"]`. Add `full = ["chat", "responses", "embeddings", "images", "audio", "files", "fine-tuning", "models", "moderations", "batches", "uploads", "beta"]` alias.
- [x] Task 1.2: Gate resource modules in `src/resources/mod.rs` <!-- sha:596f8f3 --> — wrap each `pub mod` with `#[cfg(feature = "...")]`. Gate sub-resources: `chat/` behind `chat`, `beta/` submodules behind `beta`.
- [x] Task 1.3: Gate resource accessor methods on `OpenAI` struct in `src/client.rs` <!-- sha:596f8f3 --> — wrap `pub fn chat()`, `pub fn images()`, etc. with `#[cfg(feature = "...")]`. Gate `Beta` struct methods similarly behind `beta` feature.
- [x] Task 1.4: Gate resource imports in `src/client.rs` <!-- sha:596f8f3 --> — wrap `use crate::resources::chat::Chat` etc. with matching `#[cfg(feature = "...")]` so unused imports don't error.
- [x] Task 1.5: Verify minimal compilation — test that `cargo check --no-default-features` compiles (just client + types, no resources). Test `cargo test --no-default-features --features chat` passes chat tests only.

### Verification

- [x] `cargo test` (all features, default) — 165 tests pass <!-- verified 2026-03-20 -->
- [x] `cargo check --no-default-features` — compiles (2 expected dead_code warnings for HTTP methods)
- [x] `cargo test --no-default-features --features chat` — chat tests pass
- [x] `cargo clippy -- -D warnings` clean

## Phase 2: BYOT Methods <!-- checkpoint:9cc562a -->

Add `create_raw()` methods on key endpoints that accept any serializable request and return raw `serde_json::Value`. This lets advanced users use custom types or access fields not yet in our type system.

### Tasks

- [x] Task 2.1: Add `post_json<B: Serialize>(&self, path, body) -> Result<serde_json::Value>` helper to `OpenAI` client in `src/client.rs`. Reuses `send_with_retry` logic with `serde_json::Value` as the deserialization target. <!-- sha:8549379 -->
- [x] Task 2.2: Add `create_raw(&self, request: impl Serialize) -> Result<serde_json::Value>` to `Completions` in `src/resources/chat/mod.rs`. Gate behind `chat` feature. <!-- sha:9cc562a -->
- [x] Task 2.3: Add `create_raw(&self, request: impl Serialize) -> Result<serde_json::Value>` to `Responses` in `src/resources/responses.rs`. Gate behind `responses` feature. <!-- sha:9cc562a -->
- [x] Task 2.4: Add `create_raw(&self, request: impl Serialize) -> Result<serde_json::Value>` to `Embeddings` in `src/resources/embeddings.rs`. Gate behind `embeddings` feature. <!-- sha:9cc562a -->
- [x] Task 2.5: Add mockito tests for all three `create_raw()` methods — verify custom request fields pass through and raw JSON response is returned. <!-- sha:9cc562a -->

### Verification

- [x] BYOT methods accept arbitrary JSON and return raw response
- [x] Existing typed methods still work unchanged
- [x] Tests pass for all three endpoints

## Phase 3: Image Save Helper <!-- checkpoint:34f4a35 -->

Add `Image::save(path)` convenience method that handles both URL download and b64_json decode.

### Tasks

- [x] Task 3.1: Add `base64` crate (v0.22) to `[dependencies]` in `Cargo.toml`, gated behind `images` feature. <!-- sha:34f4a35 -->
- [x] Task 3.2: Implement `Image::save(&self, path: impl AsRef<Path>) -> Result<(), OpenAIError>` in `src/types/image.rs`. If `b64_json` is set, decode and write. If `url` is set, download via reqwest and write. Error if neither is set. <!-- sha:34f4a35 -->
- [x] Task 3.3: Add tests for `Image::save()` — test b64_json decode path (unit test with known base64 data), test error when no data present. URL download test with mockito. <!-- sha:34f4a35 -->

### Verification

- [x] `Image::save("output.png")` works for b64_json responses
- [x] `Image::save("output.png")` works for URL responses
- [x] Error returned when image has neither URL nor b64_json

## Phase 4: Docs & Cleanup <!-- checkpoint:1c43c25 -->

### Tasks

- [x] Task 4.1: Update CLAUDE.md — add feature flags documentation, BYOT methods, image save helper to architecture section and implemented APIs table. <!-- sha:1c43c25 -->
- [x] Task 4.2: Update README.md — add feature flags usage example, BYOT example, image save example. <!-- sha:1c43c25 -->
- [x] Task 4.3: Update `docs/roadmap.md` — check off completed Priority 2 items (feature flags, BYOT, image save helper). <!-- sha:1c43c25 -->

### Verification

- [x] CLAUDE.md reflects current project state
- [x] README.md shows new features
- [x] Linter clean, tests pass

## Final Verification

- [x] All acceptance criteria from spec met
- [x] Tests pass (default features + minimal feature set)
- [x] `cargo clippy -- -D warnings` clean
- [x] `cargo fmt -- --check` clean
- [x] Build succeeds
- [x] Documentation up to date

## Context Handoff

_Summary for /build to load at session start — keeps context compact._

### Session Intent

Add granular feature flags, BYOT raw methods, and image save helper to improve openai-oxide ergonomics (roadmap Priority 2).

### Key Files

- `Cargo.toml` — feature flags definition, base64 dep
- `src/lib.rs` — no changes needed (types always exported)
- `src/client.rs` — `#[cfg]` gates on resource accessors + imports, `post_json` helper
- `src/resources/mod.rs` — `#[cfg]` gates on module declarations
- `src/resources/chat/mod.rs` — `create_raw()` method
- `src/resources/responses.rs` — `create_raw()` method
- `src/resources/embeddings.rs` — `create_raw()` method
- `src/types/image.rs` — `Image::save()` method

### Decisions Made

- **Types always available:** Feature flags only gate resource modules (HTTP methods), not type definitions. Users may need types for deserialization even without the HTTP resource. This matches the `aws-sdk-*` pattern.
- **BYOT via `create_raw()` not generic type params:** Adding generic params to existing methods would be a breaking change. Separate `_raw()` methods are additive and non-breaking.
- **base64 crate for image decode:** Standard choice, already a transitive dep. Gated behind `images` feature to avoid pulling it for non-image users.
- **No streaming BYOT:** Streaming returns `SseStream<T>` which requires typed chunks for SSE parsing. Raw streaming would need a different design — out of scope.

### Risks

- Feature flag `#[cfg]` attributes may cause unused-import warnings when features are disabled — need careful gating of `use` statements
- `Image::save()` with URL requires an HTTP client — currently `Image` doesn't hold a reference to the client. Solution: `save()` creates a one-off reqwest client, or takes a `&reqwest::Client` parameter
- Hyphenated feature names (`fine-tuning`) work in Cargo.toml but must be referenced as `fine_tuning` in `#[cfg(feature = "fine-tuning")]` — need to verify Cargo handles this

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
