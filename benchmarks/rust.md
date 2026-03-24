### Rust Ecosystem (`openai-oxide` vs `async-openai` vs `genai`)

| Test | `openai-oxide` | `async-openai` | `genai` |
| :--- | :--- | :--- | :--- |
| **Plain text** | 1011ms | 960ms | 835ms |
| **Structured output** | 1331ms | N/A | 1197ms |
| **Function calling** | 1192ms | 1748ms | 1030ms |
| **Multi-turn (2 reqs)** | 2362ms | 3275ms | 1641ms |
| **Streaming TTFT** | **645ms** | 685ms | 670ms |
| **Parallel 3x** | 1165ms | 1053ms | 866ms |

#### WebSocket mode (openai-oxide only)

| Test | WebSocket | HTTP | Improvement |
| :--- | :--- | :--- | :--- |
| **Plain text** | **710ms** | 1011ms | -29% |
| **Multi-turn (2 reqs)** | **1425ms** | 2362ms | -40% |
| **Rapid-fire (5 calls)** | **3227ms** | 5807ms | -44% |

*median of medians, 3×5 iterations. Model: gpt-5.4. macOS (M-series), release mode, warm HTTP/2 connections.*

Reproduce: `cargo run --example benchmark --features responses --release`
