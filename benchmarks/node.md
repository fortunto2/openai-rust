### Node.js Ecosystem (`openai-oxide` vs `openai`)

Native napi-rs bindings vs official `openai` npm. n=5 per run, 3 runs — differences <15% are within API noise.

| Test | `openai-oxide` | `openai` | Diff | Note |
| :--- | :--- | :--- | :--- | :--- |
| **Plain text** | 1075ms | 1311ms | -18% | |
| **Structured output** | 1370ms | 1765ms | -22% | |
| **Function calling** | 1725ms | 1832ms | -6% | within API noise |
| **Multi-turn (2 reqs)** | 2283ms | 2859ms | -20% | |
| **Rapid-fire (5 calls)** | 6246ms | 6936ms | -10% | within API noise |
| **Streaming TTFT** | 534ms | 580ms | -8% | within API noise |
| **Parallel 3x** | 1937ms | 1991ms | -3% | within API noise |
| **WebSocket hot pair** | 2181ms | N/A | — | preliminary, needs reproducible script |

*median of medians, 3×5 iterations. Model: gpt-5.4. At n=5 with ~200ms API jitter, only >15% differences are meaningful.*

Reproduce: `cd openai-oxide-node && BENCH_ITERATIONS=5 node examples/bench_node.js`
