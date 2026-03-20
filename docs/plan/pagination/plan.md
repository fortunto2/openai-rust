# Implementation Plan: Automatic Pagination

**Track ID:** pagination_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Add cursor-based pagination support to all list endpoints. Core infrastructure first (types, paginator stream, client helper), then update each resource, then tests.

## Phase 1: Pagination Infrastructure

Build the foundational types and the `Paginator<T>` async stream.

### Tasks

- [x] Task 1.1: Add `SortOrder` enum to `src/types/common.rs` <!-- sha:715f8d1 --> — `Asc`/`Desc` with `#[serde(rename_all = "snake_case")]`, `#[non_exhaustive]`
- [x] Task 1.2: Add `get_with_query()` method <!-- sha:715f8d1 --> to `src/client.rs` — like `get()` but accepts `&[(String, String)]` query params, used by list endpoints with pagination
- [x] Task 1.3: Add missing pagination fields <!-- sha:715f8d1 --> to list response types — add `has_more: Option<bool>`, `first_id: Option<String>`, `last_id: Option<String>` to `FileList`, `AssistantList`, `MessageList`, `VectorStoreList` in their respective type files. Update `BatchList` to use `Option<bool>` instead of `bool`. Files: `src/types/file.rs`, `src/types/beta.rs`, `src/types/batch.rs`
- [x] Task 1.4: Create `src/pagination.rs` <!-- sha:715f8d1 --> — `Paginator<T>` struct implementing `Stream<Item = Result<T, OpenAIError>>`. Uses a boxed async closure to fetch next page given an `after` cursor. Holds current page data and drains items before fetching next. Re-export from `src/lib.rs`
- [x] Task 1.5: Create list param types <!-- sha:715f8d1 --> in each resource's type file — `FileListParams` (after, limit, order, purpose), `BatchListParams` (after, limit), `FineTuningJobListParams` (after, limit), `FineTuningEventListParams` (after, limit), `AssistantListParams` (after, before, limit, order), `MessageListParams` (after, before, limit, order, run_id), `VectorStoreListParams` (after, before, limit, order). Each with builder pattern methods. Files: `src/types/file.rs`, `src/types/batch.rs`, `src/types/fine_tuning.rs`, `src/types/beta.rs`

### Verification

- [x] `cargo check` passes with new types
- [x] `Paginator<T>` compiles and implements `Stream + Unpin + Send`

## Phase 2: Update Resource List Methods

Add `list_page(params)` and `list_auto(params)` to each resource.

### Tasks

- [x] Task 2.1: Update `src/resources/files.rs` — add `list_page(params: FileListParams)` returning `Result<FileList>` and `list_auto(params: FileListParams)` returning `Paginator<FileObject>`
- [x] Task 2.2: Update `src/resources/batches.rs` — add `list_page(params: BatchListParams)` and `list_auto(params: BatchListParams)` returning `Paginator<Batch>`
- [x] Task 2.3: Update `src/resources/fine_tuning.rs` — add `list_page(params)` and `list_auto(params)` for both jobs and job events
- [x] Task 2.4: Update beta resources — add `list_page(params)` and `list_auto(params)` to `src/resources/beta/assistants.rs`, `src/resources/beta/threads.rs` (messages), `src/resources/beta/vector_stores.rs`, `src/resources/beta/runs.rs`

### Verification

- [x] `cargo check` passes
- [x] `cargo clippy -- -D warnings` clean

## Phase 3: Tests

### Tasks

- [ ] Task 3.1: Unit tests for `Paginator<T>` in `src/pagination.rs` — test single-page (has_more=false), multi-page (2-3 pages), empty page, error propagation
- [ ] Task 3.2: Mockito integration tests for `list_page()` and `list_auto()` — test files and batches with multi-page responses, verify correct `after` query param is sent, verify all items collected
- [ ] Task 3.3: Update OpenAPI coverage tests — add pagination fields (has_more, first_id, last_id) to list response coverage in `tests/openapi_coverage.rs`

### Verification

- [ ] `cargo test` all pass
- [ ] `cargo test --test openapi_coverage -- --nocapture` shows no regression

## Phase 4: Docs & Cleanup

### Tasks

- [ ] Task 4.1: Update CLAUDE.md — add `pagination.rs` to architecture, add `Paginator` to implemented APIs, note `list_page()`/`list_auto()` pattern
- [ ] Task 4.2: Update README.md — add pagination usage example showing `list_page()` with params and `list_auto()` with `StreamExt::collect()`
- [ ] Task 4.3: Remove dead code — check for unused imports, verify no orphaned types

### Verification

- [ ] CLAUDE.md reflects current project state
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo test` all pass

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] Tests pass
- [ ] Linter clean
- [ ] Build succeeds
- [ ] Documentation up to date

## Context Handoff

_Summary for /build to load at session start — keeps context compact._

### Session Intent

Add cursor-based automatic pagination to all list endpoints, matching Python SDK's CursorPage pattern.

### Key Files

- `src/pagination.rs` (NEW) — Paginator<T> stream
- `src/client.rs` — add get_with_query() helper
- `src/lib.rs` — re-export Paginator
- `src/types/common.rs` — SortOrder enum
- `src/types/file.rs` — FileListParams + pagination fields on FileList
- `src/types/batch.rs` — BatchListParams + pagination fields on BatchList
- `src/types/fine_tuning.rs` — FineTuningJobListParams, FineTuningEventListParams
- `src/types/beta.rs` — AssistantListParams, MessageListParams, VectorStoreListParams + pagination fields
- `src/resources/files.rs` — list_page(), list_auto()
- `src/resources/batches.rs` — list_page(), list_auto()
- `src/resources/fine_tuning.rs` — list_page(), list_auto() for jobs and events
- `src/resources/beta/assistants.rs` — list_page(), list_auto()
- `src/resources/beta/threads.rs` — list_page(), list_auto() for messages
- `src/resources/beta/vector_stores.rs` — list_page(), list_auto()
- `src/resources/beta/runs.rs` — list_page(), list_auto()

### Decisions Made

- **Keep existing `list()` unchanged** — backward compat. New methods: `list_page(params)` for single page with params, `list_auto(params)` for auto-paging stream.
- **Per-resource param types** (not one generic) — each endpoint has different params (files has `purpose`, beta has `before`, etc.)
- **`Paginator<T>` uses boxed closure** — avoids complex generic gymnastics. Closure captures client ref and builds the next request.
- **Cursor from `last_id` field** — fall back to last item's `id` if `last_id` not present in response.
- **Skip `ModelList`** — `/models` is not paginated by the OpenAI API.
- **`SortOrder`** not `Order` — avoids collision with `std::cmp::Ordering`.

### Risks

- `Paginator` lifetime management — it needs to borrow `&OpenAI` across `.await` points. Will use `Arc<OpenAI>` clone or `OpenAI` clone (cheap since `reqwest::Client` is `Arc` internally).
- Beta endpoints need `OpenAI-Beta: assistants=v2` header on every page fetch — the paginator's fetch closure must include it.
- `RunListParams` may need `thread_id` embedded — runs are scoped to a thread.

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
