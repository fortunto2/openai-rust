# openai-oxide-python

Python bindings for [openai-oxide](https://github.com/fortunto2/openai-oxide) — the fastest OpenAI client.

Rust core compiled to native Python extension via [PyO3](https://pyo3.rs) + [maturin](https://www.maturin.rs).

## Install (dev)

```bash
cd openai-oxide-python
uv sync
uv run maturin develop --release
```

## Usage

```python
import asyncio, json
from openai_oxide_python import Client

async def main():
    client = Client()  # reads OPENAI_API_KEY env var

    # Simple
    r = await client.create("gpt-5.4", "Hello!")
    print(json.loads(r)["text"])

    # Structured output
    schema = json.dumps({"type": "object", "properties": {"answer": {"type": "integer"}}, "required": ["answer"], "additionalProperties": False})
    r = await client.create_structured("gpt-5.4", "What is 7*8?", "math", schema)
    print(json.loads(json.loads(r)["text"])["answer"])  # 56

    # Function calling
    tools = json.dumps([{"name": "get_weather", "description": "Get weather", "parameters": {"type": "object", "properties": {"city": {"type": "string"}}, "required": ["city"], "additionalProperties": False}}])
    r = await client.create_with_tools("gpt-5.4", "Weather in Tokyo?", tools)
    print(json.loads(r)["function_calls"])

asyncio.run(main())
```

## Test

```bash
uv run python test_oxide.py
```
