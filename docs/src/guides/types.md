# Types (`openai-types` crate)

All OpenAI API types live in the standalone [`openai-types`](https://crates.io/crates/openai-types) crate. It has zero runtime dependencies beyond `serde` and can be used independently of the HTTP client.

## Architecture

```
openai-types/src/{domain}/
  _gen.rs     ← auto-generated from Python SDK (py2rust)
  manual.rs   ← hand-crafted: enums, builders, Option fields
  mod.rs      ← re-exports both
```

- **1100+ types** across 24 domains
- **Auto-synced** from the official Python SDK via `py2rust.py`
- **Manual overrides** preserved on sync (enums with `Other(String)`, builder patterns, precise `Option` fields)
- **Feature-gated** — enable only the domains you need

## Usage

The types are automatically available through `openai-oxide`:

```rust
use openai_oxide::types::chat::*;
use openai_oxide::types::responses::*;
use openai_oxide::types::audio::*;
```

Or use `openai-types` directly (no HTTP client dependency):

```rust
// Cargo.toml: openai-types = { version = "0.1", features = ["chat", "responses"] }
use openai_types::chat::ChatCompletionRequest;
use openai_types::responses::ResponseCreateRequest;
use openai_types::shared::{Role, Usage, FinishReason};
```

## Domains

| Domain | Types | Features |
|--------|-------|----------|
| chat | ~50 | Request/response, messages, tools, streaming |
| responses | ~340 | Responses API, computer actions, MCP, code interpreter |
| audio | ~36 | Speech, transcription, translation, streaming events |
| beta | ~80 | Assistants, threads, runs, vector stores |
| realtime | ~188 | WebSocket events, session management |
| image | ~52 | Generate, edit, variations, streaming |
| embedding | ~6 | Create embeddings |
| file | ~14 | CRUD, chunking strategies |
| fine_tuning | ~25 | Jobs, methods, integrations |
| batch | ~12 | Batch processing |
| moderation | ~8 | Content moderation |
| model | ~2 | Model listing |
| uploads | ~7 | Multipart uploads |
| shared | ~28 | Role, Usage, FinishReason, ListResponse\<T\> |
| + 10 more | ~300 | completion, containers, conversations, evals, graders, skills, vector_stores, video, webhooks, websocket |

## Enums with `Other(String)`

All enums include an `Other(String)` catch-all variant for forward compatibility. If OpenAI adds a new role or finish reason, your code won't break:

```rust
use openai_types::shared::Role;

let role: Role = serde_json::from_str("\"developer\"").unwrap();
assert_eq!(role, Role::Developer);

// Unknown roles deserialize without error
let future_role: Role = serde_json::from_str("\"supervisor\"").unwrap();
assert!(matches!(future_role, Role::Other(_)));
```

## Updating Types

When OpenAI updates their Python SDK:

```bash
# Re-generate all _gen.rs files (manual files untouched)
python3 scripts/py2rust.py sync ~/openai-python/src/openai/types/ openai-types/src/
cargo test
```
