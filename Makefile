.PHONY: test clippy fmt check live doc clean

test:
	cargo test

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt -- --check

check: fmt clippy test

live:
	cargo test --features "live-tests"

doc:
	cargo doc --no-deps --open

sync:
	./scripts/sync-spec.sh

bench:
	cargo run --example benchmark --features responses --release

bench-python:
	cd openai-oxide-python && .venv/bin/python ../examples/bench_python.py

bench-node:
	cd openai-oxide-node && BENCH_ITERATIONS=5 node examples/bench_node.js

bench-all: bench bench-python bench-node

bench-update:
	python3 benchmarks/generate.py
	python3 benchmarks/update-readme.py

clean:
	cargo clean

help:
	@echo "test    — run all tests"
	@echo "clippy  — lint (warnings = errors)"
	@echo "fmt     — check formatting"
	@echo "check   — fmt + clippy + test"
	@echo "live    — tests with real API (needs OPENAI_API_KEY)"
	@echo "doc     — generate and open docs"
	@echo "sync    — check OpenAPI spec drift vs upstream"
	@echo "bench        — Rust benchmark (needs OPENAI_API_KEY)"
	@echo "bench-python — Python benchmark"
	@echo "bench-node   — Node benchmark"
	@echo "bench-all    — all three benchmarks"
	@echo "bench-update — regenerate tables from results.json"
	@echo "clean        — remove build artifacts"
