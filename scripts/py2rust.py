#!/usr/bin/env python3
"""Convert Python Pydantic models (OpenAI SDK) to Rust serde structs.

Usage:
  python3 scripts/py2rust.py <python_dir> [--out FILE]
  python3 scripts/py2rust.py <python_file>
  python3 scripts/py2rust.py sync <python_dir> <rust_crate_dir>

Modes:
  (default)  Generate all types into one file or stdout
  sync       Update openai-oxide-types crate — generates AUTO files,
             preserves MANUAL files, reports new types from Python SDK

Examples:
  # Single file to stdout
  python3 scripts/py2rust.py ~/openai-python/src/openai/types/responses/easy_input_message.py

  # Whole directory into one file
  python3 scripts/py2rust.py ~/openai-python/src/openai/types/responses/ --out /tmp/responses.rs

  # Sync crate with Python SDK (main workflow)
  python3 scripts/py2rust.py sync ~/openai-python/src/openai/types/responses/ openai-oxide-types/src/responses/
"""

import ast
import re
import sys
from pathlib import Path


# Python type → Rust type mapping
TYPE_MAP = {
    "str": "String",
    "int": "i64",
    "float": "f64",
    "bool": "bool",
    "object": "serde_json::Value",
    "None": "()",
    "NoneType": "()",
    "bytes": "Vec<u8>",
    "dict": "serde_json::Value",
    "Dict": "serde_json::Value",
}


# Tracks struct names we've generated — used to resolve cross-references
_known_types: set[str] = set()


def python_type_to_rust(type_node: ast.expr, optional: bool = False,
                        field_name: str = "", class_name: str = "") -> str:
    """Convert a Python type annotation AST node to Rust type string."""

    if isinstance(type_node, ast.Constant):
        return TYPE_MAP.get(str(type_node.value), "serde_json::Value")

    if isinstance(type_node, ast.Name):
        name = type_node.id
        if name in TYPE_MAP:
            return TYPE_MAP[name]
        # Known types we've already generated — use as-is
        if name in _known_types:
            return name
        # Unknown type — use Value with hint for manual resolution
        return f"serde_json::Value /* TODO: {name} */"

    if isinstance(type_node, ast.Attribute):
        # e.g. Literal["foo"] shows up differently
        return "serde_json::Value"

    if isinstance(type_node, ast.Subscript):
        origin = type_node.value
        if isinstance(origin, ast.Name):
            origin_name = origin.id

            # Optional[T] → Option<T>
            if origin_name == "Optional":
                inner = python_type_to_rust(type_node.slice, field_name=field_name, class_name=class_name)
                return f"Option<{inner}>"

            # List[T] → Vec<T>
            if origin_name == "List":
                inner = python_type_to_rust(type_node.slice)
                return f"Vec<{inner}>"

            # Dict[K, V] → serde_json::Value (simplification)
            if origin_name == "Dict":
                return "serde_json::Value"

            # Literal["a", "b", "c"] → enum (handled separately)
            if origin_name == "Literal":
                return extract_literal(type_node, field_name, class_name)

            # Union[A, B] → enum or serde_json::Value
            if origin_name == "Union":
                return extract_union(type_node)

            # Annotated[T, ...] → T
            if origin_name == "Annotated":
                if isinstance(type_node.slice, ast.Tuple):
                    return python_type_to_rust(type_node.slice.elts[0])
                return python_type_to_rust(type_node.slice)

        # TypeAlias subscript
        if isinstance(origin, ast.Attribute):
            return "serde_json::Value"

    if isinstance(type_node, ast.BinOp) and isinstance(type_node.op, ast.BitOr):
        # T | None → Option<T>
        left = python_type_to_rust(type_node.left)
        right = python_type_to_rust(type_node.right)
        if right == "()" or right == "None":
            return f"Option<{left}>"
        if left == "()" or left == "None":
            return f"Option<{right}>"
        return "serde_json::Value"

    return "serde_json::Value"


# Accumulates enum definitions to emit before the struct that uses them
_pending_enums: list[str] = []


def literal_to_enum_name(field_name: str, class_name: str) -> str:
    """Generate enum name from field + class context."""
    parts = field_name.split("_")
    camel = "".join(p.capitalize() for p in parts)
    return f"{class_name}{camel}"


def value_to_variant(v: str) -> str:
    """Convert a literal string value to a Rust enum variant name."""
    # Handle special cases
    if v == "24h":
        return "Hours24"
    # Remove special chars, split by - and _, capitalize
    clean = v.replace("-", "_").replace(".", "_").replace(" ", "_")
    parts = clean.split("_")
    variant = "".join(p.capitalize() for p in parts if p)
    # Ensure starts with letter
    if variant and variant[0].isdigit():
        variant = "V" + variant
    return variant or "Unknown"


def extract_literal(node: ast.Subscript, field_name: str = "", class_name: str = "") -> str:
    """Extract Literal["a", "b"] — generate enum for 2+ variants, String for 1."""
    slice_node = node.slice
    if isinstance(slice_node, ast.Tuple):
        values = [
            elt.value if isinstance(elt, ast.Constant) else str(elt)
            for elt in slice_node.elts
        ]
    elif isinstance(slice_node, ast.Constant):
        values = [slice_node.value]
    else:
        return "String"

    if not all(isinstance(v, str) for v in values):
        return "String"

    # Single value — just a String (type discriminator, e.g. type: Literal["message"])
    if len(values) <= 1:
        return "String"

    # Generate enum
    enum_name = literal_to_enum_name(field_name, class_name)
    if enum_name in _known_types:
        return enum_name  # Already generated

    lines = []
    lines.append(f"#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]")
    lines.append(f"#[non_exhaustive]")
    lines.append(f"pub enum {enum_name} {{")
    for v in values:
        variant = value_to_variant(v)
        if variant.lower() != v:
            lines.append(f'    #[serde(rename = "{v}")]')
        lines.append(f"    {variant},")
    lines.append("}")

    _pending_enums.append("\n".join(lines))
    _known_types.add(enum_name)
    return enum_name


def extract_union(node: ast.Subscript) -> str:
    """Extract Union[A, B] and simplify."""
    slice_node = node.slice
    if isinstance(slice_node, ast.Tuple):
        types = [python_type_to_rust(elt) for elt in slice_node.elts]
        # Filter out None/()
        non_none = [t for t in types if t not in ("()", "None")]
        if len(non_none) == 1:
            return f"Option<{non_none[0]}>"
        if len(non_none) == 2 and "String" in non_none:
            other = [t for t in non_none if t != "String"][0]
            return f"serde_json::Value /* String | {other} */"
    return "serde_json::Value"


def extract_docstring(node) -> str | None:
    """Extract docstring from class or after field assignment."""
    if (
        isinstance(node, ast.ClassDef)
        and node.body
        and isinstance(node.body[0], ast.Expr)
        and isinstance(node.body[0].value, ast.Constant)
        and isinstance(node.body[0].value.value, str)
    ):
        return node.body[0].value.value.strip()
    return None


def field_docstring(body: list[ast.stmt], idx: int) -> str | None:
    """Check if the statement after body[idx] is a docstring expression."""
    if idx + 1 < len(body):
        next_stmt = body[idx + 1]
        if (
            isinstance(next_stmt, ast.Expr)
            and isinstance(next_stmt.value, ast.Constant)
            and isinstance(next_stmt.value.value, str)
        ):
            # First line only
            return next_stmt.value.value.strip().split("\n")[0]
    return None


def to_snake_case(name: str) -> str:
    """Convert CamelCase to snake_case."""
    s1 = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s1).lower()


def process_class(cls: ast.ClassDef) -> str:
    """Convert a Pydantic class to Rust struct."""
    _pending_enums.clear()
    lines = []

    # Docstring
    doc = extract_docstring(cls)
    if doc:
        for line in doc.split("\n"):
            lines.append(f"/// {line.strip()}")

    # Derive
    lines.append("#[derive(Debug, Clone, Serialize, Deserialize)]")

    # Check if it looks like an enum (all fields are Literal with single value)
    literal_fields = []
    regular_fields = []
    for node in cls.body:
        if isinstance(node, ast.AnnAssign) and node.target:
            annotation = node.annotation
            if (
                isinstance(annotation, ast.Subscript)
                and isinstance(annotation.value, ast.Name)
                and annotation.value.id == "Literal"
            ):
                literal_fields.append(node)
            else:
                regular_fields.append(node)

    lines.append(f"pub struct {cls.name} {{")

    for i, node in enumerate(cls.body):
        if not isinstance(node, ast.AnnAssign) or not node.target:
            continue

        field_name = node.target.id if isinstance(node.target, ast.Name) else str(node.target)
        rust_type = python_type_to_rust(node.annotation, field_name=field_name, class_name=cls.name)

        # Handle Optional with default None
        is_optional = node.value is not None and (
            (isinstance(node.value, ast.Constant) and node.value.value is None)
        )
        if is_optional and not rust_type.startswith("Option<"):
            rust_type = f"Option<{rust_type}>"

        # Field docstring
        fdoc = field_docstring(cls.body, i)
        if fdoc:
            lines.append(f"    /// {fdoc}")

        # serde attributes
        serde_attrs = []
        if rust_type.startswith("Option<"):
            serde_attrs.append('#[serde(skip_serializing_if = "Option::is_none")]')
            serde_attrs.append("#[serde(default)]")

        # Rename reserved words
        rust_field = field_name
        rename = None
        if field_name == "type":
            rust_field = "type_"
            rename = "type"
        elif field_name == "r#type":
            rust_field = "type_"
            rename = "type"

        if rename:
            serde_attrs.insert(0, f'#[serde(rename = "{rename}")]')

        for attr in serde_attrs:
            lines.append(f"    {attr}")

        lines.append(f"    pub {rust_field}: {rust_type},")

    lines.append("}")

    # Prepend any enums generated from Literal fields
    parts = list(_pending_enums) + ["\n".join(lines)]
    _pending_enums.clear()
    return "\n\n".join(parts)


def process_file(path: Path, prefix: str = "") -> list[tuple[str, str]]:
    """Parse a Python file and return list of (struct_name, rust_code) tuples."""
    source = path.read_text()
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return []

    results = []
    for node in ast.walk(tree):
        if isinstance(node, ast.ClassDef):
            for base in node.bases:
                if (isinstance(base, ast.Name) and base.id == "BaseModel") or (
                    isinstance(base, ast.Attribute) and base.attr == "BaseModel"
                ):
                    # Prefix inner classes to avoid name collisions across files
                    name = node.name
                    if prefix and name[0].isupper() and len(name) < 20:
                        # Short generic names like "Content", "Part", "Summary"
                        # get prefixed with parent type name
                        generic_names = {
                            "Content", "Part", "Summary", "Action", "Output",
                            "Result", "Tool", "Item", "Error", "Details",
                            "Environment", "Operation", "Outcome",
                        }
                        if name in generic_names:
                            name = f"{prefix}{name}"
                            node = ast.parse(
                                ast.unparse(node).replace(
                                    f"class {node.name}",
                                    f"class {name}",
                                )
                            ).body[0]
                    _known_types.add(name)
                    results.append((name, process_class(node)))
                    break

    return results


def file_prefix(path: Path) -> str:
    """Derive a prefix from filename for dedup: response_reasoning_item.py → ResponseReasoning."""
    stem = path.stem
    # Remove common prefixes
    for p in ("response_", "responses_"):
        if stem.startswith(p):
            stem = stem[len(p):]
    parts = stem.split("_")
    return "".join(p.capitalize() for p in parts[:2])


# ── Python filename → Rust file routing ──

# Maps Python filename patterns to Rust destination file.
# Order matters — first match wins.
ROUTE_TABLE = [
    # Streaming events
    ("response_*_event.py", "streaming"),
    ("response_completed_event.py", "streaming"),
    ("response_failed_event.py", "streaming"),
    ("response_incomplete_event.py", "streaming"),
    ("response_queued_event.py", "streaming"),
    # Tools
    ("function_tool.py", "tools"),
    ("file_search_tool.py", "tools"),
    ("computer_tool.py", "tools"),
    ("computer_use_preview_tool.py", "tools"),
    ("web_search_tool.py", "tools"),
    ("code_interpreter_tool.py", "tools"),
    ("custom_tool.py", "tools"),
    ("apply_patch_tool.py", "tools"),
    ("function_shell_tool.py", "tools"),
    ("namespace_tool.py", "tools"),
    # Output types
    ("response_function_tool_call*.py", "output"),
    ("response_output_*.py", "output"),
    ("response_reasoning_item.py", "output"),
    ("response_custom_tool_call*.py", "output"),
    ("response_computer_tool_call*.py", "output"),
    ("response_file_search_tool_call*.py", "output"),
    ("response_code_interpreter_tool_call*.py", "output"),
    # Input types
    ("easy_input_message.py", "input"),
    ("response_input_*.py", "input"),
    ("input_item*.py", "input"),
    ("response_function_call_output*.py", "input"),
    # Response
    ("response.py", "response"),
    ("compacted_response.py", "response"),
    ("parsed_response.py", "response"),
    # Create/config
    ("response_format*.py", "create"),
    ("container_*.py", "create"),
    ("local_*.py", "create"),
    ("inline_skill*.py", "create"),
]

import fnmatch


def route_python_file(filename: str) -> str:
    """Determine which Rust file a Python file's types should go into."""
    for pattern, dest in ROUTE_TABLE:
        if fnmatch.fnmatch(filename, pattern):
            return dest
    return "extra"  # Uncategorized — goes to extra.rs


def is_manual_file(rust_file: Path) -> bool:
    """Check if a Rust file is marked MANUAL (hand-maintained, don't overwrite)."""
    if not rust_file.exists():
        return False
    first_lines = rust_file.read_text()[:200]
    return "// MANUAL" in first_lines


def generate_file_header(dest_name: str) -> str:
    """Generate header for an auto-generated Rust file."""
    return (
        f"// AUTO-GENERATED — do not edit manually.\n"
        f"// Re-generate: python3 scripts/py2rust.py sync <python_dir> <rust_dir>\n"
        f"// Source: OpenAI Python SDK responses/\n\n"
        f"use serde::{{Deserialize, Serialize}};\n"
    )


def sync_crate(python_dir: Path, rust_dir: Path):
    """Sync Python SDK types into openai-oxide-types crate structure."""
    # Collect all Python types, routed to Rust files
    routed: dict[str, list[tuple[str, str]]] = {}  # dest → [(name, code)]
    seen_names: set[str] = set()

    for f in sorted(python_dir.glob("*.py")):
        if f.name.startswith("_") or f.name.endswith("_param.py"):
            continue
        dest = route_python_file(f.name)
        prefix = file_prefix(f)
        structs = process_file(f, prefix=prefix)
        for name, code in structs:
            if name in seen_names:
                continue
            seen_names.add(name)
            routed.setdefault(dest, []).append((name, code))

    # Read existing manual files to see which types we already have
    existing_types: set[str] = set()
    for rs_file in rust_dir.glob("*.rs"):
        if rs_file.name == "mod.rs":
            continue
        content = rs_file.read_text()
        for line in content.split("\n"):
            if line.startswith("pub struct ") or line.startswith("pub enum "):
                type_name = line.split("{")[0].split("(")[0].strip().split()[-1]
                existing_types.add(type_name)

    # Report
    total_generated = 0
    total_new = 0
    total_skipped = 0

    for dest, types in sorted(routed.items()):
        rust_path = rust_dir / f"{dest}.rs"

        if is_manual_file(rust_path):
            # Don't overwrite manual files — just report new types
            new_types = [name for name, _ in types if name not in existing_types]
            if new_types:
                print(f"  MANUAL {dest}.rs — {len(new_types)} new types to add:")
                for t in new_types:
                    print(f"    + {t}")
                total_new += len(new_types)
            else:
                print(f"  MANUAL {dest}.rs — up to date")
            total_skipped += len(types)
            continue

        # Auto-generated file — overwrite
        content = generate_file_header(dest)
        # Import from sibling modules if needed
        if dest not in ("common", "mod"):
            content += "use super::common::*;\n"
            if dest == "streaming":
                content += "use super::output::OutputItem;\n"
                content += "use super::response::Response;\n"
            elif dest == "output":
                content += "use super::response::Response;\n"
        content += "\n"
        content += "\n\n".join(code for _, code in types)
        content += "\n"

        rust_path.write_text(content)
        print(f"  AUTO   {dest}.rs — {len(types)} types")
        total_generated += len(types)

    print(f"\nSummary: {total_generated} generated, {total_skipped} in manual files, {total_new} new types to review")


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    # Sync mode
    if sys.argv[1] == "sync":
        if len(sys.argv) < 4:
            print("Usage: py2rust.py sync <python_dir> <rust_dir>")
            sys.exit(1)
        python_dir = Path(sys.argv[2])
        rust_dir = Path(sys.argv[3])
        print(f"Syncing {python_dir} → {rust_dir}\n")
        sync_crate(python_dir, rust_dir)
        sys.exit(0)

    # Default mode — single file or flat output
    target = Path(sys.argv[1])
    out_file = None
    if "--out" in sys.argv:
        out_file = Path(sys.argv[sys.argv.index("--out") + 1])

    all_structs: list[tuple[str, str]] = []
    seen_names: set[str] = set()

    if target.is_file():
        all_structs.extend(process_file(target))
    elif target.is_dir():
        for f in sorted(target.glob("*.py")):
            if f.name.startswith("_") or f.name.endswith("_param.py"):
                continue
            prefix = file_prefix(f)
            structs = process_file(f, prefix=prefix)
            for name, code in structs:
                if name in seen_names:
                    continue
                seen_names.add(name)
                all_structs.append((name, code))

    header = "// Auto-generated from Python OpenAI SDK. Do not edit manually.\n"
    header += f"// Re-generate: python3 scripts/py2rust.py {target}\n"
    header += f"// Structs: {len(all_structs)}\n\n"
    header += "use serde::{Deserialize, Serialize};\n"

    output = header + "\n\n".join(code for _, code in all_structs)

    if out_file:
        out_file.write_text(output + "\n")
        print(f"Wrote {len(all_structs)} structs to {out_file}")
    else:
        print(output)


if __name__ == "__main__":
    main()
