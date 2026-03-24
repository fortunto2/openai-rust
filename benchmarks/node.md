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
