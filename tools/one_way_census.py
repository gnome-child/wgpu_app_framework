#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Report the monolith's provisional virtual-crate dependency gauge.

This is deliberately a reporting tool, not a suite gate.  It understands the
Rust path forms that matter to the current source tree, keeps cfg(test) edges
separate, and reports enough receipts to audit any surprising edge by hand.
The slot map is provisional campaign data in ``one_way_slots.json``.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


@dataclass
class Edge:
    source: str
    target: str
    locations: set[str] = field(default_factory=set)


@dataclass
class Census:
    root: Path
    modules: list[str]
    module_slots: dict[str, str]
    production_edges: dict[tuple[str, str], Edge]
    test_edges: dict[tuple[str, str], Edge]
    external_users: dict[str, set[str]]
    pub_crate_total: int
    pub_crate_files: int
    cross_slot_pub_crate_upper_bound: int
    cross_slot_test_edges: int
    manifest_roots: int
    filesystem_reads: int
    allow_attributes: int
    panic_calls: int
    expect_calls: int


def workspace_root(start: Path | None = None) -> Path:
    here = (start or Path(__file__).resolve().parent).resolve()
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=here,
            check=True,
            capture_output=True,
            text=True,
        )
        return Path(result.stdout.strip()).resolve()
    except (FileNotFoundError, subprocess.CalledProcessError):
        return Path(__file__).resolve().parents[1]


def mask_rust_literals_and_comments(source: str) -> str:
    """Return a same-length string with comments and literals blanked."""

    chars = list(source)
    out = chars.copy()
    i = 0
    block_depth = 0
    while i < len(chars):
        if block_depth:
            if source.startswith("/*", i):
                out[i : i + 2] = "  "
                block_depth += 1
                i += 2
            elif source.startswith("*/", i):
                out[i : i + 2] = "  "
                block_depth -= 1
                i += 2
            else:
                if chars[i] != "\n":
                    out[i] = " "
                i += 1
            continue

        if source.startswith("//", i):
            end = source.find("\n", i)
            if end < 0:
                end = len(chars)
            for j in range(i, end):
                out[j] = " "
            i = end
            continue
        if source.startswith("/*", i):
            out[i : i + 2] = "  "
            block_depth = 1
            i += 2
            continue

        raw = None
        if chars[i] in {"b", "c", "r"}:
            raw = re.match(r"(?:b|c)?r(#{0,255})\"", source[i:])
        if raw:
            hashes = raw.group(1)
            opener = raw.group(0)
            closer = '"' + hashes
            end = source.find(closer, i + len(opener))
            end = len(chars) if end < 0 else end + len(closer)
            for j in range(i, end):
                if chars[j] != "\n":
                    out[j] = " "
            i = end
            continue

        string_prefix = None
        if chars[i] in {"b", "c", '"'}:
            for prefix in ('b"', 'c"', '"'):
                if source.startswith(prefix, i):
                    string_prefix = prefix
                    break
        if string_prefix:
            j = i + len(string_prefix)
            escaped = False
            while j < len(chars):
                if not escaped and chars[j] == '"':
                    j += 1
                    break
                if chars[j] == "\\" and not escaped:
                    escaped = True
                else:
                    escaped = False
                j += 1
            for k in range(i, j):
                if chars[k] != "\n":
                    out[k] = " "
            i = j
            continue

        # Character literals contain one value; lifetimes such as 'a must stay.
        if chars[i] == "'" and i + 2 < len(chars):
            char_match = re.match(r"'(?:\\.|[^\\'\n])'", source[i:])
            if char_match:
                end = i + len(char_match.group(0))
                out[i:end] = " " * (end - i)
                i = end
                continue
        i += 1
    return "".join(out)


def matching_brace(source: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def test_ranges(masked: str) -> list[tuple[int, int]]:
    """Find cfg(test) and #[test] item ranges in a masked Rust source."""

    ranges: list[tuple[int, int]] = []
    attr = re.compile(r"#\s*\[\s*(?:cfg\s*\([^\]]*\btest\b[^\]]*\)|test)\s*\]")
    for match in attr.finditer(masked):
        cursor = match.end()
        # Include additional attributes before the item.
        while True:
            whitespace = re.match(r"\s*", masked[cursor:])
            cursor += whitespace.end() if whitespace else 0
            if not masked.startswith("#[", cursor):
                break
            close = masked.find("]", cursor + 2)
            if close < 0:
                break
            cursor = close + 1
        brace = masked.find("{", cursor)
        semicolon = masked.find(";", cursor)
        if semicolon >= 0 and (brace < 0 or semicolon < brace):
            ranges.append((match.start(), semicolon + 1))
        elif brace >= 0:
            close = matching_brace(masked, brace)
            if close is not None:
                ranges.append((match.start(), close + 1))
    # Merge nested/overlapping ranges, especially #[cfg(test)] mod tests.
    merged: list[tuple[int, int]] = []
    for start, end in sorted(ranges):
        if merged and start <= merged[-1][1]:
            merged[-1] = (merged[-1][0], max(end, merged[-1][1]))
        else:
            merged.append((start, end))
    return merged


def partition_test_code(masked: str, force_test: bool = False) -> tuple[str, str]:
    if force_test:
        return " " * len(masked), masked
    ranges = test_ranges(masked)
    production = list(masked)
    tests = ["\n" if char == "\n" else " " for char in masked]
    for start, end in ranges:
        for index in range(start, end):
            tests[index] = masked[index]
            if production[index] != "\n":
                production[index] = " "
    return "".join(production), "".join(tests)


def module_path(src: Path, path: Path) -> list[str]:
    relative = path.relative_to(src)
    parts = list(relative.parts)
    if parts[-1] == "lib.rs":
        return []
    if parts[-1] == "mod.rs":
        parts.pop()
    else:
        parts[-1] = Path(parts[-1]).stem
    return parts


def external_module_candidates(parent: Path, name: str) -> tuple[Path, Path]:
    """Return Rust's ordinary file candidates for an external child module."""

    directory = parent.parent if parent.name in {"lib.rs", "mod.rs"} else parent.with_suffix("")
    return directory / f"{name}.rs", directory / name / "mod.rs"


def test_only_module_roots(
    src: Path, masked_files: dict[Path, str]
) -> set[tuple[str, ...]]:
    """Resolve external modules whose declarations live in cfg(test) code."""

    roots: set[tuple[str, ...]] = set()
    declaration = re.compile(rf"\bmod\s+({IDENT})\s*;")
    available = set(masked_files)
    for parent, masked in masked_files.items():
        _, tests = partition_test_code(masked)
        for name in declaration.findall(tests):
            for candidate in external_module_candidates(parent, name):
                if candidate in available:
                    roots.add(tuple(module_path(src, candidate)))
                    break
    return roots


def belongs_to_test_only_module(
    parts: list[str], roots: set[tuple[str, ...]]
) -> bool:
    path = tuple(parts)
    return any(path[: len(root)] == root for root in roots)


def grouped_roots(source: str, prefix: str) -> set[str]:
    roots: set[str] = set()
    pattern = re.compile(rf"\b{re.escape(prefix)}\s*::\s*\{{")
    for match in pattern.finditer(source):
        opening = source.find("{", match.start())
        close = matching_brace(source, opening)
        if close is None:
            continue
        body = source[opening + 1 : close]
        depth = 0
        start = 0
        items: list[str] = []
        for index, char in enumerate(body):
            if char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
            elif char == "," and depth == 0:
                items.append(body[start:index])
                start = index + 1
        items.append(body[start:])
        for item in items:
            root = re.search(rf"\b({IDENT})\b", item)
            if root and root.group(1) not in {"self", "super", "crate"}:
                roots.add(root.group(1))
    return roots


def grouped_super_roots(source: str, path_parts: list[str]) -> set[str]:
    roots: set[str] = set()
    pattern = re.compile(r"\b((?:super\s*::\s*)+)\{")
    for match in pattern.finditer(source):
        levels = len(re.findall(r"super", match.group(1)))
        opening = source.find("{", match.start())
        close = matching_brace(source, opening)
        if close is None:
            continue
        base = path_parts[:-levels] if levels <= len(path_parts) else []
        if base:
            roots.add(base[0])
            continue
        body = source[opening + 1 : close]
        depth = 0
        start = 0
        items: list[str] = []
        for index, char in enumerate(body):
            if char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
            elif char == "," and depth == 0:
                items.append(body[start:index])
                start = index + 1
        items.append(body[start:])
        for item in items:
            root = re.search(rf"\b({IDENT})\b", item)
            if root and root.group(1) not in {"self", "super", "crate"}:
                roots.add(root.group(1))
    return roots


def referenced_roots(source: str, path_parts: list[str], is_lib: bool = False) -> tuple[set[str], set[str]]:
    internal = set(re.findall(rf"\bcrate\s*::\s*({IDENT})", source))
    internal.update(grouped_roots(source, "crate"))
    internal.update(grouped_super_roots(source, path_parts))

    super_pattern = re.compile(rf"\b((?:super\s*::\s*)+)({IDENT})")
    for match in super_pattern.finditer(source):
        if match.group(2) == "super":
            continue
        levels = len(re.findall(r"super", match.group(1)))
        base = path_parts[:-levels] if levels <= len(path_parts) else []
        target = base[0] if base else match.group(2)
        internal.add(target)

    if is_lib:
        internal.update(
            re.findall(rf"\b(?:pub\s+)?use\s+({IDENT})\s*::", source)
        )

    externals = set(
        re.findall(rf"\b(?:pub\s+)?use\s+({IDENT})\s*::", source)
    )
    externals.difference_update(internal)
    externals.difference_update({"crate", "self", "super", "std", "core", "alloc"})
    return internal, externals


def discover_modules(lib_source: str) -> list[str]:
    masked = mask_rust_literals_and_comments(lib_source)
    production, _ = partition_test_code(masked)
    return sorted(set(re.findall(rf"(?m)^\s*(?:pub\s+)?mod\s+({IDENT})\s*;", production)))


def load_slots(
    path: Path,
) -> tuple[
    dict[str, str],
    dict[str, set[str]],
    dict[str, set[str]],
    dict[str, set[str]],
]:
    data = json.loads(path.read_text(encoding="utf-8"))
    owners: dict[str, str] = {}
    allowed: dict[str, set[str]] = {}
    for slot, definition in data["slots"].items():
        allowed[slot] = set(definition["may_depend_on"])
        for module in definition["modules"]:
            if module in owners:
                raise ValueError(f"module {module!r} assigned to two slots")
            owners[module] = slot
    external_allowed = {
        dependency.replace("-", "_"): set(slots)
        for dependency, slots in data.get("external_boundaries", {}).items()
    }
    external_exceptions = {
        dependency.replace("-", "_"): set(modules)
        for dependency, modules in data.get("external_module_exceptions", {}).items()
    }
    return owners, allowed, external_allowed, external_exceptions


def cargo_dependencies(manifest: Path) -> set[str]:
    data = tomllib.loads(manifest.read_text(encoding="utf-8"))
    dependencies: set[str] = set()

    def visit(value: object, key: str | None = None) -> None:
        if not isinstance(value, dict):
            return
        if key in {"dependencies", "dev-dependencies", "build-dependencies"}:
            dependencies.update(name.replace("-", "_") for name in value)
            return
        for child_key, child in value.items():
            visit(child, child_key)

    visit(data)
    return dependencies


def uses_dependency(source: str, dependency: str) -> bool:
    return bool(
        re.search(
            rf"(?<![A-Za-z0-9_:]){re.escape(dependency)}\s*::",
            source,
        )
    )


def add_edge(edges: dict[tuple[str, str], Edge], source: str, target: str, location: str) -> None:
    if source == target:
        return
    edge = edges.setdefault((source, target), Edge(source, target))
    edge.locations.add(location)


def count_pattern(sources: Iterable[str], pattern: re.Pattern[str]) -> tuple[int, int]:
    count = 0
    touched = 0
    for source in sources:
        found = len(pattern.findall(source))
        count += found
        touched += int(found > 0)
    return count, touched


def run_census(
    root: Path, slots_path: Path
) -> tuple[
    Census,
    dict[str, set[str]],
    dict[str, set[str]],
    dict[str, set[str]],
]:
    src = root / "src"
    rust_files = sorted(src.rglob("*.rs"))
    lib = src / "lib.rs"
    modules = discover_modules(lib.read_text(encoding="utf-8"))
    owners, allowed, external_allowed, external_exceptions = load_slots(slots_path)
    dependencies = cargo_dependencies(root / "Cargo.toml")
    expected = set(modules) | {"lib"}
    missing = sorted(expected - set(owners))
    extra = sorted(set(owners) - expected)
    if missing or extra:
        raise ValueError(f"slot map mismatch; missing={missing}, extra={extra}")

    masked_files = {
        path: mask_rust_literals_and_comments(path.read_text(encoding="utf-8"))
        for path in rust_files
    }
    test_module_roots = test_only_module_roots(src, masked_files)
    production_files: dict[Path, str] = {}
    test_files: dict[Path, str] = {}
    for path in rust_files:
        parts = module_path(src, path)
        masked = masked_files[path]
        production, tests = partition_test_code(
            masked, belongs_to_test_only_module(parts, test_module_roots)
        )
        production_files[path] = production
        test_files[path] = tests

    production_edges: dict[tuple[str, str], Edge] = {}
    test_edges: dict[tuple[str, str], Edge] = {}
    external_users: dict[str, set[str]] = defaultdict(set)
    for path in rust_files:
        parts = module_path(src, path)
        source_module = parts[0] if parts else "lib"
        if source_module == "tests":
            source_module = "lib"
        production = production_files[path]
        tests = test_files[path]
        location = path.relative_to(root).as_posix()
        for code, edges, is_test in (
            (production, production_edges, False),
            (tests, test_edges, True),
        ):
            internal, _ = referenced_roots(code, parts, path == lib)
            for target in internal:
                if target in expected:
                    add_edge(edges, source_module, target, location)
            if not is_test:
                for dependency in dependencies:
                    if uses_dependency(code, dependency):
                        external_users[dependency].add(source_module)

    pub_total, pub_files = count_pattern(
        production_files.values(), re.compile(r"\bpub\s*\(\s*crate\s*\)")
    )
    cross_providers = {
        target
        for source, target in production_edges
        if owners[source] != owners[target]
    }
    upper_bound = 0
    for path in rust_files:
        parts = module_path(src, path)
        source_module = parts[0] if parts else "lib"
        if source_module == "tests":
            source_module = "lib"
        if source_module not in cross_providers:
            continue
        upper_bound += len(
            re.findall(r"\bpub\s*\(\s*crate\s*\)", production_files[path])
        )

    manifest_roots, _ = count_pattern(
        (path.read_text(encoding="utf-8") for path in rust_files),
        re.compile(r"\bCARGO_MANIFEST_DIR\b"),
    )
    filesystem_reads, _ = count_pattern(
        masked_files.values(),
        re.compile(r"(?:\bfs\s*::\s*(?:read|read_to_string|read_dir)|\bstd\s*::\s*fs\s*::\s*(?:read|read_to_string|read_dir))\s*\("),
    )
    allow_attributes, _ = count_pattern(
        masked_files.values(), re.compile(r"#\s*\[\s*allow\s*\(")
    )
    panic_calls, _ = count_pattern(
        production_files.values(), re.compile(r"\bpanic\s*!\s*\(")
    )
    expect_calls, _ = count_pattern(
        production_files.values(), re.compile(r"\.\s*expect\s*\(")
    )
    cross_slot_test_edges = sum(
        1 for source, target in test_edges if owners[source] != owners[target]
    )

    return Census(
        root=root,
        modules=modules,
        module_slots=owners,
        production_edges=production_edges,
        test_edges=test_edges,
        external_users=dict(external_users),
        pub_crate_total=pub_total,
        pub_crate_files=pub_files,
        cross_slot_pub_crate_upper_bound=upper_bound,
        cross_slot_test_edges=cross_slot_test_edges,
        manifest_roots=manifest_roots,
        filesystem_reads=filesystem_reads,
        allow_attributes=allow_attributes,
        panic_calls=panic_calls,
        expect_calls=expect_calls,
    ), allowed, external_allowed, external_exceptions


def forbidden_edges(census: Census, allowed: dict[str, set[str]]) -> list[Edge]:
    result = []
    for edge in census.production_edges.values():
        source_slot = census.module_slots[edge.source]
        target_slot = census.module_slots[edge.target]
        if source_slot != target_slot and target_slot not in allowed[source_slot]:
            result.append(edge)
    return sorted(result, key=lambda edge: (census.module_slots[edge.source], edge.source, edge.target))


def forbidden_external_edges(
    census: Census,
    external_allowed: dict[str, set[str]],
    external_exceptions: dict[str, set[str]],
) -> list[tuple[str, str, str]]:
    violations = []
    for dependency, allowed_slots in external_allowed.items():
        for module in census.external_users.get(dependency, set()):
            slot = census.module_slots[module]
            excepted_modules = external_exceptions.get(dependency, set())
            if slot not in allowed_slots and module not in excepted_modules:
                violations.append((module, dependency, slot))
    return sorted(violations)


def slot_edges(census: Census) -> set[tuple[str, str]]:
    return {
        (census.module_slots[source], census.module_slots[target])
        for source, target in census.production_edges
        if census.module_slots[source] != census.module_slots[target]
    }


def strongly_connected(nodes: Iterable[str], edges: set[tuple[str, str]]) -> list[list[str]]:
    adjacency: dict[str, list[str]] = defaultdict(list)
    for source, target in edges:
        adjacency[source].append(target)
    index = 0
    stack: list[str] = []
    indices: dict[str, int] = {}
    low: dict[str, int] = {}
    on_stack: set[str] = set()
    components: list[list[str]] = []

    def visit(node: str) -> None:
        nonlocal index
        indices[node] = index
        low[node] = index
        index += 1
        stack.append(node)
        on_stack.add(node)
        for target in adjacency[node]:
            if target not in indices:
                visit(target)
                low[node] = min(low[node], low[target])
            elif target in on_stack:
                low[node] = min(low[node], indices[target])
        if low[node] == indices[node]:
            component = []
            while True:
                member = stack.pop()
                on_stack.remove(member)
                component.append(member)
                if member == node:
                    break
            if len(component) > 1:
                components.append(sorted(component))

    for node in sorted(nodes):
        if node not in indices:
            visit(node)
    return sorted(components)


def as_json(
    census: Census,
    allowed: dict[str, set[str]],
    external_allowed: dict[str, set[str]],
    external_exceptions: dict[str, set[str]],
) -> str:
    forbidden = forbidden_edges(census, allowed)
    external_forbidden = forbidden_external_edges(
        census, external_allowed, external_exceptions
    )
    data = {
        "modules": len(census.modules),
        "production_module_edges": len(census.production_edges),
        "test_module_edges": len(census.test_edges),
        "slot_edges": sorted([list(edge) for edge in slot_edges(census)]),
        "slot_cycles": strongly_connected(set(census.module_slots.values()), slot_edges(census)),
        "forbidden_edges": [
            {
                "source": edge.source,
                "target": edge.target,
                "source_slot": census.module_slots[edge.source],
                "target_slot": census.module_slots[edge.target],
                "locations": sorted(edge.locations),
            }
            for edge in forbidden
        ],
        "external_boundary_violations": [
            {"module": module, "dependency": dependency, "slot": slot}
            for module, dependency, slot in external_forbidden
        ],
        "external_module_exceptions": {
            dependency: sorted(modules)
            for dependency, modules in sorted(external_exceptions.items())
        },
        "pub_crate": {
            "total": census.pub_crate_total,
            "files": census.pub_crate_files,
            "cross_slot_provider_upper_bound": census.cross_slot_pub_crate_upper_bound,
        },
        "cross_slot_test_edges": census.cross_slot_test_edges,
        "manifest_root_mentions": census.manifest_roots,
        "filesystem_reads": census.filesystem_reads,
        "allow_attributes": census.allow_attributes,
        "production_panic_calls": census.panic_calls,
        "production_expect_calls": census.expect_calls,
        "external_users": {
            dependency: sorted(users)
            for dependency, users in sorted(census.external_users.items())
        },
    }
    return json.dumps(data, indent=2) + "\n"


def as_markdown(
    census: Census,
    allowed: dict[str, set[str]],
    external_allowed: dict[str, set[str]],
    external_exceptions: dict[str, set[str]],
) -> str:
    forbidden = forbidden_edges(census, allowed)
    external_forbidden = forbidden_external_edges(
        census, external_allowed, external_exceptions
    )
    edges = slot_edges(census)
    cycles = strongly_connected(set(census.module_slots.values()), edges)
    lines = [
        "# One-Way Internals census",
        "",
        "This is a gauge over the provisional slot map, not a suite gate or a seam ruling.",
        "",
        "| Metric | Value |",
        "|---|---:|",
        f"| Top-level modules | {len(census.modules)} |",
        f"| Production module edges | {len(census.production_edges)} |",
        f"| Test-only module edges | {len(census.test_edges)} |",
        f"| Provisional slot edges | {len(edges)} |",
        f"| Provisional forbidden module edges | {len(forbidden)} |",
        f"| Provisional external-boundary violations | {len(external_forbidden)} |",
        f"| Provisional slot SCCs | {len(cycles)} |",
        f"| `pub(crate)` production declarations | {census.pub_crate_total} in {census.pub_crate_files} files |",
        f"| `pub(crate)` upper bound in cross-slot provider modules | {census.cross_slot_pub_crate_upper_bound} |",
        f"| Cross-slot test-only module edges | {census.cross_slot_test_edges} |",
        f"| `CARGO_MANIFEST_DIR` mentions | {census.manifest_roots} |",
        f"| Filesystem read calls | {census.filesystem_reads} |",
        f"| `#[allow(...)]` attributes | {census.allow_attributes} |",
        f"| Production `panic!` calls | {census.panic_calls} |",
        f"| Production `.expect(...)` calls | {census.expect_calls} |",
        "",
        "## Provisional forbidden edges",
        "",
        "| Source | Target | Slot direction | Receipts |",
        "|---|---|---|---|",
    ]
    for edge in forbidden:
        receipts = "<br>".join(f"`{location}`" for location in sorted(edge.locations))
        lines.append(
            f"| `{edge.source}` | `{edge.target}` | "
            f"`{census.module_slots[edge.source]}` -> `{census.module_slots[edge.target]}` | {receipts} |"
        )
    if not forbidden:
        lines.append("| — | — | — | — |")
    lines.extend(
        [
            "",
            "## Provisional external-boundary violations",
            "",
            "| Module | Dependency | Provisional slot | Allowed slots |",
            "|---|---|---|---|",
        ]
    )
    for module, dependency, slot in external_forbidden:
        allowed_slots = ", ".join(f"`{item}`" for item in sorted(external_allowed[dependency]))
        lines.append(f"| `{module}` | `{dependency}` | `{slot}` | {allowed_slots} |")
    if not external_forbidden:
        lines.append("| — | — | — | — |")
    lines.extend(["", "Accepted module-specific external exceptions:"])
    if external_exceptions:
        for dependency, modules in sorted(external_exceptions.items()):
            lines.append(
                f"- `{dependency}`: " + ", ".join(f"`{module}`" for module in sorted(modules))
            )
    else:
        lines.append("- None.")
    lines.extend(["", "## Provisional slot cycles", ""])
    if cycles:
        for component in cycles:
            lines.append("- " + " <-> ".join(f"`{slot}`" for slot in component))
    else:
        lines.append("None.")
    lines.extend(["", "## External dependency users", ""])
    for dependency, users in sorted(census.external_users.items()):
        lines.append(f"- `{dependency}`: " + ", ".join(f"`{user}`" for user in sorted(users)))
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path)
    parser.add_argument("--slots", type=Path)
    parser.add_argument("--format", choices=("markdown", "json"), default="markdown")
    args = parser.parse_args()
    root = (args.root or workspace_root()).resolve()
    slots = (args.slots or Path(__file__).with_name("one_way_slots.json")).resolve()
    try:
        census, allowed, external_allowed, external_exceptions = run_census(root, slots)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"one-way census failed: {error}", file=sys.stderr)
        return 2
    print(
        as_json(census, allowed, external_allowed, external_exceptions)
        if args.format == "json"
        else as_markdown(census, allowed, external_allowed, external_exceptions),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
