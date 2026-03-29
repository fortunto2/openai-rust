#!/usr/bin/env python3
"""migrate_types.py — Move hand-crafted types from src/types/ to openai-types/src/.

Reads each src/types/{domain}.rs, extracts everything that isn't a test module
or a re-export from openai_types, and writes it as a manual override file in
openai-types/src/{domain}/manual.rs.

After running this:
  1. python3 scripts/py2rust.py sync ... (regenerates _gen.rs, skips manual types)
  2. Update src/types/{domain}.rs to `pub use openai_types::{domain}::*;`
  3. cargo test
"""

import re
import sys
from pathlib import Path

SRC_TYPES = Path("src/types")
OPENAI_TYPES = Path("openai-types/src")

# Domains to migrate (skip audio — already done, common — stays in src)
DOMAIN_MAP = {
    "batch": "batch",
    "embedding": "embedding",
    "file": "file",
    "upload": "uploads",
    "moderation": "moderation",
    "fine_tuning": "fine_tuning",
    "image": "image",
    "chat": "chat",
    "beta": "beta",
    "realtime": "realtime",
}

# Already migrated
SKIP = {"audio", "model", "common", "responses"}

# Imports to strip (they reference crate-internal items or are re-exports)
CRATE_IMPORTS = re.compile(
    r"^use (?:crate|super)::.*$|^pub use openai_types::.*\{[^}]*\};?$|^pub use openai_types::.*$|^use crate::openai_enum;$",
    re.MULTILINE,
)

# Also strip orphaned lines from multi-line pub use blocks
ORPHAN_USE_LINES = re.compile(
    r"^    \w+.*,$\n|^\};$",
    re.MULTILINE,
)

# openai_enum! macro invocations — convert to plain derive
OPENAI_ENUM_RE = re.compile(
    r"openai_enum!\s*\{(.*?)\}", re.DOTALL
)


def expand_openai_enum(match: re.Match) -> str:
    """Expand openai_enum! { ... } to standard derive enum."""
    body = match.group(1).strip()
    # Parse: /// doc\n pub enum Name { Variant = "value", ... }
    lines = body.split("\n")
    result = []
    in_enum = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("///"):
            result.append(line)
        elif stripped.startswith("pub enum"):
            result.append("#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]")
            result.append('#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]')
            result.append("#[non_exhaustive]")
            # Extract enum name
            enum_name = stripped.split("{")[0].strip()
            result.append(f"{enum_name} {{")
            in_enum = True
        elif in_enum and "=" in stripped:
            # Variant = "value",
            parts = stripped.split("=")
            variant = parts[0].strip()
            value = parts[1].strip().strip('",').strip('"')
            # Check if variant lowercase matches value (no rename needed)
            variant_lower = re.sub(r"([A-Z])", r"_\1", variant).lower().strip("_")
            # Use rename_all=snake_case approach: check if simple snake_case works
            simple_snake = variant.lower()
            if value == simple_snake or value == variant_lower:
                result.append(f"    {variant},")
            else:
                result.append(f'    #[serde(rename = "{value}")]')
                result.append(f"    {variant},")
        elif in_enum and stripped == "}":
            result.append("}")
            in_enum = False
        elif in_enum and stripped:
            result.append(line)
    return "\n".join(result)


def transform_source(source: str, domain: str) -> str:
    """Transform src/types/{domain}.rs into openai-types manual override."""
    # Remove crate-specific imports and re-export blocks
    source = CRATE_IMPORTS.sub("", source)
    source = ORPHAN_USE_LINES.sub("", source)

    # Expand openai_enum! macros
    source = OPENAI_ENUM_RE.sub(expand_openai_enum, source)

    # Fix crate references in types
    source = source.replace("crate::types::file::FilePurpose", "super::FilePurpose")
    source = source.replace("crate::types::file::FileObject", "super::FileObject")
    source = source.replace("crate::types::common::", "")

    # Remove test modules
    test_start = source.find("#[cfg(test)]")
    if test_start != -1:
        source = source[:test_start]

    # Remove comments about mirrors
    source = re.sub(r"^// .* types — .*$", "", source, flags=re.MULTILINE)

    # Add proper header
    header = f"// Manual: hand-crafted {domain} types (enums, builders, precise Optional fields).\n\n"
    header += "use serde::{Deserialize, Serialize};\n"

    # Clean up multiple blank lines
    source = re.sub(r"\n{3,}", "\n\n", source).strip()

    return header + "\n" + source + "\n"


def extract_tests(source: str) -> str:
    """Extract test module from source."""
    test_start = source.find("#[cfg(test)]")
    if test_start == -1:
        return ""
    return source[test_start:]


def migrate_domain(src_name: str, dst_name: str):
    """Migrate one domain's types."""
    src_file = SRC_TYPES / f"{src_name}.rs"
    dst_dir = OPENAI_TYPES / dst_name
    manual_file = dst_dir / "manual.rs"

    if not src_file.exists():
        print(f"  SKIP {src_name} — not found")
        return

    source = src_file.read_text()

    # Skip if already a pure re-export
    if source.strip().startswith("pub use openai_types::"):
        print(f"  SKIP {src_name} — already re-exported")
        return

    # Check if manual.rs already exists
    if manual_file.exists():
        print(f"  SKIP {src_name} — manual.rs already exists")
        return

    # Transform
    manual_source = transform_source(source, src_name)

    # Count types
    type_count = len(re.findall(r"pub (?:struct|enum|type) \w+", manual_source))

    # Write manual override
    dst_dir.mkdir(parents=True, exist_ok=True)
    manual_file.write_text(manual_source)

    # Extract tests for src/types re-export file
    tests = extract_tests(source)

    # Write new src/types re-export
    reexport = f"// {src_name.replace('_', ' ').title()} types — re-exported from openai-types.\n\n"
    reexport += f"pub use openai_types::{dst_name}::*;\n"
    if tests:
        reexport += "\n" + tests

    src_file.write_text(reexport)

    print(f"  {src_name} → openai-types/{dst_name}/manual.rs ({type_count} types)")


def main():
    print("Migrating hand-crafted types to openai-types...\n")

    for src_name, dst_name in sorted(DOMAIN_MAP.items()):
        if src_name in SKIP:
            continue
        migrate_domain(src_name, dst_name)

    print("\nNext steps:")
    print("  1. python3 scripts/py2rust.py sync ~/startups/shared/openai-python/src/openai/types/ openai-types/src/")
    print("  2. cargo check && cargo test")
    print("  3. Fix any cross-module references (super:: paths, etc.)")


if __name__ == "__main__":
    main()
