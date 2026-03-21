#!/usr/bin/env python3
"""Side-by-side benchmark: openai-oxide-python (Rust/PyO3) vs openai (Python/httpx)."""

import asyncio
import concurrent.futures
import json
import time
import sys

try:
    from openai import OpenAI as PyClient
except ImportError:
    print("Please install openai: pip install openai")
    sys.exit(1)

try:
    from openai_oxide_python import Client as OxideClient
except ImportError:
    print("Please install openai_oxide_python. Run `uv run maturin develop --release` in `openai-oxide-python`.")
    sys.exit(1)

MODEL = "gpt-5.4"
N = 5  # iterations

py = PyClient()
oxide = OxideClient()

def stats(times):
    times.sort()
    return times[len(times) // 2] if times else 0

results = []

def row(name, oxide_ms, py_ms):
    if oxide_ms == "N/A":
        print(f"{name:<25} {'N/A':>8} {py_ms:>8}ms  {'python':>8} (---%)")
        return
    winner = "OXIDE" if oxide_ms <= py_ms else "python"
    pct = abs(int((py_ms - oxide_ms) / max(py_ms, 1) * 100))
    results.append((name, oxide_ms, py_ms, winner))
    print(f"{name:<25} {oxide_ms:>6}ms {py_ms:>6}ms  {winner:>8} ({pct}%)")

async def main():
    print("Warming up...")
    py.responses.create(model=MODEL, input="ping", max_output_tokens=16)
    await oxide.create(MODEL, "ping", max_output_tokens=16)
    print("Ready.\n")

    print(f"{'Test':<25} {'oxide-py':>8} {'python':>8}  {'winner':>8}")
    print("-" * 62)

    # 1. Plain text
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        await oxide.create(MODEL, "What is the capital of France? One word.", max_output_tokens=16)
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        py.responses.create(model=MODEL, input="What is the capital of France? One word.", max_output_tokens=16)
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Plain text", stats(ot), stats(pt))

    # 2. Structured output
    schema_obj = {
        "type": "object",
        "properties": {"languages": {"type": "array", "items": {"type": "object",
            "properties": {"name": {"type": "string"}, "year": {"type": "integer"}},
            "required": ["name", "year"], "additionalProperties": False}}},
        "required": ["languages"], "additionalProperties": False}
    schema_str = json.dumps(schema_obj)
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        await oxide.create_structured(MODEL, "List 3 programming languages with year created", "langs", schema_str, max_output_tokens=200)
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        py.responses.create(model=MODEL, input="List 3 programming languages with year created", max_output_tokens=200,
            text={"format":{"type":"json_schema","name":"langs","strict":True,"schema":schema_obj}})
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Structured output", stats(ot), stats(pt))

    # 3. Function calling
    tools_obj = [{"type":"function","name":"get_weather","description":"Get weather","parameters":{"type":"object","properties":{"city":{"type":"string"}, "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}},"required":["city", "unit"],"additionalProperties":False}}]
    tools_str = json.dumps(tools_obj)
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        await oxide.create_with_tools(MODEL, "What's the weather in Tokyo?", tools_str)
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        py.responses.create(model=MODEL, input="What's the weather in Tokyo?", tools=tools_obj)
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Function calling", stats(ot), stats(pt))

    # 4. Multi-turn
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        r1_raw = await oxide.create_raw(json.dumps({"model": MODEL, "input": "Remember: the answer is 42.", "store": True, "max_output_tokens": 32}))
        r1_id = json.loads(r1_raw).get("id", "")
        await oxide.create_raw(json.dumps({"model": MODEL, "input": "What is the answer?", "previous_response_id": r1_id, "max_output_tokens": 16}))
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        r1 = py.responses.create(model=MODEL, input="Remember: the answer is 42.", store=True, max_output_tokens=32)
        py.responses.create(model=MODEL, input="What is the answer?", previous_response_id=r1.id, max_output_tokens=16)
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Multi-turn (2 reqs)", stats(ot), stats(pt))

    # 5. Web search
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        await oxide.create_raw(json.dumps({"model": MODEL, "input": "What is the latest Rust version?", "tools": [{"type": "web_search", "search_context_size": "low"}], "max_output_tokens": 100}))
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        py.responses.create(model=MODEL, input="What is the latest Rust version?", max_output_tokens=100, tools=[{"type": "web_search", "search_context_size": "low"}])
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Web search", stats(ot), stats(pt))

    # 6. Nested structured output
    complex_schema = {
        "type": "object",
        "properties": {
            "company": {"type": "object", "properties": {
                "name": {"type": "string"}, "founded": {"type": "integer"}, "ceo": {"type": "string"},
                "products": {"type": "array", "items": {"type": "object", "properties": {
                    "name": {"type": "string"},
                    "category": {"type": "string", "enum": ["hardware", "software", "service"]},
                    "revenue_billions": {"type": "number"}, "active": {"type": "boolean"}},
                    "required": ["name", "category", "revenue_billions", "active"], "additionalProperties": False}}},
                "required": ["name", "founded", "ceo", "products"], "additionalProperties": False},
            "competitors": {"type": "array", "items": {"type": "string"}},
            "summary": {"type": "string"}},
        "required": ["company", "competitors", "summary"], "additionalProperties": False}
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        await oxide.create_raw(json.dumps({"model": MODEL, "input": "Analyze Apple Inc: products with revenue, competitors, summary.", "max_output_tokens": 800, "text": {"format": {"type": "json_schema", "name": "company_analysis", "strict": True, "schema": complex_schema}}}))
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        py.responses.create(model=MODEL, input="Analyze Apple Inc: products with revenue, competitors, summary.", max_output_tokens=800, text={"format": {"type": "json_schema", "name": "company_analysis", "strict": True, "schema": complex_schema}})
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Nested structured", stats(ot), stats(pt))

    # 7. Agent loop (2-step)
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        r1_raw = await oxide.create_raw(json.dumps({"model": MODEL, "input": "What's the weather in Tokyo and what should I wear?", "store": True, "tools": tools_obj}))
        r1_d = json.loads(r1_raw)
        call_id = "call_1"
        for item in r1_d.get("output", []):
            if item.get("type") == "function_call":
                call_id = item.get("call_id")
                break
        await oxide.create_raw(json.dumps({"model": MODEL, "previous_response_id": r1_d.get("id"), "max_output_tokens": 200, "input": [{"type": "function_call_output", "call_id": call_id, "output": '{"temp":22,"condition":"sunny","humidity":45}'}], "text": {"format": {"type": "json_schema", "name": "recommendation", "strict": True, "schema": {"type": "object", "properties": {"outfit": {"type": "string"}, "accessories": {"type": "array", "items": {"type": "string"}}, "warning": {"type": "string"}}, "required": ["outfit", "accessories", "warning"], "additionalProperties": False}}}}))
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        step1 = py.responses.create(model=MODEL, input="What's the weather in Tokyo and what should I wear?", store=True, tools=tools_obj)
        call_id = next((item.call_id for item in step1.output if item.type == "function_call"), "call_1")
        py.responses.create(model=MODEL, previous_response_id=step1.id, max_output_tokens=200, input=[{"type": "function_call_output", "call_id": call_id, "output": '{"temp":22,"condition":"sunny","humidity":45}'}], text={"format": {"type": "json_schema", "name": "recommendation", "strict": True, "schema": {"type": "object", "properties": {"outfit": {"type": "string"}, "accessories": {"type": "array", "items": {"type": "string"}}, "warning": {"type": "string"}}, "required": ["outfit", "accessories", "warning"], "additionalProperties": False}}})
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Agent loop (2-step)", stats(ot), stats(pt))

    # 8. Rapid-fire (5 calls)
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        for i in range(1, 6):
            await oxide.create(MODEL, f"What is {i}+{i}? Reply with just the number.", max_output_tokens=16)
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        for i in range(1, 6):
            py.responses.create(model=MODEL, input=f"What is {i}+{i}? Reply with just the number.", max_output_tokens=16)
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Rapid-fire (5 calls)", stats(ot), stats(pt))

    # 9. Prompt-cached
    ot, pt = [], []
    system_prompt = "You are a senior software architect with 20 years of experience in distributed systems, microservices, and cloud-native architectures. Always provide specific, actionable advice with code examples where relevant. Consider scalability, maintainability, and security in every recommendation."
    await oxide.create_raw(json.dumps({"model": MODEL, "instructions": system_prompt, "input": "ping", "prompt_cache_key": "bench-architect-ox", "max_output_tokens": 16}))
    py.responses.create(model=MODEL, instructions=system_prompt, input="ping", prompt_cache_key="bench-architect-py", max_output_tokens=16)
    
    for _ in range(N):
        t0 = time.perf_counter()
        await oxide.create_raw(json.dumps({"model": MODEL, "instructions": system_prompt, "input": "How should I design a rate limiter for an API gateway?", "prompt_cache_key": "bench-architect-ox", "prompt_cache_retention": "24h", "max_output_tokens": 200}))
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        py.responses.create(model=MODEL, instructions=system_prompt, input="How should I design a rate limiter for an API gateway?", prompt_cache_key="bench-architect-py", prompt_cache_retention="24h", max_output_tokens=200)
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Prompt-cached", stats(ot), stats(pt))

    # 10. Streaming TTFT
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        stream = py.responses.create(model=MODEL, input="Explain quicksort in 3 sentences.", max_output_tokens=200, stream=True)
        for event in stream:
            if event.type == "response.output_text.delta":
                pt.append(int((time.perf_counter() - t0) * 1000))
                break
    row("Streaming TTFT", "N/A", stats(pt))

    # 11. Parallel 3x
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        await asyncio.gather(
            oxide.create(MODEL, "Capital of France? One word.", max_output_tokens=16),
            oxide.create(MODEL, "Capital of Japan? One word.", max_output_tokens=16),
            oxide.create(MODEL, "Capital of Brazil? One word.", max_output_tokens=16),
        )
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as ex:
            fs = [ex.submit(lambda q: py.responses.create(model=MODEL, input=f"Capital of {q}? One word.", max_output_tokens=16), q) for q in ["France", "Japan", "Brazil"]]
            for f in concurrent.futures.as_completed(fs):
                f.result()
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Parallel 3x", stats(ot), stats(pt))

    # 12. Hedged (2x race)
    ot, pt = [], []
    for _ in range(N):
        t0 = time.perf_counter()
        t1 = oxide.create(MODEL, "What is 7*8? Number only.", max_output_tokens=16)
        t2 = oxide.create(MODEL, "What is 7*8? Number only.", max_output_tokens=16)
        done, pending = await asyncio.wait([t1, t2], return_when=asyncio.FIRST_COMPLETED)
        for p in pending:
            p.cancel()
        ot.append(int((time.perf_counter() - t0) * 1000))
    for _ in range(N):
        t0 = time.perf_counter()
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
            f1 = ex.submit(lambda: py.responses.create(model=MODEL, input="What is 7*8? Number only.", max_output_tokens=16))
            f2 = ex.submit(lambda: py.responses.create(model=MODEL, input="What is 7*8? Number only.", max_output_tokens=16))
            done, _ = concurrent.futures.wait([f1, f2], return_when=concurrent.futures.FIRST_COMPLETED)
            next(iter(done)).result()
        pt.append(int((time.perf_counter() - t0) * 1000))
    row("Hedged (2x race)", stats(ot), stats(pt))

    print(f"\n{N} iterations, median. Model: {MODEL}")
    print(f"oxide-py: openai-oxide-python v0.1.0 (Rust via PyO3)")
    print(f"python:   openai v{__import__('openai').__version__} (httpx)")

    wins = sum(1 for _, _, _, w in results if w == "OXIDE")
    print(f"\noxide-py wins {wins}/{len(results)} tests")

if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(main())
