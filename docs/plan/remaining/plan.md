# openai-sdk — Remaining Endpoints

**Status:** [~] In Progress
**Track:** remaining

## Context Handoff

**Intent:** Achieve 100% API parity with openai-python. Currently only Chat Completions is implemented. Must cover ALL remaining endpoints.

**Method:** For EACH task below:
1. `WebFetch` the Python SDK source from GitHub to see exact types and methods: `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/resources/{resource}.py` and `https://raw.githubusercontent.com/openai/openai-python/main/src/openai/types/{type_file}.py`
2. Create matching Rust types with serde
3. Create resource module with client methods
4. Write mockito tests
5. Commit

**Also:** Rename crate from `openai-rust` to `openai-sdk` in Cargo.toml.

**What's DONE:** client.rs, config.rs, error.rs, streaming.rs, chat completions (create + stream + tools). 29 tests.

**What's MISSING (check Python SDK `src/openai/resources/` for the full list):**

---

## Phase 0: Review Fix Tasks (from review 2026-03-20)

- [x] Task 0.1: Fix lib.rs doc comment — replace all `openai_rust` → `openai_oxide` in the doc example <!-- sha:f8eff0e -->
- [x] Task 0.2: Fix examples/chat.rs — replace `use openai_rust::` → `use openai_oxide::` <!-- sha:f8eff0e -->
- [x] Task 0.3: Fix examples/chat_stream.rs — replace `use openai_rust::` → `use openai_oxide::` <!-- sha:f8eff0e -->
- [x] Task 0.4: Fix README.md — replace all `openai-rust`/`openai_rust` → `openai-oxide`/`openai_oxide`, update Cargo.toml dep to `openai-oxide = "0.1"`, fix Configuration example <!-- sha:f8eff0e -->
- [x] Task 0.5: Run `cargo test` (all tests including doc-tests and examples) — must pass 30/30 <!-- sha:f8eff0e -->
- [x] Task 0.6: Publish `openai-oxide` v0.1.1 with fixes to crates.io <!-- sha:c16a5be -->

---

- [x] Task 1.1: Rename crate to `openai-oxide` in Cargo.toml. <!-- done via deploy -->
- [x] Task 1.2: WebFetch Python `resources/embeddings.py` + `types/embedding.py`. Implement `src/types/embedding.rs` + `src/resources/embeddings.rs`. Methods: `client.embeddings().create(model, input)`. Mockito test.
- [x] Task 1.3: WebFetch Python `resources/models.py` <!-- sha:26ae0da --> + `types/model.py`. Implement `src/types/model.rs` + `src/resources/models.rs`. Methods: `list()`, `retrieve(id)`, `delete(id)`. Mockito tests.
- [x] Task 1.4: WebFetch Python `resources/moderations.py` <!-- sha:7a1e120 --> + `types/moderation.py`. Implement `src/types/moderation.rs` + `src/resources/moderations.rs`. Method: `create(input)`. Mockito test.
- [x] Task 1.5: WebFetch Python `resources/images.py` <!-- sha:48e04f4 --> + `types/image.py`. Implement `src/types/image.rs` + `src/resources/images.rs`. Methods: `generate()`, `edit()`, `create_variation()`. Mockito tests.
- [x] Task 1.6: WebFetch Python `resources/audio/transcriptions.py` <!-- sha:1ed0246 --> + `types/audio/transcription.py`. Implement `src/types/audio.rs` + `src/resources/audio.rs`. Method: `transcriptions().create(file, model)`. Multipart upload. Mockito test.
- [x] Task 1.7: WebFetch Python `resources/audio/speech.py` <!-- sha:1ed0246 -->. Add `speech().create(input, voice, model)` → returns bytes. Mockito test.
- [x] Task 1.8: WebFetch Python `resources/audio/translations.py` <!-- sha:1ed0246 -->. Add `translations().create(file, model)`. Multipart. Mockito test.
- [x] Task 1.9: WebFetch Python `resources/files.py` <!-- sha:a698f4d --> + `types/file_object.py`. Implement `src/resources/files.rs`. Methods: `create(file, purpose)`, `list()`, `retrieve(id)`, `delete(id)`, `content(id)`. Mockito tests.
- [x] Task 1.10: WebFetch Python `resources/fine_tuning/jobs.py` <!-- sha:aeed41e --> + `types/fine_tuning/`. Implement `src/resources/fine_tuning.rs`. Methods: `jobs().create()`, `list()`, `retrieve(id)`, `cancel(id)`, `list_events(id)`. Mockito tests.
- [x] Task 1.11: WebFetch Python `resources/responses/responses.py` <!-- sha:f10dbf0 --> + `types/responses/`. Implement `src/resources/responses.rs`. Method: `create()` with tools, instructions, previous_response_id. Support streaming. Mockito tests.
- [x] Task 1.12: WebFetch Python `resources/beta/assistants.py` <!-- sha:65eb210 --> + `types/beta/assistant.py`. Implement `src/resources/assistants.rs`. Methods: `create()`, `list()`, `retrieve()`, `update()`, `delete()`. Mockito tests.
- [x] Task 1.13: WebFetch Python `resources/beta/threads/` <!-- sha:65eb210 -->. Implement `src/resources/threads.rs`. Methods: `create()`, `retrieve()`, `update()`, `delete()`. Sub-resources: `messages.create()`, `messages.list()`. Mockito tests.
- [x] Task 1.14: WebFetch Python `resources/beta/threads/runs/` <!-- sha:65eb210 -->. Implement `src/resources/runs.rs`. Methods: `create()`, `retrieve()`, `cancel()`, `submit_tool_outputs()`. Mockito tests.
- [x] Task 1.15: WebFetch Python `resources/beta/vector_stores/` <!-- sha:65eb210 -->. Implement `src/resources/vector_stores.rs`. Methods: `create()`, `list()`, `retrieve()`, `delete()`. Sub: `file_batches.create()`. Mockito tests.
- [x] Task 1.16: WebFetch Python SDK <!-- sha:0af88a3 --> `src/openai/resources/` directory listing. Compare ALL modules vs what we have. If ANY resource is missing — implement it. This is the final coverage check.
- [x] Task 1.17: Update README.md with ALL endpoints table <!-- sha:a4d240d -->. Run `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check`. Final commit.

---

## Phase 2: Review Fix Tasks (from review 2026-03-20)

- [ ] Task 2.1: Fix README.md — update `openai-oxide = "0.1"` to `openai-oxide = "0.2"` in Quick Start Cargo.toml example
