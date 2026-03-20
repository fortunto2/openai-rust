# Specification: Type Quality — Zero serde_json::Value, Typed Enums

**Track ID:** type-quality_20260320
**Type:** Refactor
**Created:** 2026-03-20
**Status:** Draft

## Summary

Replace all `serde_json::Value` in public types with typed structs/enums and convert 40+ `String` fields to proper Rust enums where the OpenAI API defines a finite set of values. This is Priority 1 on the roadmap ("make Knuth proud") and the key differentiator vs async-openai.

Currently: 21 `serde_json::Value` fields and ~41 `String` fields that should be enums. Target: 0 Value fields in easy/medium category (keep ≤5 for genuinely polymorphic JSON Schema fields), 0 String fields where Python SDK uses `Literal[...]`.

## Acceptance Criteria

- [x] All `serde_json::Value` fields in types/ replaced with typed alternatives (except `parameters: Option<serde_json::Value>` for JSON Schema — 5 fields max)
- [x] All status/role/format/finish_reason String fields replaced with `#[non_exhaustive]` enums
- [x] New enums: `FinishReason`, `ServiceTier`, `ReasoningEffort`, `FilePurpose`, `FileStatus`, `BatchStatus`, `UploadStatus`, `FineTuningStatus`, `RunStatus`, `VectorStoreStatus`, `ImageQuality`, `ImageSize`, `ImageStyle`, `ImageOutputFormat`, `AudioFormat`, `AudioVoice`, `EncodingFormat`, `ImageDetail`, `SearchContextSize`
- [x] New typed structs for Value replacements: `HyperparameterValue` (auto/fixed), `MaxResponseTokens` (inf/fixed), `UserLocation`, `RankingOptions`, `ApprovalConfig`
- [x] OpenAPI coverage tests expanded to ≥8 schemas (from current 4)
- [x] All 141+ existing tests still pass
- [x] `cargo clippy -- -D warnings` clean
- [x] Pre-commit quality checks pass (serde_json::Value count ≤ 5)

## Dependencies

- Python SDK reference: `~/startups/shared/openai-python/src/openai/types/`
- OpenAPI spec: `tests/openapi.yaml`
- No new external crate dependencies

## Out of Scope

- Comprehensive doc comments on all 567 fields (separate track)
- Polymorphic streaming event `data` field (ResponseStreamEvent.data) — complex, separate track
- ResponseInputItem.content polymorphic union — complex, separate track
- JSON Schema `parameters` fields — legitimately `serde_json::Value`
- PredictionContent.content — complex union type

## Technical Notes

- All new enums get `#[non_exhaustive]` + `#[serde(rename_all = "snake_case")]`
- `HyperparameterValue` pattern: `enum { Auto, Fixed(T) }` with custom serde (deserialize "auto" string or number)
- Python SDK uses `Union[Literal["auto"], int]` for hyperparameters — same pattern
- `parameters` fields (5 occurrences in chat, beta, responses, realtime) stay as `serde_json::Value` because they hold arbitrary JSON Schema
- Pre-commit hook already counts Value fields — threshold must be updated from current to ≤5
