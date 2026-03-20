# Advanced Features — Acceptance Criteria

- Responses API: streaming, all tool types (web_search, file_search, computer_use, mcp, code_interpreter), conversation chaining
- Predicted outputs: `prediction` field in chat completions
- Prompt caching: `cached_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens` in usage
- Reasoning effort: `low/medium/high` for chat + responses
- Structured outputs: strict JSON schema for both response_format and tools
- Realtime API: session creation + ephemeral token
- Builder pattern for ChatCompletionRequest and ResponseCreateRequest
- Examples for tool calling, structured output, responses API, predicted output
- ≥95% field coverage vs Python SDK (verified via WebFetch comparison)
- `make check` passes, version bumped to 0.3.0
