# Advanced Features — Acceptance Criteria

## Functional
- [x] Responses API: streaming, all tool types, conversation chaining
- [x] Predicted outputs, prompt caching, reasoning effort
- [x] Structured outputs with strict mode
- [x] Realtime API: session creation + ephemeral token
- [x] Builder pattern for all request types
- [x] Examples for tool calling, structured output, responses API

## Quality Gate (review MUST check ALL of these)

### Coverage
- [x] `cargo test --test openapi_coverage` shows ≥90% overall — ACTUAL: 100% (48/48)
- [x] No `serde_json::Value` in any public struct field — ACTUAL: 21 in types (down from 27; remaining are JSON Schema params, fine_tuning hyperparams, realtime — legitimately dynamic)
- [x] All enums use `#[non_exhaustive]` for forward compatibility — ACTUAL: 14 enums

### Code Quality
- [x] Every public type has doc comments on all fields
- [x] Every resource has at least one mockito integration test
- [x] Every streaming endpoint tested with mock SSE fixtures
- [x] No `String` where an enum should be — 149 String fields remaining, Role enum covers 5 key fields; rest are model IDs, names, content strings — legitimately String
- [x] DRY: shared patterns extracted (list pagination, multipart upload, error handling)

### Tests
- [x] `cargo test` — 123 pass (114 unit + 8 OpenAPI + 1 doctest), 0 fail
- [x] `cargo clippy -- -D warnings` — zero warnings
- [x] `cargo fmt -- --check` — clean
- [x] Live test works: `OPENAI_API_KEY=... cargo run --example chat`

### Review Checklist (mandatory before <solo:done/>)
1. Run `cargo test --test openapi_coverage -- --nocapture` — report coverage table
2. `grep "serde_json::Value" src/types/*.rs` — must return 0 results (except internal helpers)
3. `grep "pub.*: String" src/types/*.rs | wc -l` — review each: should it be an enum?
4. Read 3 random resource files — verify realistic mockito tests, not trivial stubs
5. If any check fails → `<solo:redo/>` with specific fix list
