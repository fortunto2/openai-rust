# Advanced Features — Acceptance Criteria

## Functional
- Responses API: streaming, all tool types, conversation chaining
- Predicted outputs, prompt caching, reasoning effort
- Structured outputs with strict mode
- Realtime API: session creation + ephemeral token
- Builder pattern for all request types
- Examples for tool calling, structured output, responses API

## Quality Gate (review MUST check ALL of these)

### Coverage
- `cargo test --test openapi_coverage` shows ≥90% overall
- No `serde_json::Value` in any public struct field — replace with typed struct/enum
- All enums use `#[non_exhaustive]` for forward compatibility

### Code Quality
- Every public type has doc comments on all fields
- Every resource has at least one mockito integration test
- Every streaming endpoint tested with mock SSE fixtures
- No `String` where an enum should be (grep for `pub.*: String` and verify)
- DRY: shared patterns extracted (list pagination, multipart upload, error handling)

### Tests
- `cargo test` — all pass (target: 120+ tests)
- `cargo clippy -- -D warnings` — zero warnings
- `cargo fmt -- --check` — clean
- Live test works: `OPENAI_API_KEY=... cargo run --example chat`

### Review Checklist (mandatory before <solo:done/>)
1. Run `cargo test --test openapi_coverage -- --nocapture` — report coverage table
2. `grep "serde_json::Value" src/types/*.rs` — must return 0 results (except internal helpers)
3. `grep "pub.*: String" src/types/*.rs | wc -l` — review each: should it be an enum?
4. Read 3 random resource files — verify realistic mockito tests, not trivial stubs
5. If any check fails → `<solo:redo/>` with specific fix list
