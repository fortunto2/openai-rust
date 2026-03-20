# Implementation Plan: Type Quality — Zero serde_json::Value, Typed Enums

**Track ID:** type-quality_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Replace weak types (`serde_json::Value`, `String` for known finite sets) with proper Rust enums and structs. Work bottom-up: enums first (no deps), then Value replacements (may use new enums), then tests, then cleanup.

## Phase 1: String → Enum Conversions

Add typed enums for all String fields where the Python SDK uses `Literal[...]`. Each enum gets `#[non_exhaustive]`, `#[serde(rename_all = "snake_case")]`, `Clone`, `Debug`, `Serialize`, `Deserialize`.

### Tasks

- [x] Task 1.1: Add status/reason enums to `src/types/common.rs` <!-- sha:c9f7058 --> — `FinishReason` (stop, length, tool_calls, content_filter, function_call), `ServiceTier` (auto, default, flex, scale, priority), `ReasoningEffort` (low, medium, high), `SearchContextSize` (low, medium, high). Update `src/types/chat.rs` fields to use them.
- [~] Task 1.2: Add file/batch/upload status enums — `FilePurpose` and `FileStatus` in `src/types/file.rs`, `BatchStatus` in `src/types/batch.rs`, `UploadStatus` in `src/types/upload.rs`. Update corresponding struct fields.
- [ ] Task 1.3: Add fine-tuning and beta status enums — `FineTuningStatus` and `FineTuningEventLevel` in `src/types/fine_tuning.rs`, `RunStatus` and `VectorStoreStatus` in `src/types/beta.rs`. Update struct fields.
- [ ] Task 1.4: Add image enums — `ImageQuality`, `ImageSize`, `ImageStyle`, `ImageOutputFormat`, `ImageBackground`, `ImageModeration` in `src/types/image.rs`. Update `ImageGenerateRequest` fields.
- [ ] Task 1.5: Add audio/embedding enums — `AudioResponseFormat`, `SpeechResponseFormat`, `AudioVoice`, `AudioFormat`, `InputAudioFormat` in `src/types/audio.rs`, `EncodingFormat` in `src/types/embedding.rs`, `ImageDetail` in `src/types/chat.rs`. Update fields.
- [ ] Task 1.6: Add realtime enums — `RealtimeAudioFormat`, `TurnDetectionType`, `Eagerness` in `src/types/realtime.rs`. Update struct fields.

### Verification

- [ ] `cargo test` — all existing tests pass
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] Grep: no `String` fields where Python SDK uses `Literal[...]` for status/format/role/reason

## Phase 2: serde_json::Value → Typed Replacements

Replace Value fields with proper typed enums/structs. Group by pattern.

### Tasks

- [ ] Task 2.1: Add `AutoOrFixed<T>` enum in `src/types/common.rs` — handles "auto" string OR numeric value. Custom `Serialize`/`Deserialize`. Use for `n_epochs` (i64), `batch_size` (i64), `learning_rate_multiplier` (f64) in `src/types/fine_tuning.rs`.
- [ ] Task 2.2: Add `MaxResponseTokens` enum (Inf/Fixed(i64)) in `src/types/common.rs` — handles "inf" string OR integer. Use for `max_response_output_tokens` in `src/types/realtime.rs` (2 fields).
- [ ] Task 2.3: Replace Value fields in `src/types/responses.rs` — `user_location` → `UserLocation` struct, `ranking_options` → `RankingOptions` struct, `require_approval` → `ApprovalConfig` enum, `container` → `ContainerConfig` struct, `allowed_tools` → `Vec<String>`. Read Python SDK `~/startups/shared/openai-python/src/openai/types/responses/` for exact shapes.
- [ ] Task 2.4: Replace `function_call: Option<serde_json::Value>` in `src/types/chat.rs` → `FunctionCallOption` enum (None/Auto/Named { name }). Match existing `ToolChoice` pattern.
- [ ] Task 2.5: Replace `data: Option<serde_json::Value>` in `src/types/fine_tuning.rs` FineTuningJobEvent → `serde_json::Value` stays (genuinely unstructured event data — add doc comment explaining why).

### Verification

- [ ] `cargo test` — all tests pass
- [ ] `serde_json::Value` count in `src/types/` ≤ 7 (5 parameters + event data + PredictionContent.content)
- [ ] Deserialization round-trip tests for `AutoOrFixed`, `MaxResponseTokens`, `FunctionCallOption`

## Phase 3: OpenAPI Coverage Expansion

Expand `tests/openapi_coverage.rs` to validate new enum fields against the spec.

### Tasks

- [ ] Task 3.1: Add coverage tests for File, Batch, Upload, FineTuning schemas — validate status enum values match spec.
- [ ] Task 3.2: Add coverage tests for Image, Audio, Embedding schemas — validate format/quality/size enum values match spec.
- [ ] Task 3.3: Add deserialization round-trip tests for new types — `AutoOrFixed`, `MaxResponseTokens`, `FunctionCallOption`, `UserLocation`, `RankingOptions` in `tests/` or inline.

### Verification

- [ ] OpenAPI coverage ≥8 schemas tested (up from 4)
- [ ] All new enum variants match OpenAPI spec values

## Phase 4: Docs & Cleanup

### Tasks

- [ ] Task 4.1: Update CLAUDE.md — add new enums to Architecture section, update "Implemented APIs" if needed, note the type quality improvements.
- [ ] Task 4.2: Update pre-commit quality check threshold — `serde_json::Value` target count from current to ≤7.
- [ ] Task 4.3: Remove dead code — unused imports, stale type aliases, orphaned helper functions from refactoring.

### Verification

- [ ] CLAUDE.md reflects current project state
- [ ] `make check` passes (fmt + clippy + test)
- [ ] Pre-commit hook passes

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] Tests pass (141+ existing + new)
- [ ] Linter clean
- [ ] Build succeeds
- [ ] `serde_json::Value` count in types/ ≤ 7

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
