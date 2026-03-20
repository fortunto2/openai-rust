# openai-oxide — Advanced Features (GPT-5.4 era)

**Status:** [ ] Not Started
**Track:** advanced

## Context Handoff

**Intent:** Add advanced GPT-5.4 features and ensure 95%+ field coverage vs Python SDK. Every implementation MUST be based on reading the actual Python source — no guessing.

**What's DONE:** 18 resources, 93 tests, tool calling, streaming, responses API (basic). Published on crates.io v0.2.0.

**CRITICAL WORKFLOW — Python SDK as source of truth:**
For EVERY task:
1. WebFetch the Python SDK source file from `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/{path}`
2. Read EVERY field, type, and method in the Python code
3. Implement the EXACT same fields in Rust (same names via serde rename if needed)
4. Do NOT invent fields or guess — if the Python SDK doesn't have it, don't add it

**OpenAPI spec for validation:**
- Fetch `https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml` for the official schema
- Use it to generate test fixtures (valid request/response JSON)
- Validate our Rust types can deserialize real API responses

---

## Phase 0: OpenAPI Schema Validation Tests

- [ ] Task 0.1: WebFetch `https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml`. Parse the `components/schemas` section. For each schema (ChatCompletionRequest, ChatCompletionResponse, etc.) extract ALL required and optional fields. Create `tests/openapi_coverage.rs` that checks our Rust types have matching fields. Use serde_json::to_value on a default/empty struct and verify all OpenAPI fields are present as keys.
- [ ] Task 0.2: Create `tests/fixtures/` directory with real-world JSON response fixtures based on OpenAPI examples. One fixture per endpoint: `chat_completion.json`, `chat_completion_stream.json`, `embedding.json`, `image.json`, `transcription.json`, `moderation.json`, `fine_tuning_job.json`, `responses.json`. Test that each fixture deserializes without error into our Rust types.

## Phase 1: Chat Completions — Missing Fields from Python SDK

- [ ] Task 1.1: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/chat/chat_completion_create_params.py`. Compare EVERY field with our `ChatCompletionRequest`. Add ALL missing fields: `prediction`, `reasoning_effort`, `audio`, `modalities`, `metadata`, `service_tier`, `store`, `user`, `seed`, `logit_bias`, `logprobs`, `top_logprobs`, `n`, `presence_penalty`, `frequency_penalty`, `stop`. TDD: deserialize fixture with all fields set.
- [ ] Task 1.2: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/chat/chat_completion.py`. Compare EVERY field with our `ChatCompletionResponse`. Add missing: `service_tier`, `system_fingerprint`, `usage.prompt_tokens_details` (with `cached_tokens`, `audio_tokens`), `usage.completion_tokens_details` (with `reasoning_tokens`, `audio_tokens`, `accepted_prediction_tokens`, `rejected_prediction_tokens`). TDD.
- [ ] Task 1.3: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/chat/chat_completion_chunk.py`. Compare with our `ChatCompletionChunk`. Add missing fields for streaming: `usage` (stream_options), `service_tier`, `system_fingerprint`. TDD.

## Phase 2: Responses API — Full Power (from Python SDK)

- [ ] Task 2.1: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/responses/response_create_params.py`. Read ALL fields. Update `ResponseCreateRequest` to match exactly. Add: `instructions`, `tools` array with all tool types, `tool_choice`, `truncation`, `max_output_tokens`, `metadata`, `reasoning` (effort + summary), `include`, `parallel_tool_calls`, `temperature`, `top_p`. TDD.
- [ ] Task 2.2: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/responses/response.py`. Read the Response object fields. Update our `Response` struct. Add: `output` array (ResponseOutputItem variants: message, function_call, file_search_results, etc.), `status`, `usage` with cache fields, `metadata`. TDD.
- [ ] Task 2.3: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/resources/responses/responses.py`. Read the `create` and `stream` methods. Implement `create_stream()` → `Stream<Item = Result<ResponseStreamEvent>>`. WebFetch stream event types. TDD with mock SSE.
- [ ] Task 2.4: WebFetch Python tool type definitions. Create `ResponseTool` enum: `WebSearch { user_location, search_context_size }`, `FileSearch { vector_store_ids, max_num_results }`, `CodeInterpreter`, `ComputerUse { display_width, display_height, environment }`, `Mcp { server_label, server_url, allowed_tools }`, `Function { name, description, parameters, strict }`. TDD.

## Phase 3: Structured Outputs + Builders

- [ ] Task 3.1: WebFetch Python `types/chat/chat_completion_create_params.py` — find `response_format` definition. Enhance our `ResponseFormat::JsonSchema` with `strict: bool`. Add `FunctionDef.strict: Option<bool>`. TDD.
- [ ] Task 3.2: Add builder pattern to `ChatCompletionRequest` — chainable methods: `.model()`, `.messages()`, `.tools()`, `.temperature()`, `.max_tokens()`, `.response_format()`, `.reasoning_effort()`, `.prediction()`, `.stream_options()`. Keep `new()` constructor, add `builder()` as alternative. TDD.
- [ ] Task 3.3: Add builder pattern to `ResponseCreateRequest` — `.model()`, `.input()`, `.instructions()`, `.tools()`, `.previous_response_id()`, `.reasoning()`, `.include()`, `.temperature()`. TDD.

## Phase 4: Realtime API + Examples

- [ ] Task 4.1: WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/resources/beta/realtime/sessions.py` and `types/beta/realtime/`. Create `src/resources/realtime.rs` + `src/types/realtime.rs`. `client.beta().realtime().sessions().create()` → ephemeral token. TDD with mockito.
- [ ] Task 4.2: Add examples: `examples/tool_calling.rs`, `examples/structured_output.rs`, `examples/responses_api.rs`. Each shows a complete working flow. Gated behind `live-tests` feature.
- [ ] Task 4.3: Bump version to 0.3.0. Update README with new features table. `make check`. Final commit.

## Review Criteria

1. WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/chat/chat_completion_create_params.py` — list ALL fields, compare with our struct. Report coverage %.
2. WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/responses/response_create_params.py` — same check.
3. Run `cargo test` — all must pass including OpenAPI fixture tests.
4. Coverage < 95% of Python SDK fields → `<solo:redo/>`.
5. Coverage ≥ 95% → `<solo:done/>`.
