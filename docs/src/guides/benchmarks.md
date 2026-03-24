# Benchmarks

All benchmarks: median of 3 runs × 5 iterations each. Model: `gpt-5.4`. Environment: macOS (M-series), release mode, warm HTTP/2 connections.

{{#include ../../../benchmarks/all.md}}

## How to run

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

## Methodology

- **Warm connections**: First request is a warmup (not measured). All subsequent requests reuse HTTP/2 connections with keep-alive.
- **Median of medians**: Each test runs 5 iterations per run, 3 runs total. We report the median of the 3 median values.
- **Same prompts**: Both clients send identical requests to the same model.
- **Release mode**: Rust benchmarks compiled with `--release`. Python and Node use prebuilt native extensions.

## Updating benchmarks

1. Edit `benchmarks/results.json` with new numbers
2. Run `python3 benchmarks/generate.py` to regenerate tables
3. Docs and README include from generated files
