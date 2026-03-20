# Specification: Ergonomics

**Track ID:** ergonomics
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Add three ergonomic improvements that differentiate openai-oxide from async-openai and bring it closer to the Python SDK's developer experience: **granular feature flags** for compile-time savings, **BYOT (bring your own types) methods** for advanced users who need custom request/response types, and an **image save helper** for the most common post-generation workflow.

These are Priority 2 items from the roadmap, following the completed Type Quality track. The retro showed zero-waste runs work best with focused 10-15 task tracks — this track targets 12 tasks across 3 implementation phases.

## Acceptance Criteria

- [x] Each API resource (chat, responses, embeddings, images, audio, files, fine_tuning, models, moderations, batches, uploads, beta) is gated behind an optional Cargo feature flag
- [x] Default features include all resources (backward compatible — `openai-oxide = "0.7"` compiles everything)
- [x] Users can opt in to specific resources: `openai-oxide = { default-features = false, features = ["chat", "embeddings"] }`
- [x] BYOT `create_raw()` method on Chat Completions accepts `impl Serialize` and returns `serde_json::Value`
- [x] BYOT `create_raw()` method on Responses accepts `impl Serialize` and returns `serde_json::Value`
- [x] BYOT `create_raw()` method on Embeddings accepts `impl Serialize` and returns `serde_json::Value`
- [x] `Image::save(path)` async method downloads URL or decodes b64_json and writes to disk
- [x] All existing tests pass with default features
- [x] `cargo test --no-default-features --features chat` compiles and passes chat tests
- [x] `cargo clippy -- -D warnings` clean
- [x] OpenAPI coverage tests still pass

## Dependencies

- No new external dependencies (image save uses `tokio::fs` + `reqwest` for URL download + base64 decode from standard library)
- base64 crate needed for b64_json decoding (or use the data-encoding crate)

## Out of Scope

- `dyn Config` trait (separate track — requires deeper architecture change)
- Per-request `.header()` sugar on resource groups (already covered by `with_options()`)
- Streaming BYOT methods (streaming has different return type semantics)
- Feature-gated re-exports in `lib.rs` (only resource modules are gated, types are always available)

## Technical Notes

- Feature flags gate `resources/*` modules and their accessor methods on `OpenAI`/`Beta` structs. Types in `types/*` remain always available so users can deserialize responses from other sources.
- BYOT methods reuse existing `client.post()` infrastructure but with generic type params instead of concrete request/response types.
- Image save helper goes on `Image` struct (not `ImagesResponse`) since each image in the `data` array may need individual saving.
- The `base64` crate (v0.22) is the standard choice for decoding b64_json — it's already a transitive dep through reqwest.
