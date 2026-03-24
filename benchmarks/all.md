## Benchmarks

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

---

### Python Ecosystem (`openai-oxide-python` vs `openai`)

`openai-oxide` wins **10/12** tests. Native PyO3 bindings vs `openai` (openai 2.29.0).

| Test | `openai-oxide` | `openai` | Winner |
| :--- | :--- | :--- | :--- |
| **Plain text** | **845ms** | 997ms | OXIDE (+15%) |
| **Structured output** | **1367ms** | 1379ms | OXIDE (+1%) |
| **Function calling** | **1195ms** | 1230ms | OXIDE (+3%) |
| **Multi-turn (2 reqs)** | **2260ms** | 3089ms | OXIDE (+27%) |
| **Web search** | **3157ms** | 3499ms | OXIDE (+10%) |
| **Nested structured** | 5377ms | **5339ms** | python (+1%) |
| **Agent loop (2-step)** | **4570ms** | 5144ms | OXIDE (+11%) |
| **Rapid-fire (5 calls)** | **5667ms** | 6136ms | OXIDE (+8%) |
| **Prompt-cached** | **4425ms** | 5564ms | OXIDE (+20%) |
| **Streaming TTFT** | **626ms** | 638ms | OXIDE (+2%) |
| **Parallel 3x** | 1184ms | **1090ms** | python (+9%) |
| **Hedged (2x race)** | **893ms** | 995ms | OXIDE (+10%) |

*median of medians, 3×5 iterations. Model: gpt-5.4.*

Reproduce: `cd openai-oxide-python && uv run python ../examples/bench_python.py`

---

### Node.js Ecosystem (`openai-oxide` vs `openai`)

`openai-oxide` wins **8/8** tests. Native napi-rs bindings vs official `openai` npm.

| Test | `openai-oxide` | `openai` | Winner |
| :--- | :--- | :--- | :--- |
| **Plain text** | **1075ms** | 1311ms | OXIDE (+18%) |
| **Structured output** | **1370ms** | 1765ms | OXIDE (+22%) |
| **Function calling** | **1725ms** | 1832ms | OXIDE (+6%) |
| **Multi-turn (2 reqs)** | **2283ms** | 2859ms | OXIDE (+20%) |
| **Rapid-fire (5 calls)** | **6246ms** | 6936ms | OXIDE (+10%) |
| **Streaming TTFT** | **534ms** | 580ms | OXIDE (+8%) |
| **Parallel 3x** | **1937ms** | 1991ms | OXIDE (+3%) |
| **WebSocket hot pair** | **2181ms** | N/A | OXIDE |

*median of medians, 3×5 iterations. Model: gpt-5.4.*

Reproduce: `cd openai-oxide-node && BENCH_ITERATIONS=5 node examples/bench_node.js`
