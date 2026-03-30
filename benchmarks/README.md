# Benchmarks

## Structure

```
benchmarks/
  bench_science.js  — scientific benchmark (mock + live)
  test2.json        — real coding agent request fixture (718KB, 320 msgs, 42 tools)
  out2.json         — real coding agent response fixture (114 SSE chunks)
  results.json      — live benchmark numbers for table generation
  generate.py       — generates markdown tables from results.json
  update-readme.py  — injects tables into READMEs
```

## Running

```bash
# Mock only (default) — localhost mock server, zero network
node --expose-gc benchmarks/bench_science.js

# Mock + real API
MODE=live OPENAI_API_KEY=sk-... node --expose-gc benchmarks/bench_science.js

# Custom model / iterations
MODE=live OPENAI_MODEL=gpt-5.4 BENCH_ITERATIONS=20 node --expose-gc benchmarks/bench_science.js
```

## What we measure and what we don't

This benchmark measures **SDK framework overhead** — the time spent inside the SDK
on request building, JSON serialization, response parsing, SSE chunk processing, and
object construction. It does NOT measure network latency or model inference time.

We use two setups:
- **Mock:** localhost HTTP server returns canned responses instantly. Isolates pure SDK cost.
- **Live:** real OpenAI API. Shows whether SDK overhead is visible in end-to-end latency.

## Results

### Mock — pure SDK overhead (no network)

Node v22.22.0 · darwin arm64 · `--expose-gc`
50 iterations · 20 warmup · Welch's t-test

| Test | oxide | official | oxide faster | sig |
|------|-------|----------|-------------|-----|
| Tiny req → Tiny resp | 172µs | 443µs | +61% | *** |
| Tiny req → Structured 5KB | 161µs | 499µs | +68% | *** |
| Medium 150KB → Tool call | 1.1ms | 1.7ms | +37% | *** |
| Heavy 657KB → Real resp | 4.9ms | 6.2ms | +21% | *** |
| Agent 5x tiny | 604µs | 1.7ms | +63% | *** |
| Agent 10x tiny | 1.1ms | 2.7ms | +58% | *** |
| Agent 20x tiny | 2.1ms | 5.4ms | +61% | *** |
| Agent 5x heavy (657KB) | 24.8ms | 30.1ms | +18% | *** |
| Agent 10x heavy (657KB) | 51.7ms | 62.2ms | +17% | *** |
| SSE stream (114 chunks) | 283µs | 742µs | +62% | *** |
| Tiny + gzip | 154µs | 466µs | +67% | *** |
| Heavy + gzip | 5.0ms | 6.3ms | +21% | * |

Fast path (`createResponseFast` — pre-serialized JSON):

| Request size | napi (default) | fast path | official | fast vs official |
|-------------|---------------|-----------|----------|-----------------|
| Tiny | 146µs | 113µs | 347µs | +67% |
| Medium 150KB | 923µs | 661µs | 1.7ms | +60% |
| Heavy 657KB | 4.9ms | 2.8ms | 6.0ms | +53% |

### Live — real API (gpt-5.4, 10 iterations)

| Test | oxide | official | oxide faster | sig |
|------|-------|----------|-------------|-----|
| Plain text | 898ms | 974ms | +8% | ns |
| Structured output | 1.46s | 1.34s | -9% | ns |
| Function calling | 1.14s | 1.26s | +9% | ns |
| SSE stream | 1.93s | 1.94s | +1% | ns |
| 3-step agent | 3.01s | 3.26s | +8% | ns |

## Honest assessment

**Where oxide is clearly faster:**
- Pure SDK overhead on the client side: 2-3x on small payloads, 1.2-1.3x on heavy ones.
  This is statistically significant (p<0.001) and reproducible.
- SSE streaming: per-chunk processing is ~2.5x faster due to zero-copy parsing in Rust
  vs Uint8Array allocation + TextDecoder + JSON.parse per chunk in JS.
- Agent loops: overhead compounds linearly. 20 sequential calls save ~3ms of pure SDK time.

**Where it doesn't matter:**
- Single API calls to real OpenAI endpoints. Network latency (200ms-2s) is 100-1000x
  larger than SDK overhead (0.1-5ms). Live results are statistically indistinguishable
  (`ns`). The +8% trend on the 3-step agent is consistent but not significant at n=10.
- Heavy requests (600KB+). `JSON.stringify` takes ~2ms and is the same for both SDKs
  (V8 C++). This shared cost dilutes the framework advantage from +60% down to +20%.

**What this means in practice:**
- If your bottleneck is API latency (most use cases) — SDK choice doesn't matter for speed.
- If you're building high-throughput pipelines, local proxies, or processing many
  requests with fast backends (cached responses, local models) — oxide saves real time.
- `createResponseFast(JSON.stringify(req))` is useful when you cache or repeat requests —
  it skips the napi object copy and is +53-67% faster even on 657KB payloads.

## Why the difference exists

The official openai npm SDK is well-written and does no unnecessary work. The difference
comes from the runtime, not code quality:

1. **Object allocation:** Every JS object lives on the heap with GC tracking.
   Rust structs are stack-allocated, no GC.
2. **SSE parsing:** Official SDK allocates a new `Uint8Array` per chunk, decodes UTF-8,
   splits lines, then `JSON.parse` each one. Oxide parses in-place from a reusable buffer.
3. **Promise/async overhead:** Each SDK call in Node goes through APIPromise wrapping,
   header merging, and multiple `.then()` chains. Oxide does one napi boundary crossing.
4. **napi tradeoff:** Oxide pays a cost to copy JS objects into Rust (visible on 657KB
   requests). The `createResponseFast` path avoids this by accepting a JSON string directly.

## Methodology

### Mock
- HTTP/1.1 server on 127.0.0.1 (not HTTPS, no HTTP/2 — measures SDK overhead only, not multiplexing)
- Fixtures captured from a real coding agent session
- SSE fixture (`out2.json`) is Chat Completions format served at `/v1/responses` endpoint — may slightly penalize official SDK's Responses adapter
- Both SDKs in same Node.js process, same event loop
- `--expose-gc` with `global.gc()` between suites
- 50 iterations, 20 warmup, median reported
- Welch's t-test for statistical significance (unequal variance)

### Live
- Real OpenAI API, configurable model
- HTTP/2 with connection pooling (reqwest ALPN negotiation)
- 10 iterations, 3 warmup, warm connections
- macOS Apple Silicon (M-series)
- At n=10, differences <15% are within API jitter (not significant)

## Reproducing

```bash
git clone https://github.com/fortunto2/openai-oxide
cd openai-oxide/openai-oxide-node && pnpm install && pnpm run build
cd .. && node --expose-gc benchmarks/bench_science.js
```

## Where results appear

| Location | How |
|----------|-----|
| `docs/src/guides/benchmarks.md` | `{{#include}}` from `all.md` |
| `README.md` | `<!-- BENCH:node:START -->` markers |
| `openai-oxide-node/README.md` | `<!-- BENCH:node:START -->` marker |

Update live tables: `python3 benchmarks/generate.py && python3 benchmarks/update-readme.py`
