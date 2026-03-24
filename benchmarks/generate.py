#!/usr/bin/env python3
"""Generate benchmark markdown tables from results.json.

Outputs:
  benchmarks/rust.md    — Rust ecosystem table
  benchmarks/python.md  — Python ecosystem table
  benchmarks/node.md    — Node ecosystem table
  benchmarks/all.md     — Combined (for docs)

Usage: python3 benchmarks/generate.py
"""

import json
from pathlib import Path

ROOT = Path(__file__).parent
DATA = json.loads((ROOT / "results.json").read_text())
META = DATA["meta"]


def winner(oxide, other, oxide_label="OXIDE", other_label="other"):
    if oxide is None:
        return f"**{other}ms**", "N/A", other_label
    if other is None:
        return f"**{oxide}ms**", "N/A", oxide_label
    if oxide <= other:
        pct = round((other - oxide) / other * 100)
        return f"**{oxide}ms**", f"{other}ms", f"{oxide_label} (+{pct}%)"
    else:
        pct = round((oxide - other) / other * 100)
        return f"{oxide}ms", f"**{other}ms**", f"{other_label} (+{pct}%)"


def gen_rust():
    lines = [
        f"### Rust Ecosystem (`openai-oxide` vs `async-openai` vs `genai`)\n",
        f"| Test | `openai-oxide` | `async-openai` | `genai` |",
        "| :--- | :--- | :--- | :--- |",
    ]
    for t in DATA["rust"]["tests"]:
        ox = t["oxide"]
        ao = t["async_openai"]
        ge = t["genai"]
        ox_s = f"**{ox}ms**" if (ao is None or ox <= ao) and (ge is None or ox <= ge) else f"{ox}ms"
        ao_s = "N/A" if ao is None else f"{ao}ms"
        ge_s = "N/A" if ge is None else f"{ge}ms"
        lines.append(f"| **{t['name']}** | {ox_s} | {ao_s} | {ge_s} |")
    lines.append("")
    lines.append("#### WebSocket mode (openai-oxide only)\n")
    lines.append("| Test | WebSocket | HTTP | Improvement |")
    lines.append("| :--- | :--- | :--- | :--- |")
    for t in DATA["rust"]["websocket"]:
        lines.append(f"| **{t['name']}** | **{t['ws']}ms** | {t['http']}ms | {t['improvement']} |")
    lines.append(f"\n*{META['method']}, {META['runs']}×{META['iterations_per_run']} iterations. Model: {META['model']}. {META['environment']}.*")
    lines.append(f"\nReproduce: `cargo run --example benchmark --features responses --release`\n")
    return "\n".join(lines)


def gen_python():
    d = DATA["python"]
    wins = d["wins"]
    lines = [
        f"### Python Ecosystem (`openai-oxide-python` vs `openai`)\n",
        f"`openai-oxide` wins **{wins}** tests. Native PyO3 bindings vs `openai` ({d['official_version']}).\n",
        "| Test | `openai-oxide` | `openai` | Winner |",
        "| :--- | :--- | :--- | :--- |",
    ]
    for t in d["tests"]:
        ox_s, of_s, w = winner(t["oxide"], t["official"], "OXIDE", "python")
        lines.append(f"| **{t['name']}** | {ox_s} | {of_s} | {w} |")
    lines.append(f"\n*{META['method']}, {META['runs']}×{META['iterations_per_run']} iterations. Model: {META['model']}.*")
    lines.append(f"\nReproduce: `cd openai-oxide-python && uv run python ../examples/bench_python.py`\n")
    return "\n".join(lines)


def gen_node():
    d = DATA["node"]
    wins = d["wins"]
    lines = [
        f"### Node.js Ecosystem (`openai-oxide` vs `openai`)\n",
        f"`openai-oxide` wins **{wins}** tests. Native napi-rs bindings vs official `openai` npm.\n",
        "| Test | `openai-oxide` | `openai` | Winner |",
        "| :--- | :--- | :--- | :--- |",
    ]
    for t in d["tests"]:
        ox_s, of_s, w = winner(t["oxide"], t["official"], "OXIDE", "openai")
        lines.append(f"| **{t['name']}** | {ox_s} | {of_s} | {w} |")
    lines.append(f"\n*{META['method']}, {META['runs']}×{META['iterations_per_run']} iterations. Model: {META['model']}.*")
    lines.append(f"\nReproduce: `cd openai-oxide-node && BENCH_ITERATIONS=5 node examples/bench_node.js`\n")
    return "\n".join(lines)


def main():
    rust = gen_rust()
    python = gen_python()
    node = gen_node()

    (ROOT / "rust.md").write_text(rust)
    (ROOT / "python.md").write_text(python)
    (ROOT / "node.md").write_text(node)
    (ROOT / "all.md").write_text(f"## Benchmarks\n\n{rust}\n---\n\n{python}\n---\n\n{node}")

    print("Generated:")
    for f in ["rust.md", "python.md", "node.md", "all.md"]:
        print(f"  benchmarks/{f}")


if __name__ == "__main__":
    main()
