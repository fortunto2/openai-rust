# Specification: Automatic Pagination

**Track ID:** pagination_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

All list endpoints (`files`, `batches`, `fine_tuning/jobs`, `assistants`, `threads/messages`, `vector_stores`, `runs`) currently return only the first page of results with no way to request subsequent pages. The OpenAI API supports cursor-based pagination via `after`, `limit`, `order` query params and returns `has_more`, `first_id`, `last_id` in responses.

This track adds:
1. Missing pagination fields to all list response types
2. Per-resource list parameter builders (after, limit, order, purpose, etc.)
3. `list_page(params)` methods that accept pagination parameters
4. A `Paginator<T>` async stream that auto-fetches all pages, matching the Python SDK's `CursorPage` pattern

## Acceptance Criteria

- [x] All list response types include `has_more`, `first_id`, `last_id` fields (where API supports them)
- [x] Every paginated list endpoint accepts `after`, `limit` params (and `order`/`before` where applicable)
- [x] `Paginator<T>` implements `futures_core::Stream<Item = Result<T, OpenAIError>>`
- [x] `list_auto()` methods return a `Paginator<T>` that fetches subsequent pages automatically
- [x] Existing `list()` methods remain unchanged (backward compatible)
- [x] `SortOrder` enum added (`Asc`, `Desc`) with serde rename
- [x] Mockito tests for multi-page pagination (2+ pages with `has_more: true` → `has_more: false`)
- [x] OpenAPI coverage tests updated for new pagination fields

## Dependencies

- `futures-core` 0.3 (already in Cargo.toml)
- `futures-util` 0.3 (already in Cargo.toml)
- No new external dependencies needed

## Out of Scope

- `before` parameter for backward pagination (beta endpoints support it, but auto-paging only goes forward)
- `ModelList` — models endpoint is not paginated by the API
- Sync/blocking pagination (this crate is async-only)
- `ConversationCursorPage` variant (not used by current endpoints)

## Technical Notes

- **Python SDK pattern:** `CursorPage<T>` extracts cursor from `last_id` field or last item's `id`. We'll use `last_id` from response when available, fall back to last item's `id`.
- **Generic approach:** `Paginator<T>` needs a way to fetch the next page. It holds a closure/fn pointer that takes an `after` cursor and returns a future.
- **Zero-cost for single page:** Users who only call `list()` or `list_page()` pay no overhead — `Paginator` is only constructed when `list_auto()` is called.
- **Stream + Unpin:** Like `SseStream<T>`, `Paginator<T>` will be `Unpin` and `Send` for ergonomic use with `StreamExt`.
- **Client needs `get_with_query()`:** Current `get()` helper doesn't support query params. We need a variant that accepts query parameters for list endpoints.
