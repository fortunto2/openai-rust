# Evolution Log

Factory defects and improvements from pipeline retros.

---

## 2026-03-20 | openai-rust | Factory Score: 7.0/10

Pipeline: setup→build→deploy→review (x5 runs, 6 tracks) | Iters: 26 | Waste: 38%
Tracks: phase1 (27m) → remaining (48m) → advanced (49m) → request-options (18m) → type-quality (41m)
Total: 77 commits, 151 tests, v0.5.0 published on crates.io, ~183 min wall time

### Defects
- **CRITICAL** — FIXED (0462049) | deploy (skills/deploy/SKILL.md): AskUserQuestion spin-loop in pipeline mode — 7 wasted deploy iterations (64% deploy waste). Crate name conflict (`openai-rust` taken on crates.io) triggered repeated user questions that can't be answered in pipeline. **3rd project with this exact defect** (supervox, openwok, openai-rust).
  - Fix: `skills/deploy/SKILL.md` — add Pipeline Mode section: when `$SOLO_PIPELINE=1` or `.solo/states/` exists, NEVER use AskUserQuestion. For naming conflicts: auto-pick from alternatives or read from Cargo.toml `[package] name`. For version bumps: auto-determine from git tags.

- **MEDIUM** | deploy (skills/deploy/SKILL.md): No Rust/crate publish strategy. Deploy skill designed for web apps (Vercel, Cloudflare, Fly.io) — has no `cargo publish` workflow. Agent improvised crate naming, LICENSE, README, and publish steps each time.
  - Fix: Add Rust/crate deploy section to SKILL.md: read `Cargo.toml`, verify name available on crates.io via `cargo search`, dry-run publish, publish, create GitHub release tag.

- **LOW** | solo-dev.sh: Review iter 11 forced-done at redo limit 2 — review found a real issue (stale README version) but was cut off. Redo limit is correct but the fix task was left unchecked in plan.
  - Fix: When forcing done at redo limit, add a note to plan.md with remaining issues.

### Harness Gaps
- **Context:** No `rust-native` or `rust-crate` stack YAML template — deploy had no reference for `cargo publish` workflow
- **Constraints:** Module boundaries respected throughout. Clean architecture (types/, resources/, client.rs).
- **Precedents (good):** Python SDK local-read workflow produced excellent type parity. OpenAPI validation tests caught gaps early. Auto-plan correctly identified and executed 2 additional tracks (request-options, type-quality) from backlog.
- **Precedents (bad):** Deploy AskUserQuestion blocking — 3rd consecutive project. Must be fixed at factory level.

### Missing
- `rust-crate` stack YAML template (cargo publish, docs.rs, crates.io, GitHub releases)
- Pipeline-mode detection in deploy skill (AskUserQuestion suppression)
- Auto-close superseded plan tasks on archiving

### What worked well
- **Build skill:** 5/6 productive (83%) — clean implementations, TDD followed, SHA annotations
- **Python SDK reference workflow:** Reading local Python source produced 100% field coverage
- **OpenAPI validation tests:** tests/openapi_coverage.rs caught missing fields proactively
- **Redo limit (2):** Prevented infinite review-build cycling, forced progress
- **Conventional commits:** 76/77 = 98.7% adherence
- **Progressive delivery:** v0.1.0 → v0.1.1 → v0.2.0 → v0.3.0 → v0.4.0 → v0.5.0
- **Auto-plan:** Correctly picked up request-options AND type-quality from backlog, both zero-waste (6 iters, 0 waste)
- **Runs 3-5 zero-waste:** Once deploy naming was resolved, pipeline ran cleanly (11 iters, 0 waste)
- **Type-quality track:** 19 new enums + 5 typed structs replacing weak types in a single clean 3-iter run
