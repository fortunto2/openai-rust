# openai-oxide — Advanced Features (GPT-5.4 era)

**Status:** [x] Complete
**Track:** advanced

## Context Handoff

**Intent:** Add advanced GPT-5.4 features and ensure 95%+ field coverage vs Python SDK.

**What's DONE:** 18 resources, 93 tests, tool calling, streaming, responses API (basic). v0.2.0 on crates.io.

**CRITICAL WORKFLOW — Read Python SDK LOCALLY:**
For EVERY task:
1. Read `~/startups/shared/openai-python/src/openai/types/{file}.py` — the Python Pydantic model
2. Read `~/startups/shared/openai-python/src/openai/resources/{file}.py` — the Python resource methods
3. Compare EVERY field with our Rust struct
4. Add ALL missing fields. Same names via `#[serde(rename = "...")]` if needed
5. Do NOT invent fields — if Python doesn't have it, we don't add it

**OpenAPI spec:** `tests/openapi.yaml` in repo — use for fixture generation and validation tests.

---

## Phase 0: OpenAPI Validation Tests

- [x] Task 0.1: Parse `tests/openapi.yaml` `components/schemas` section. Create `tests/openapi_coverage.rs` — for each major schema (ChatCompletion, Embedding, etc.) extract all field names, compare with our Rust struct fields. Report coverage %. <!-- sha:acb5ce0 -->
- [x] Task 0.2: Create `tests/fixtures/` with JSON response fixtures from OpenAPI examples. One per endpoint. Test each fixture deserializes into our Rust types without error. <!-- sha:acb5ce0 -->

## Phase 1: Chat Completions — Full Field Parity

- [x] Task 1.1: Read Python SDK. Add missing request fields: modalities, reasoning_effort, verbosity, audio, prediction, web_search_options, max_tokens, functions, function_call. 100% OpenAPI coverage. TDD. <!-- sha:acb5ce0 -->
- [x] Task 1.2: Add prompt_tokens_details and completion_tokens_details to Usage. TDD. <!-- sha:acb5ce0 -->
- [x] Task 1.3: Streaming fields (service_tier, system_fingerprint, usage) already present from prior work. <!-- sha:acb5ce0 -->

## Phase 2: Responses API — Full Power

- [x] Task 2.1: Update ResponseCreateRequest with tools, tool_choice, parallel_tool_calls, top_p, truncation, reasoning, include, service_tier, user, text. TDD. <!-- sha:dfa5a42 -->
- [x] Task 2.2: Update Response with all echo fields, usage details (input/output token details), completed_at, etc. TDD. <!-- sha:dfa5a42 -->
- [x] Task 2.3: Implement create_stream() → Stream<Item = Result<ResponseStreamEvent>>. TDD with mock SSE. <!-- sha:dfa5a42 -->
- [x] Task 2.4: Create ResponseTool enum: Function, WebSearch, FileSearch, CodeInterpreter, ComputerUse, Mcp, ImageGeneration. TDD. <!-- sha:dfa5a42 -->

## Phase 3: Structured Outputs + Builders

- [x] Task 3.1: strict already on JsonSchema and FunctionDef from prior work. <!-- sha:acb5ce0 -->
- [x] Task 3.2: Builder pattern for ChatCompletionRequest — 12 chainable methods. TDD. <!-- sha:601bc55 -->
- [x] Task 3.3: Builder pattern for ResponseCreateRequest — 10 chainable methods. TDD. <!-- sha:601bc55 -->

## Phase 4: Realtime API + Examples

- [x] Task 4.1: Realtime API types + resource. Session creation with ephemeral token, tools, turn detection. TDD. <!-- sha:c35eec5 -->
- [x] Task 4.2: Examples: tool_calling.rs, structured_output.rs, responses_api.rs. All compile. <!-- sha:c35eec5 -->
- [x] Task 4.3: Bump to 0.3.0. README updated with all 22 endpoints. make check passes. <!-- sha:c35eec5 -->

## Review Criteria

1. Read `~/startups/shared/openai-python/src/openai/types/chat/chat_completion_create_params.py` — list ALL fields, compare with our struct. Report coverage %.
2. Read `~/startups/shared/openai-python/src/openai/types/responses/response_create_params.py` — same.
3. `cargo test` must pass including OpenAPI fixture tests.
4. Coverage < 95% → `<solo:redo/>`. Coverage ≥ 95% → `<solo:done/>`.
