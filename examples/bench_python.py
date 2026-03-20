#!/usr/bin/env python3
"""Python openai SDK benchmark — same tests as Rust, with connection warmup."""

import json
import time

from openai import OpenAI

client = OpenAI()
MODEL = "gpt-5.4"

# Warmup — establish TLS + HTTP/2
print("Warming up...")
client.responses.create(model=MODEL, input="ping", max_output_tokens=16)
print("Ready.\n")

print(f"=== Python openai SDK v{__import__('openai').__version__} (warm) ===")
print(f"Model: {MODEL}\n")


def bench(fn) -> float:
    t0 = time.perf_counter()
    fn()
    return (time.perf_counter() - t0) * 1000


# Test 1: Plain text
print("--- Test 1: Plain text ---")
ms = bench(lambda: client.responses.create(
    model=MODEL,
    instructions="You are a helpful assistant. Be concise.",
    input="What is the capital of Kazakhstan? One sentence.",
    max_output_tokens=50,
))
print(f"  python ({ms:>7.0f}ms)")

# Test 2: Structured output
print("\n--- Test 2: Structured output ---")
ms = bench(lambda: client.responses.create(
    model=MODEL,
    instructions="You are a geography expert.",
    input="Tell me about Tokyo.",
    max_output_tokens=500,
    text={"format": {
        "type": "json_schema",
        "name": "city_info",
        "strict": True,
        "schema": {
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "country": {"type": "string"},
                "population": {"type": "integer"},
                "landmarks": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["name", "country", "population", "landmarks"],
            "additionalProperties": False,
        },
    }},
))
print(f"  python ({ms:>7.0f}ms)")

# Test 3: Function calling
print("\n--- Test 3: Function calling ---")
ms = bench(lambda: client.responses.create(
    model=MODEL,
    instructions="Use tools when needed.",
    input="What's the weather in Moscow?",
    tools=[{
        "type": "function",
        "name": "get_weather",
        "description": "Get current weather for a city",
        "strict": True,
        "parameters": {
            "type": "object",
            "properties": {
                "city": {"type": "string", "description": "City name"},
                "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
            },
            "required": ["city", "unit"],
            "additionalProperties": False,
        },
    }],
))
print(f"  python ({ms:>7.0f}ms)")

# Test 4: Multi-turn
print("\n--- Test 4: Multi-turn (2 requests) ---")
def multi_turn():
    r1 = client.responses.create(
        model=MODEL, input="My name is Rustam.", max_output_tokens=50, store=True)
    client.responses.create(
        model=MODEL, input="What is my name?",
        previous_response_id=r1.id, max_output_tokens=50)

ms = bench(multi_turn)
print(f"  python ({ms:>7.0f}ms)")

print("\n=== Done ===")
