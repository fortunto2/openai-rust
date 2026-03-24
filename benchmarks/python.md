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
