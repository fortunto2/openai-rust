#!/usr/bin/env python3
"""Inject generated benchmark tables into README.md and sub-READMEs.

Replaces content between <!-- BENCH:python:START --> and <!-- BENCH:python:END --> markers.

Usage: python3 benchmarks/update-readme.py
"""

import re
from pathlib import Path

ROOT = Path(__file__).parent.parent
BENCH = Path(__file__).parent


def inject(file: Path, tag: str, content: str):
    """Replace content between <!-- BENCH:{tag}:START --> and <!-- BENCH:{tag}:END -->."""
    text = file.read_text()
    pattern = rf"(<!-- BENCH:{tag}:START -->).*?(<!-- BENCH:{tag}:END -->)"
    replacement = rf"\1\n{content}\n\2"
    new_text = re.sub(pattern, replacement, text, flags=re.DOTALL)
    if new_text != text:
        file.write_text(new_text)
        print(f"  Updated {file} [{tag}]")
    else:
        print(f"  No markers for [{tag}] in {file}")


def main():
    rust_md = (BENCH / "rust.md").read_text().strip()
    python_md = (BENCH / "python.md").read_text().strip()
    node_md = (BENCH / "node.md").read_text().strip()

    # Main README
    readme = ROOT / "README.md"
    inject(readme, "rust", rust_md)
    inject(readme, "python", python_md)
    inject(readme, "node", node_md)

    # Node README
    node_readme = ROOT / "openai-oxide-node" / "README.md"
    if node_readme.exists():
        inject(node_readme, "node", node_md)

    print("Done.")


if __name__ == "__main__":
    main()
