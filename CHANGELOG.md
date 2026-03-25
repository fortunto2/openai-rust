# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [0.9.8]

### Added
- Conversations and Videos (Sora) resources (`c3a9f16`)
- Missing API methods: `cancel`, `input_items`, `count_tokens`, and stored completions (`2948d59`)
- 9 additional missing API endpoints to close coverage gaps (`d9bc105`)
- Drop-in replacement examples matching the official OpenAI SDK (`78dad82`)
- Drop-in `AsyncOpenAI` compat layer for Python + `create_chat_raw` (`ca5e8d3`)
- Drop-in OpenAI compat layer for Node.js matching official SDK (`3b51a59`)
- Benchmarks README and Makefile targets (`2fc7981`)
- WASM compilation check added to pre-commit, CI, and Makefile (`0cb14d4`)

### Changed
- Updated README and CLAUDE.md models updated to `gpt-5.4`, new APIs, drop-in compat notes (`ad36e70`)
- Docs audit updated models to `gpt-5.4`, included examples from files (`68ed766`)
- Single-sourced benchmarks via JSON + generate + include (`7c6b71a`)
- Updated Node README with latest benchmark numbers (8/8 wins) (`8e9a61f`)
- Updated Python and Node benchmarks median of 3 runs on `gpt-5.4` (`8b1a3e3`)
- Docs updated for structured outputs, stream helpers, and webhooks guides (`c7777b2`)
- Highlighted Structured Outputs and Stream Helpers in top features (`da9b758`)
- Simple Cloudflare Worker updated workspace fix, model set to `gpt-5.4-mini` (`217b97b`)

### Fixed
- Node release workflow to accept both `v*` and `node-v*` tags (`00bbf75`)
- Dioxus frontend rebuild added JS/WASM assets to dist (`217b97b`)
- Full WebAssembly compatibility pagination, files, uploads (`3b2f679`)
- WASM compatibility + deployed Cloudflare Worker demo (`91f84f2`)

---

## [0.9.7]

### Added
- Structured Outputs for Rust: `parse::<T>()` with JSON schema generation (`001da52`)
- Structured Outputs for Responses API: `parse::<T>()` (`7f33d49`)
- High-level stream helpers with typed events and accumulation (`ba4cbe5`)
- Webhook verification, request ID, file upload from path, prompt caching (`464f000`)
- `createChatParsed`, `createResponseParsed` with Zod support for Node.js (`540bef0`)
- `create_parsed()` with Pydantic `BaseModel` support for Python (`2057c13`)
- Live feature test hitting real OpenAI API (`gpt-5.4-mini`) (`febc997`)
- Transcription sessions API, Items input, WebSocket diagnostics (`ae144e7`)

### Changed
- Unified release single `v*` tag now triggers Rust + Node + Python releases (`205506d`)
- Robust `ensure_strict` handles nullable, allOf, items (`0f87227`)
- Docs updated for Structured Outputs, Stream Helpers, Webhooks sections in README (`70d5665`)
- Docs added showing raw JSON fallback for Node/Python in mapping table (`d841de4`)

### Fixed
- Replaced example webhook secret flagged by GitHub secret scanner (`38ff193`)
- Python strict schema enforcement + Pydantic v2 only (`701b3a6`)
- Eliminated panics, added streaming retry, typed stream events (`0112b6e`)
- WebSocket: auto-strip decimal temperature (OpenAI bug workaround) (`a9e240b`)
- Added `Accept: text/event-stream` header to prevent reverse proxy buffering and improve TTFT (`1c57fcf`)
- Fallback to HTTP `/chat/completions` route if non-OpenAI `base_url` doesn't support WSS Responses API (`933e65a`)

---

## [0.9.6]

### Added
- Socket badge and `SECURITY.md` (`077d276`)
- Guide badge, documentation URL, and repo homepage (`5dd3f9f`)
- `context7.json` for Context7 indexing (`e477144`)
- Auto-deploy mdbook docs to GitHub Pages (`e3633af`)
- mdbook site, `llms.txt`, OpenAI docs mapping, and docstring links (`d1de225`)

### Changed
- Replaced hardcoded examples with `{{#include}}` from `examples/` in docs (`6d8b2ca`)
- Updated logo in README header (`54cd557`)
- Updated Node and Python READMEs with install instructions (`dd97809`)
- Added installation section for all package managers (`7eb2324`)
- Added doc coverage check to pre-commit, updated `CLAUDE.md` (`a3c35a2`)

### Fixed
- CI release: use `taiki-e/install-action` for mdbook install (`e477144`)
- Fixed OpenAI docs URL for Responses guide in docs (`40f5acf`)
- CI release configuration using `--allow-dirty` for `Cargo.lock` (`874e848`)

---

## [0.9.5]

### Added
- Node.js benchmark against official SDK (`6707153`)
- Highlighted WebSocket benefits in Node.js bindings (`e1d3529`)

### Fixed
- Node: match singular `version` in already-published detection (`5d27e9d`)
- Python crate made its own workspace root (`2b8d104`)
- Excluded Python crate from workspace to unblock release (`6018c71`)
- Fixed publish order macros crate published before main crate (`e836f7b`)
- Handled npm publish rate limits (`2da57c8`)
- Node: publish platform packages without GitHub release (`ec4dfab`)
- Node: prepare manifests during release (`a5da45c`)
- Node: use Zig for Linux arm64 release builds (`9e10cc8`)
- Switched rustls provider and updated repo URLs (`5426373`)
- Synced publish artifacts for 0.9.4 (`c37c71e`)
- Pinned pnpm version in workflows (`9d29a33`)

---

## [0.9.4]

### Added
- Tools builder, reasoning models validation, developer role, usage metrics in stream (`cca53dc`)
- Middlewares, dynamic Config, proc-macros, and Node.js bindings (`bd85a58`)
- Initial napi-rs bindings for Node.js and TypeScript (`43f1ab9`)
- Python bindings via PyO3 + maturin (`openai-oxide-python`) (`5ff84b8`)
- Full-stack Rust example (Dioxus + Cloudflare Worker DO + WebSocket) (`c95697b`)
- Streaming async iterator in PyO3 bindings (`7af7011`)
- Optional `simd-json` for SIMD-accelerated response parsing (`4d6723c`)
- `prompt_cache_key`, 8-test benchmark oxide wins 5/8 vs Python (`4f20481`)
- `make sync` automated OpenAPI spec drift detection (`1555ac9`)

### Changed
- Enabled verbose PyPI upload (`fb6e379`)
- Renamed npm package to `openai-oxide` (`efc4728`)
- Renamed import module to `openai_oxide` for cleaner Python usage (`22a2acd`)
- Renamed PyPI package to `openai-oxide-python` to match import path (`f1a0167`)
- Updated Python package version to match Rust core and added PyPI metadata (`a520b73`)
- Roadmap: added Tauri and HTMX+Axum entries (`6ba1d3f`)
- Roadmap: finalized Node.js napi bindings section (`f46b7dc`)
- WASM UI: custom base URL and flexible model selection (`79de38d`)
- Dioxus app: prompt caching with chat history + model selector (`e55f073`)

### Fixed
- Python abi3 compatibility for Python 3.14, skip crates.io if token missing (`4e165cc`)
- Invalid use of secrets in `if` condition in CI (`fb6e379`)
- Fixed target args for maturin build (`754aa77`)
- Fixed macOS runner version in CI (`4a4680c`)
- Hybrid WS/HTTP fallback for non-native WS upstream providers (`ebaa7f5`)
- Fallback to HTTP route for non-OpenAI base URLs that don't support WSS (`933e65a`)
- Added `sdist` generation to release pipeline (`3eaefb4`)
- Added GitHub Actions for tests and cross-platform PyPI/crates.io publishing (`0e36d06`)

---

## [0.9.3]

### Added
- Benchmarks comparing openai-oxide vs async-openai vs genai in Rust (`698a286`)
- TTFT display, tokens per second speed, and API key query loading in WASM UI (`83577a6`)
- WebSocket mode, hedged requests, and speculative execution (`51c3806`)
- Streaming TTFT, parallel fan-out, hedged requests benchmark (`ea980a8`)
- Robust `StreamFcHandle` with timeout, error reporting, typed API (`0a76d1d`)
- Streaming FC early parse 38% faster tool calls (`f2affc1`)
- Cargo features section and WASM optimization docs (`7247c1f`)

### Changed
- Bumped version to 0.9.3 for unified release (`2bd2d61`)
- Updated benchmarks separating chat and Responses API for fair comparison (`7b13e22`)
- Corrected benchmark numbers for streaming TTFT in async-openai (`960d341`)
- Updated Python installation instructions to use PyPI (`1f833e3`)
- Enriched WASM + Cloudflare Worker example READMEs (`206b10f`)

### Fixed
- WebSocket: auth via Bearer header, `response.create` wrapper, live-tested (`f2affc1`)

---

## [0.9.0]

### Added
- WebSocket mode, hedged requests, streaming function calling, reqwest 0.13 (`d5b1e69`)
- Streaming FC early parse 38% faster tool calls (`f2affc1`)
- HTTP/2 keep-alive + adaptive window (`55b036f`)
- Fast-path retry, statistical benchmark (5 iterations) (`898612a`)
- Upgrade reqwest 0.12 → 0.13 (HTTP/2, query feature) (`1fa1aa9`)
- gzip, `tcp_nodelay`, Response helpers, tracing on deserialization errors (`f779237`)

---

## [0.8.0]

### Added
- `list_page()` and `list_auto()` to all resources (`9e1160d`)
- Pagination infrastructure types, Paginator stream, list params (`715f8d1`)

### Changed
- Updated CLAUDE.md, README, plan pagination track complete (`c0941d2`)

---

## [0.7.0]

### Added
- `Image::save()` helper for `b64_json` and URL responses (`34f4a35`)
- BYOT `create_raw()` methods to chat, responses, embeddings (`8549379`)
- `post_json` helper for BYOT raw methods (`5cac612`)
- Granular feature flags for API resources (`2213e4a`)
- Examples gated behind required features (`596f8f3`)

### Changed
- Completed ergonomics track all phases done (`c2ebfb4`)
- Updated CLAUDE.md, README, and roadmap for ergonomics track (`1c43c25`)

---

## [0.6.0]

### Added
- `AzureConfig` builder and `OpenAI::azure()` constructor (`b22ce82`)
- 14 tests for Azure OpenAI support (`54cd0ba`)

### Changed
- Completed Azure support track (`5474e46`)
- Updated CLAUDE.md and roadmap for Azure support (`f3292f0`)

---

## [0.5.0]

### Added
- `AutoOrFixed<T>` and `MaxResponseTokens` enums (`4a12661`)
- Image, audio, embedding, and realtime enums (`1fa194a`)
- `FineTuningStatus`, `FineTuningEventLevel`, `RunStatus`, `VectorStoreStatus` enums (`3a539b8`)
- `FilePurpose`, `FileStatus`, `BatchStatus`, `UploadStatus` enums (`ad0c4ca`)
- `FinishReason`, `ServiceTier`, `ReasoningEffort`, `SearchContextSize` enums (`c9f7058`)
- 10 coverage tests for new enums and typed replacements (`1c49204`)
- `#[non_exhaustive]` on all 10 public enums (`78ea712`)
- `Role` enum replacing String role fields across chat/responses/beta (`93b8de8`)

### Changed
- Replaced `serde_json::Value` fields in responses/chat/fine_tuning with typed structs (`3fd60e2`)
- Replaced `serde_json::Value` with typed structs in `responses.rs` (`e040644`)
- Replaced `serde_json::Value` with typed structs in `beta.rs` (`e1e6989`)
- Completed type-quality track updated docs and quality threshold (`1ce281c`)
- Added quality checks to pre-commit (serde_json::Value count, String→enum audit) (`1c26afc`)

### Fixed
- 6 missing GPT image model fields added to `ImageGenerateRequest` (`53ac94a`)

---

## [0.4.0]

### Added
- `RequestOptions` and `with_options()` for per-request customization (`8789d75`)
- 7 mockito tests for `RequestOptions` and `with_options()` (`0aee89a`)

### Changed
- Updated architecture docs with `request_options` module (`0aee89a`)
- Completed request-options track all phases done (`678ec4d`)

---

## [0.3.0]

### Added
- Realtime API, examples (`601bc55`)
- Builder pattern for `ChatCompletionRequest` and `ResponseCreateRequest` (`dfa5a42`)
- Full power for Responses API tools, reasoning, streaming, all fields (`acb5ce0`)
- Missing fields for 100% OpenAPI coverage in chat (`17efd5d`)
- OpenAPI coverage tests + pre-commit hook (`1abba49`)
- Architecture patterns docs Config trait, Middleware, Paginator, RetryPolicy (`e1e6989`)

### Changed
- Expanded roadmap beat async-openai on type safety, Azure, BYOT, middleware (`78ea712`)
- Excluded non-essential files from crate package (`d1bbb87`)

---

## [0.2.0]

### Added
- Batches and Uploads APIs (`0af88a3`)
- Assistants, Threads, Runs, Vector Stores APIs (Beta) (`65eb210`)
- Responses API (`f10dbf0`)
- Fine-tuning API (`aeed41e`)
- Files API (`a698f4d`)
- Audio API transcriptions, translations, speech (`1ed0246`)
- Images API (`48e04f4`)
- Moderations API (`7a1e120`)
- Models API (`26ae0da`)
- Embeddings API (`b25f84a`)
- OpenAPI spec for validation tests (`1240604`)
- Advanced features plan streaming, predicted outputs, prompt caching, structured outputs, realtime, builders (`35656ca`)

### Changed
- Upgraded to Rust edition 2024 (`75af083`)
- Updated README with all 18 implemented API endpoints (`0af88a3`)
- Updated CLAUDE.md with complete architecture and API table (`536c2c7`)

---

## [0.1.1]

### Changed
- Renamed crate to `openai-oxide` for crates.io publish (`763955b`)
- Replaced all `openai_rust` references with `openai_oxide` (`f8eff0e`)

---

## [0.1.0]

### Added
- Initial release idiomatic Rust client for OpenAI API (`99fdcdc`)
- Core client with `get`/`post`/`delete` helpers and error parsing (`c3db6f7`)
- Retry logic with exponential backoff (`9b8af51`)
- Chat completions types (`b12876b`)
- `completions().create()` endpoint (`c3a7744`)
- SSE parser and `create_stream()` streaming support (`475203d`)
- Chat and streaming examples (`d51f220`)
- LICENSE, README, and GitHub release metadata (`48c98a9`)