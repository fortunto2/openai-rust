# openai-oxide for Python

[![PyPI](https://img.shields.io/pypi/v/openai-oxide.svg)](https://pypi.org/project/openai-oxide/)
[![PyPI downloads](https://img.shields.io/pypi/dm/openai-oxide.svg)](https://pypi.org/project/openai-oxide/)

Python bindings for [openai-oxide](https://github.com/fortunto2/openai-oxide) — the fastest OpenAI client. Also available on [crates.io](https://crates.io/crates/openai-oxide) (Rust) and [npm](https://www.npmjs.com/package/openai-oxide) (Node.js).

Rust core compiled to native Python extension via [PyO3](https://pyo3.rs) + [maturin](https://www.maturin.rs).

## Install

```bash
pip install openai-oxide
# or
uv pip install openai-oxide
# or
uv add openai-oxide
```

For local development (requires Rust toolchain):

```bash
cd openai-oxide-python
uv sync
uv run maturin develop --release
```

## Usage

```python
import asyncio, json
from openai_oxide import Client

async def main():
    client = Client()  # reads OPENAI_API_KEY env var

    # Simple
    r = await client.create("gpt-4o-mini", "Hello!")
    print(r["text"])

    # Streaming
    stream = await client.create_stream("gpt-4o-mini", "Explain quantum computing...", max_output_tokens=200)
    async for event in stream:
        print(event)

    # Structured output
    schema = json.dumps({
        "type": "object",
        "properties": {"answer": {"type": "integer"}},
        "required": ["answer"],
        "additionalProperties": False
    })
    r = await client.create_structured("gpt-4o-mini", "What is 7*8?", "math", schema)
    print(json.loads(r["text"])["answer"])  # 56

    # Function calling
    tools = json.dumps([{
        "name": "get_weather",
        "description": "Get weather",
        "parameters": {
            "type": "object",
            "properties": {"city": {"type": "string"}},
            "required": ["city"],
            "additionalProperties": False
        }
    }])
    r = await client.create_with_tools("gpt-4o-mini", "Weather in Tokyo?", tools)
    print(r["function_calls"])

asyncio.run(main())
```

## Test

```bash
uv run python test_oxide.py
```

## Benchmarks

Run `uv run python examples/bench_python.py` to compare against the official `openai` SDK.

See the [main README](https://github.com/fortunto2/openai-oxide#python-ecosystem-openai-oxide-python-vs-openai) for full benchmark results.
