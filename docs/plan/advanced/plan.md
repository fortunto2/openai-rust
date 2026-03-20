# openai-oxide — Advanced Features (GPT-5.4 era)

**Status:** [ ] Not Started
**Track:** advanced

## Context Handoff

**Intent:** Add advanced GPT-5.4 features missing from current implementation. The crate has basic CRUD for all endpoints but lacks cutting-edge features that power users need.

**What's DONE:** 18 resources, 93 tests, tool calling, streaming, responses API (basic). Published on crates.io.

**Method:** For EACH task, WebFetch the Python SDK source to find exact field names and types. Compare with our Rust types and add missing fields/methods.

---

## Phase 1: Responses API — Full Power

- [ ] Task 1.1: WebFetch Python `types/responses/response.py` and `types/responses/response_input_item.py`. Add ALL missing fields to `ResponseCreateRequest`: `instructions`, `tools` (code_interpreter, file_search, function, computer_use, mcp), `tool_choice`, `truncation`, `max_output_tokens`, `metadata`, `reasoning` (effort, summary). Add `ResponseTool` enum with all variants. TDD.
- [ ] Task 1.2: Add Responses API streaming — `create_stream()` method returning `Stream<Item = Result<ResponseStreamEvent>>`. WebFetch Python `types/responses/response_stream_event.py` for all event types: `response.created`, `response.in_progress`, `response.output_item.added`, `response.output_text.delta`, `response.function_call_arguments.delta`, `response.completed`. TDD with mock SSE.
- [ ] Task 1.3: Add `previous_response_id` conversation chaining — verify it's wired through, add convenience method `create_continuation(prev_id, input)`. Add `include` parameter for filtering response fields. TDD.
- [ ] Task 1.4: Add Responses API tool definitions — `WebSearchTool`, `FileSearchTool`, `ComputerUseTool`, `McpTool`, `CodeInterpreterTool` structs. Each with their config fields. WebFetch Python for exact schemas.

## Phase 2: Predicted Outputs + Prompt Caching

- [ ] Task 2.1: Add `prediction` field to `ChatCompletionRequest` — `Prediction { type: "content", content: String }`. For code editing use case (model returns delta from prediction). WebFetch Python `types/chat/chat_completion.py` for exact type. TDD.
- [ ] Task 2.2: Add prompt caching fields — `cached_tokens` in Usage, `cache_creation_input_tokens` and `cache_read_input_tokens` in response. WebFetch Python for exact field names. TDD: verify deserialization of cached response.
- [ ] Task 2.3: Add `reasoning_effort` field to ChatCompletionRequest — `"low" | "medium" | "high"`. Add `reasoning` config to Responses API. WebFetch Python for both. TDD.

## Phase 3: Structured Outputs — Full Support

- [ ] Task 3.1: Enhance `ResponseFormat::JsonSchema` — add `strict: bool` field. Add convenience constructor `ResponseFormat::strict_json_schema(name, schema)`. TDD.
- [ ] Task 3.2: Add `strict: true` support for tool/function calling — `FunctionDef.strict` field. When true, model output guaranteed to match schema. TDD.
- [ ] Task 3.3: Add helper: `ChatCompletionRequest::with_structured_output<T: JsonSchema>(schema)` — auto-generates JSON schema from schemars trait, sets response_format. TDD with sample struct.

## Phase 4: Realtime API Types

- [ ] Task 4.1: WebFetch Python `resources/beta/realtime/` — add realtime session types. `RealtimeSession`, `RealtimeSessionCreateRequest` (model, voice, instructions, tools, modalities). `src/resources/realtime.rs` + `src/types/realtime.rs`. TDD.
- [ ] Task 4.2: Add realtime token generation — `client.beta().realtime().sessions().create()` returns ephemeral token for WebSocket connection. TDD with mockito.

## Phase 5: Builder Ergonomics + Examples

- [ ] Task 5.1: Add builder methods to `ChatCompletionRequest` — `.model()`, `.messages()`, `.tools()`, `.temperature()`, `.max_tokens()`, `.stream()`, `.response_format()`, `.reasoning_effort()`. Chainable. TDD.
- [ ] Task 5.2: Add builder methods to `ResponseCreateRequest` — `.model()`, `.input()`, `.tools()`, `.instructions()`, `.previous_response_id()`, `.reasoning()`. Chainable. TDD.
- [ ] Task 5.3: Add examples: `examples/tool_calling.rs` (function calling round-trip), `examples/structured_output.rs` (JSON schema response), `examples/responses_api.rs` (multi-turn with tools), `examples/predicted_output.rs` (code editing). All gated behind `live-tests`.
- [ ] Task 5.4: Update README.md with all new features. Bump version to 0.3.0. `make check`. Final commit.

## Review Criteria

WebFetch `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/chat/chat_completion.py` and compare ALL fields. Coverage must be ≥95% of Python SDK fields. If <95% → `<solo:redo/>`.
