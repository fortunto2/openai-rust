#!/usr/bin/env python3
"""Statistical benchmark: Python openai SDK — same tests as Rust benchmark."""

import json
import time
from openai import OpenAI

MODEL = "gpt-5.4"
ITERATIONS = 5

client = OpenAI()

def stats(times):
    times.sort()
    median = times[len(times) // 2]
    p95 = times[int(len(times) * 0.95)]
    return median, p95, times[0], times[-1]

def bench(fn):
    times = []
    for _ in range(ITERATIONS):
        t0 = time.perf_counter()
        fn()
        times.append(int((time.perf_counter() - t0) * 1000))
    return stats(times)

# Warmup
print("Warming up...")
client.responses.create(model=MODEL, input="ping", max_output_tokens=16)
print("Ready.\n")

print(f"=== Python openai SDK v{__import__('openai').__version__} ({ITERATIONS} iterations, model: {MODEL}) ===\n")
print(f"{'Test':<25} {'Median':>8} {'P95':>8} {'Min':>8} {'Max':>8}")
print("-" * 65)

# Test 1: Plain text
med, p95, mn, mx = bench(lambda: client.responses.create(
    model=MODEL, input="What is the capital of France? One word.", max_output_tokens=16))
print(f"{'Plain text':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 2: Structured output
med, p95, mn, mx = bench(lambda: client.responses.create(
    model=MODEL,
    input="List 3 programming languages with year created",
    max_output_tokens=200,
    text={"format": {"type": "json_schema", "name": "languages", "strict": True, "schema": {
        "type": "object",
        "properties": {"languages": {"type": "array", "items": {"type": "object",
            "properties": {"name": {"type": "string"}, "year": {"type": "integer"}},
            "required": ["name", "year"], "additionalProperties": False}}},
        "required": ["languages"], "additionalProperties": False}}}))
print(f"{'Structured output':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 3: Function calling
med, p95, mn, mx = bench(lambda: client.responses.create(
    model=MODEL, input="What's the weather in Tokyo?",
    tools=[{"type": "function", "name": "get_weather", "description": "Get weather",
        "parameters": {"type": "object",
            "properties": {"city": {"type": "string"}, "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}},
            "required": ["city", "unit"], "additionalProperties": False}}]))
print(f"{'Function calling':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 4: Multi-turn
def multi_turn():
    r1 = client.responses.create(model=MODEL, input="Remember: the answer is 42.", store=True, max_output_tokens=32)
    client.responses.create(model=MODEL, input="What is the answer?", previous_response_id=r1.id, max_output_tokens=16)

med, p95, mn, mx = bench(multi_turn)
print(f"{'Multi-turn (2 reqs)':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 5: Web search
med, p95, mn, mx = bench(lambda: client.responses.create(
    model=MODEL, input="What is the latest Rust version?", max_output_tokens=100,
    tools=[{"type": "web_search", "search_context_size": "low"}]))
print(f"{'Web search':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

print(f"\n{ITERATIONS} iterations per test. All times include full HTTP round-trip.")
print(f"Client: openai-python v{__import__('openai').__version__}, httpx, HTTP/2.")
