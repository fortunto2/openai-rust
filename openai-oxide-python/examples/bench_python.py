#!/usr/bin/env python3
"""Benchmark: openai-oxide vs official openai SDK (Python 3.13+)

Run: cd openai-oxide-python && uv run python examples/bench_python.py
"""

import asyncio
import json
import os
import time
from statistics import median

MODEL = os.environ.get("OPENAI_MODEL", "gpt-5.4")
ITERATIONS = int(os.environ.get("BENCH_ITERATIONS", "5"))

WEATHER_TOOLS = [
    {
        "type": "function",
        "name": "get_weather",
        "description": "Get weather",
        "parameters": {
            "type": "object",
            "properties": {
                "city": {"type": "string"},
                "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
            },
            "required": ["city", "unit"],
            "additionalProperties": False,
        },
    }
]

LANG_SCHEMA = {
    "type": "object",
    "properties": {
        "languages": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {"name": {"type": "string"}, "year": {"type": "integer"}},
                "required": ["name", "year"],
                "additionalProperties": False,
            },
        }
    },
    "required": ["languages"],
    "additionalProperties": False,
}

NESTED_SCHEMA = {
    "type": "object",
    "properties": {
        "company": {"type": "string"},
        "departments": {
            "type": "array",
            "items": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "employees": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": {"type": "string"},
                                "role": {"type": "string"},
                                "skills": {"type": "array", "items": {"type": "string"}},
                            },
                            "required": ["name", "role", "skills"],
                            "additionalProperties": False,
                        },
                    },
                },
                "required": ["name", "employees"],
                "additionalProperties": False,
            },
        },
    },
    "required": ["company", "departments"],
    "additionalProperties": False,
}


async def sample(iterations, fn):
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        await fn()
        times.append(round((time.perf_counter() - start) * 1000))
    return median(times)


def winner_label(oxide_ms, official_ms):
    if oxide_ms == "N/A":
        return "official"
    if official_ms == "N/A":
        return "oxide"
    if oxide_ms <= official_ms:
        pct = round((official_ms - oxide_ms) / official_ms * 100)
        return f"OXIDE (+{pct}%)"
    else:
        pct = round((oxide_ms - official_ms) / official_ms * 100)
        return f"python (+{pct}%)"


def fmt(ms):
    return "N/A" if ms == "N/A" else f"{ms}ms"


def print_row(name, oxide_ms, official_ms):
    w = winner_label(oxide_ms, official_ms)
    print(f"{name:<28} {fmt(oxide_ms):>10} {fmt(official_ms):>10} {w:>16}")


async def main():
    from openai import AsyncOpenAI
    from openai_oxide import Client as OxideClient

    official = AsyncOpenAI()
    oxide = OxideClient()

    print(f"Warming up ({MODEL})...")
    await official.responses.create(model=MODEL, input="ping", max_output_tokens=16)
    json.loads(await oxide.create(MODEL, "ping", max_output_tokens=16))
    print("Ready.\n")

    print(f"{'Test':<28} {'oxide':>10} {'official':>10} {'winner':>16}")
    print("-" * 68)

    # Plain text
    async def oxide_plain():
        json.loads(await oxide.create(MODEL, "Capital of France? One word.", max_output_tokens=16))

    async def official_plain():
        await official.responses.create(model=MODEL, input="Capital of France? One word.", max_output_tokens=16)

    print_row("Plain text", await sample(ITERATIONS, oxide_plain), await sample(ITERATIONS, official_plain))

    # Structured output
    async def oxide_structured():
        await oxide.create_structured(
            MODEL, "List 3 programming languages with year", "langs", json.dumps(LANG_SCHEMA), max_output_tokens=200
        )

    async def official_structured():
        await official.responses.create(
            model=MODEL,
            input="List 3 programming languages with year",
            max_output_tokens=200,
            text={"format": {"type": "json_schema", "name": "langs", "strict": True, "schema": LANG_SCHEMA}},
        )

    print_row("Structured output", await sample(ITERATIONS, oxide_structured), await sample(ITERATIONS, official_structured))

    # Function calling
    async def oxide_fc():
        await oxide.create_with_tools(MODEL, "Weather in Tokyo?", json.dumps(WEATHER_TOOLS))

    async def official_fc():
        await official.responses.create(model=MODEL, input="Weather in Tokyo?", tools=WEATHER_TOOLS)

    print_row("Function calling", await sample(ITERATIONS, oxide_fc), await sample(ITERATIONS, official_fc))

    # Multi-turn
    async def oxide_multi():
        r = json.loads(await oxide.create(MODEL, "Remember: answer is 42.", max_output_tokens=32))
        # Use raw for follow-up with previous_response_id
        await oxide.create_raw(
            json.dumps({"model": MODEL, "input": "What is the answer?", "previous_response_id": r["id"], "max_output_tokens": 16})
        )

    async def official_multi():
        first = await official.responses.create(model=MODEL, input="Remember: answer is 42.", max_output_tokens=32, store=True)
        await official.responses.create(model=MODEL, input="What is the answer?", previous_response_id=first.id, max_output_tokens=16)

    print_row("Multi-turn (2 reqs)", await sample(ITERATIONS, oxide_multi), await sample(ITERATIONS, official_multi))

    # Web search
    async def oxide_web():
        await oxide.create_with_tools(MODEL, "Latest Rust version?", json.dumps([{"type": "web_search_preview"}]))

    async def official_web():
        await official.responses.create(model=MODEL, input="Latest Rust version?", tools=[{"type": "web_search_preview"}])

    print_row("Web search", await sample(ITERATIONS, oxide_web), await sample(ITERATIONS, official_web))

    # Nested structured
    async def oxide_nested():
        await oxide.create_structured(
            MODEL,
            "Describe a tech company with 2 departments, 3 employees each, with skills",
            "company",
            json.dumps(NESTED_SCHEMA),
            max_output_tokens=1000,
        )

    async def official_nested():
        await official.responses.create(
            model=MODEL,
            input="Describe a tech company with 2 departments, 3 employees each, with skills",
            max_output_tokens=1000,
            text={"format": {"type": "json_schema", "name": "company", "strict": True, "schema": NESTED_SCHEMA}},
        )

    print_row("Nested structured", await sample(ITERATIONS, oxide_nested), await sample(ITERATIONS, official_nested))

    # Agent loop (2-step)
    async def oxide_agent():
        r = json.loads(await oxide.create_with_tools(MODEL, "Weather in Paris?", json.dumps(WEATHER_TOOLS)))
        fc = r.get("function_calls", [{}])
        tool_output = json.dumps({"temperature": 18, "unit": "celsius"})
        items = [{"type": "function_call_output", "call_id": fc[0].get("call_id", ""), "output": tool_output}]
        await oxide.create_raw(
            json.dumps({"model": MODEL, "input": items, "previous_response_id": r["id"], "max_output_tokens": 100})
        )

    async def official_agent():
        first = await official.responses.create(model=MODEL, input="Weather in Paris?", tools=WEATHER_TOOLS)
        tool_output = json.dumps({"temperature": 18, "unit": "celsius"})
        call_id = first.output[0].call_id if first.output else ""
        await official.responses.create(
            model=MODEL,
            input=[{"type": "function_call_output", "call_id": call_id, "output": tool_output}],
            previous_response_id=first.id,
            max_output_tokens=100,
        )

    print_row("Agent loop (2-step)", await sample(ITERATIONS, oxide_agent), await sample(ITERATIONS, official_agent))

    # Rapid-fire
    async def oxide_rapid():
        for i in range(1, 6):
            json.loads(await oxide.create(MODEL, f"What is {i}+{i}? Number only.", max_output_tokens=16))

    async def official_rapid():
        for i in range(1, 6):
            await official.responses.create(model=MODEL, input=f"What is {i}+{i}? Number only.", max_output_tokens=16)

    print_row("Rapid-fire (5 calls)", await sample(ITERATIONS, oxide_rapid), await sample(ITERATIONS, official_rapid))

    # Streaming TTFT
    async def oxide_ttft():
        stream = await oxide.create_stream(MODEL, "Explain quicksort in 3 sentences.", max_output_tokens=200)
        async for event_json in stream:
            event = json.loads(event_json)
            if event.get("type") == "OutputTextDelta" or "delta" in event:
                return

    async def official_ttft():
        stream = await official.responses.create(
            model=MODEL, input="Explain quicksort in 3 sentences.", max_output_tokens=200, stream=True
        )
        async for event in stream:
            if event.type == "response.output_text.delta":
                return

    print_row("Streaming TTFT", await sample(ITERATIONS, oxide_ttft), await sample(ITERATIONS, official_ttft))

    # Parallel 3x
    async def oxide_parallel():
        await asyncio.gather(
            asyncio.create_task(asyncio.coroutine(lambda: oxide.create(MODEL, "Capital of France?", max_output_tokens=16))()),
            asyncio.create_task(asyncio.coroutine(lambda: oxide.create(MODEL, "Capital of Japan?", max_output_tokens=16))()),
            asyncio.create_task(asyncio.coroutine(lambda: oxide.create(MODEL, "Capital of Brazil?", max_output_tokens=16))()),
        )

    async def _oxide_one(q):
        return await oxide.create(MODEL, q, max_output_tokens=16)

    async def _official_one(q):
        return await official.responses.create(model=MODEL, input=q, max_output_tokens=16)

    async def oxide_par():
        await asyncio.gather(_oxide_one("Capital of France?"), _oxide_one("Capital of Japan?"), _oxide_one("Capital of Brazil?"))

    async def official_par():
        await asyncio.gather(
            _official_one("Capital of France?"), _official_one("Capital of Japan?"), _official_one("Capital of Brazil?")
        )

    print_row("Parallel 3x (fan-out)", await sample(ITERATIONS, oxide_par), await sample(ITERATIONS, official_par))

    print(f"\n{ITERATIONS} iterations, median. Model: {MODEL}")
    print("oxide:    openai-oxide (native PyO3)")
    print("official: openai SDK (httpx)")


asyncio.run(main())
