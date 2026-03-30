### Rust Ecosystem (`openai-oxide` vs `async-openai` vs `genai`)

n=5 per run, 3 runs. At this sample size with ~200ms API jitter, differences <15% are noise.

| Test | `openai-oxide` | `async-openai` | `genai` | Note |
| :--- | :--- | :--- | :--- | :--- |
| **Plain text** | 1011ms | **960ms** | **835ms** | oxide slower |
| **Structured output** | 1331ms | N/A | **1197ms** | within noise |
| **Function calling** | **1192ms** | 1748ms | **1030ms** | genai fastest |
| **Multi-turn (2 reqs)** | 2362ms | 3275ms | **1641ms** | genai fastest |
| **Streaming TTFT** | **645ms** | 685ms | 670ms | within noise |
| **Parallel 3x** | 1165ms | **1053ms** | **866ms** | oxide slower |

**Honest note:** On single HTTP requests, oxide is not faster than async-openai or genai. All three SDKs are within API variance at n=5. The value proposition is API completeness and type safety, not raw speed.

#### WebSocket mode (openai-oxide only) — preliminary

⚠️ These numbers need a reproducible benchmark script. Currently measured manually, not via automated suite.

| Test | WebSocket | HTTP | Improvement |
| :--- | :--- | :--- | :--- |
| **Plain text** | 710ms | 1011ms | -29% |
| **Multi-turn (2 reqs)** | 1425ms | 2362ms | -40% |
| **Rapid-fire (5 calls)** | 3227ms | 5807ms | -44% |

*median of medians, 3×5 iterations. Model: gpt-5.4. macOS (M-series), release mode, warm connections.*

Reproduce (HTTP only): `cargo run --example benchmark --features responses --release`
TODO: add `benchmarks/ws_compare.rs` for reproducible WebSocket vs HTTP comparison.
