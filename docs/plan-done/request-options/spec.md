# Specification: Per-Request Options & with_options()

**Track ID:** request-options_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Add per-request customization matching the Python SDK's `extra_headers`, `extra_query`, `extra_body`, and `timeout` pattern. Implement via a `RequestOptions` struct stored on the client, with a `with_options()` method that returns a cheap clone with merged options. This covers all resource methods without modifying each one — the same pattern the Python SDK uses for `client.with_options(timeout=30).chat.completions.create(...)`.

Also adds `default_headers` and `default_query` to `ClientConfig` for client-level defaults, matching the Python SDK's constructor parameters.

## Acceptance Criteria

- [x] `RequestOptions` struct with `headers`, `query`, `extra_body`, `timeout` fields
- [x] `RequestOptions` builder methods: `.header()`, `.headers()`, `.query()`, `.query_param()`, `.extra_body()`, `.timeout()`
- [x] `OpenAI::with_options(opts)` returns a new client with merged options (cheap — reqwest::Client is Arc-cloned)
- [x] `ClientConfig::default_headers()` and `ClientConfig::default_query()` set initial options
- [x] Internal `request()` method applies options (headers merged, query params appended, timeout overridden)
- [x] All existing resource methods automatically respect client-level options without signature changes
- [x] Mockito tests verify: extra headers sent, query params appended, timeout override works, options merge correctly
- [x] Doc example in lib.rs showing `with_options()` usage
- [x] `cargo test` passes, `cargo clippy -- -D warnings` clean

## Dependencies

- None (builds on existing client architecture)

## Out of Scope

- Azure OpenAI support (separate track — needs Config trait)
- `dyn Config` trait (separate track)
- Per-method `_with_options()` variants (not needed — `with_options()` on client covers all methods)
- Middleware/interceptor trait (Priority 3)

## Technical Notes

- `reqwest::Client` uses `Arc` internally — cloning is cheap (no HTTP connection duplication)
- Python SDK uses `extra_headers`, `extra_query`, `extra_body` on every method. In Rust, `client.with_options(opts)` achieves the same without modifying 40+ resource method signatures.
- Python SDK's `copy()` / `with_options()` returns a new client instance with merged settings — we mirror this exactly.
- `extra_body` merges additional JSON fields into the request body at the `request()` level via `serde_json::Value` merge. This is internal-only — not exposed in public types.
- Headers merge: per-request > client options > ClientConfig defaults. Later values win on key collision.
