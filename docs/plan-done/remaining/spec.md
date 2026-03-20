# Remaining Endpoints — Acceptance Criteria

Cover ALL OpenAI API endpoints to reach 100% parity with openai-python.
Each endpoint: types (Serialize+Deserialize) + resource method + mockito test.
Study Python SDK via WebFetch for exact field names.

## Review Criteria (MANDATORY before <solo:done/>)

The `/review` stage MUST perform a full parity check:

1. **WebFetch** `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/resources/__init__.py` — get the list of ALL resource modules.
2. **WebFetch** each resource's Python file and compare with our Rust implementation:
   - Every public method in Python must have a Rust equivalent
   - Every request/response type field must be present
   - Streaming variants must exist where Python has them
3. **Report a coverage table:**

| Resource | Python methods | Rust methods | Coverage |
|----------|---------------|-------------|----------|
| chat.completions | create, create_stream | ? | ?% |
| embeddings | create | ? | ?% |
| ... | ... | ... | ... |

4. If coverage < 90% → output `<solo:redo/>` with list of missing endpoints.
5. If coverage >= 90% → output `<solo:done/>`.

Crate name: `openai-oxide`.
