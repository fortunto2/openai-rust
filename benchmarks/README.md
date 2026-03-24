# Benchmarks

Single source of truth for all benchmark data across Rust, Python, and Node.js.

## Structure

```
benchmarks/
  results.json      — all numbers (edit this)
  generate.py       — generates markdown tables from JSON
  update-readme.py  — injects tables into READMEs
  rust.md            — generated
  python.md          — generated
  node.md            — generated
  all.md             — generated (used by mdbook docs)
```

## Running benchmarks

### Rust
```bash
cargo run --example benchmark --features responses --release
```

### Python
```bash
cd openai-oxide-python && uv run python ../examples/bench_python.py
```

### Node.js
```bash
cd openai-oxide-node && BENCH_ITERATIONS=5 node examples/bench_node.js
```

## Updating results

1. Run benchmarks 3 times, take median of each run's median
2. Edit `results.json` with new numbers
3. Regenerate and inject:

```bash
python3 benchmarks/generate.py
python3 benchmarks/update-readme.py
```

4. Commit:

```bash
git add benchmarks/ README.md openai-oxide-node/README.md
git commit -m "bench: update results"
```

## Where results appear

| Location | How |
|----------|-----|
| `docs/src/guides/benchmarks.md` | `{{#include}}` from `all.md` |
| `README.md` | `<!-- BENCH:python:START -->` / `<!-- BENCH:node:START -->` markers |
| `openai-oxide-node/README.md` | `<!-- BENCH:node:START -->` marker |

## Methodology

- Model: `gpt-5.4`
- 3 runs × 5 iterations, median of medians
- Warm HTTP/2 connections (first request is warmup, not measured)
- macOS (M-series), release mode
- Same prompts for both clients
