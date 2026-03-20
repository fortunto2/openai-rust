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

# Test 6: Nested structured output
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

med, p95, mn, mx = bench(lambda: json.loads(client.responses.create(
    model=MODEL, input="Analyze Apple Inc: products with revenue, competitors, summary.",
    max_output_tokens=800,
    text={"format": {"type": "json_schema", "name": "company_analysis", "strict": True,
        "schema": complex_schema}}).output_text))
print(f"{'Nested structured':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 7: Agent loop (2-step: FC → result → structured)
def agent_loop():
    step1 = client.responses.create(
        model=MODEL, input="What's the weather in Tokyo and what should I wear?",
        store=True,
        tools=[{"type": "function", "name": "get_weather", "description": "Get weather",
            "parameters": {"type": "object",
                "properties": {"city": {"type": "string"}, "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]}},
                "required": ["city", "unit"], "additionalProperties": False}}])
    call_id = next((item.call_id for item in step1.output if item.type == "function_call"), "call_1")
    client.responses.create(
        model=MODEL, previous_response_id=step1.id, max_output_tokens=200,
        input=[{"type": "function_call_output", "call_id": call_id,
            "output": '{"temp":22,"condition":"sunny","humidity":45}'}],
        text={"format": {"type": "json_schema", "name": "recommendation", "strict": True,
            "schema": {"type": "object", "properties": {
                "outfit": {"type": "string"},
                "accessories": {"type": "array", "items": {"type": "string"}},
                "warning": {"type": "string"}},
                "required": ["outfit", "accessories", "warning"], "additionalProperties": False}}})

med, p95, mn, mx = bench(agent_loop)
print(f"{'Agent loop (2-step)':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 8: Rapid-fire (5 sequential calls)
def rapid_fire():
    for i in range(1, 6):
        client.responses.create(model=MODEL, input=f"What is {i}+{i}? Reply with just the number.", max_output_tokens=16)

med, p95, mn, mx = bench(rapid_fire)
print(f"{'Rapid-fire (5 calls)':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 9: Prompt-cached (repeated system prompt with cache key)
system_prompt = "You are a senior software architect with 20 years of experience in distributed systems, microservices, and cloud-native architectures. Always provide specific, actionable advice with code examples where relevant. Consider scalability, maintainability, and security in every recommendation."
client.responses.create(model=MODEL, instructions=system_prompt, input="ping",
    prompt_cache_key="bench-architect", max_output_tokens=16)
med, p95, mn, mx = bench(lambda: client.responses.create(
    model=MODEL, instructions=system_prompt,
    input="How should I design a rate limiter for an API gateway?",
    prompt_cache_key="bench-architect", prompt_cache_retention="24h",
    max_output_tokens=200))
print(f"{'Prompt-cached':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 10: Streaming TTFT
times = []
for _ in range(ITERATIONS):
    t0 = time.perf_counter()
    stream = client.responses.create(
        model=MODEL, input="Explain quicksort in 3 sentences.", max_output_tokens=200, stream=True)
    for event in stream:
        if event.type == "response.output_text.delta":
            times.append(int((time.perf_counter() - t0) * 1000))
            break
med, p95, mn, mx = stats(times)
print(f"{'Streaming TTFT':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 11: Parallel 3x (Python uses threads since httpx is sync)
import concurrent.futures
def single_call(q):
    return client.responses.create(model=MODEL, input=f"Capital of {q}? One word.", max_output_tokens=16)

times = []
for _ in range(ITERATIONS):
    t0 = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as ex:
        fs = [ex.submit(single_call, q) for q in ["France", "Japan", "Brazil"]]
        for f in concurrent.futures.as_completed(fs):
            f.result()
    times.append(int((time.perf_counter() - t0) * 1000))
med, p95, mn, mx = stats(times)
print(f"{'Parallel 3x (fan-out)':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

# Test 12: Hedged request (send 2, take first)
times = []
for _ in range(ITERATIONS):
    t0 = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:
        f1 = ex.submit(lambda: client.responses.create(model=MODEL, input="What is 7*8? Number only.", max_output_tokens=16))
        f2 = ex.submit(lambda: client.responses.create(model=MODEL, input="What is 7*8? Number only.", max_output_tokens=16))
        done, _ = concurrent.futures.wait([f1, f2], return_when=concurrent.futures.FIRST_COMPLETED)
        next(iter(done)).result()
    times.append(int((time.perf_counter() - t0) * 1000))
med, p95, mn, mx = stats(times)
print(f"{'Hedged (2x race)':<25} {med:>6}ms {p95:>6}ms {mn:>6}ms {mx:>6}ms")

print(f"\n{ITERATIONS} iterations per test. All times include full HTTP round-trip.")
print(f"Client: openai-python v{__import__('openai').__version__}, httpx.")
