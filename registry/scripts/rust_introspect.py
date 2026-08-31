"""Best-effort regex introspection of adico's Dioxus/Rust component source.

Not a real Rust parser -- good enough to extract component function names,
their `#[derive(Props...)]` struct field names/types, and which `use_*`
hooks a file imports or defines, for compatibility-tracking scripts.
"""

from __future__ import annotations

import re
from pathlib import Path

COMPONENT_FN_RE = re.compile(r"^pub fn ([A-Z]\w*)(?:<[^>]*>)?\(", re.MULTILINE)
HOOK_FN_RE = re.compile(r"^pub fn (use_[a-z0-9_]+)", re.MULTILINE)
PROPS_STRUCT_RE = re.compile(
    r"pub struct (\w+Props)(?:<[^>{]*>)?\s*\{(.*?)\n\}", re.DOTALL
)
FIELD_RE = re.compile(r"^\s*pub (\w+):\s*([^,\n]+?),?\s*$", re.MULTILINE)
USE_IMPORT_RE = re.compile(r"use\s+(?:crate|dioxus[\w:]*)::[\w:]*\{?([^;]*)\}?;", re.DOTALL)


def read_source(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8")
    except OSError:
        return None


def find_components(source: str) -> list[str]:
    """Public component functions (`pub fn Foo(...)`), in source order,
    deduplicated."""
    seen: list[str] = []
    for match in COMPONENT_FN_RE.finditer(source):
        name = match.group(1)
        if name not in seen:
            seen.append(name)
    return seen


def find_hooks_defined(source: str) -> list[str]:
    """Public hook functions this file itself defines (`pub fn use_x`)."""
    seen: list[str] = []
    for match in HOOK_FN_RE.finditer(source):
        name = match.group(1)
        if name not in seen:
            seen.append(name)
    return seen


def find_hooks_used(source: str) -> list[str]:
    """`use_*` identifiers pulled in via `use crate::...` imports -- a
    heuristic for which shared primitives/hooks a component composes."""
    hooks: set[str] = set()
    for match in USE_IMPORT_RE.finditer(source):
        for identifier in re.findall(r"\buse_[a-z0-9_]+\b", match.group(1)):
            hooks.add(identifier)
    return sorted(hooks)


def find_props(source: str) -> dict[str, list[dict[str, str]]]:
    """Map of `FooProps` struct name -> ordered list of {"name", "type"}
    for each `pub` field."""
    result: dict[str, list[dict[str, str]]] = {}
    for struct_match in PROPS_STRUCT_RE.finditer(source):
        struct_name = struct_match.group(1)
        body = struct_match.group(2)
        fields = []
        for field_match in FIELD_RE.finditer(body):
            fields.append(
                {"name": field_match.group(1), "type": field_match.group(2).strip()}
            )
        result[struct_name] = fields
    return result


def _matching_paren(source: str, open_index: int) -> int:
    """Index of the `)` matching the `(` at `open_index`, tracking nested
    `()`/`<>` (generic type args also use commas/parens internally)."""
    depth = 0
    for index in range(open_index, len(source)):
        if source[index] in "(<":
            depth += 1
        elif source[index] in ")>":
            depth -= 1
            if depth == 0 and source[index] == ")":
                return index
    return -1


def _split_top_level(params_text: str) -> list[str]:
    parts: list[str] = []
    depth = 0
    current = []
    for char in params_text:
        if char in "(<[":
            depth += 1
        elif char in ")>]":
            depth -= 1
        if char == "," and depth == 0:
            parts.append("".join(current))
            current = []
        else:
            current.append(char)
    if "".join(current).strip():
        parts.append("".join(current))
    return parts


def find_inline_component_props(source: str) -> dict[str, list[dict[str, str]]]:
    """Components declared Dioxus-macro-style with plain fn parameters
    (`#[component] pub fn Foo(index: usize, children: Element) -> Element`)
    instead of an explicit `#[derive(Props)] struct FooProps`. Skips any
    component whose single parameter is `props: FooProps` -- that shape is
    already covered by `find_props`."""
    result: dict[str, list[dict[str, str]]] = {}
    for match in COMPONENT_FN_RE.finditer(source):
        name = match.group(1)
        open_paren = match.end() - 1
        close_paren = _matching_paren(source, open_paren)
        if close_paren == -1:
            continue
        params_text = source[open_paren + 1 : close_paren].strip()
        if not params_text or re.fullmatch(r"props:\s*\w+", params_text):
            continue
        fields = []
        for raw_part in _split_top_level(params_text):
            part = re.sub(r"#\[[^\]]*\]", "", raw_part).strip()
            if not part or ":" not in part:
                continue
            field_name, field_type = part.split(":", 1)
            fields.append({"name": field_name.strip(), "type": field_type.strip()})
        if fields:
            result[name] = fields
    return result


def introspect_file(path: Path) -> dict:
    """Full introspection summary for one Rust source file, or an empty
    summary (`exists: False`) if the file doesn't exist / can't be read."""
    source = read_source(path)
    if source is None:
        return {
            "exists": False,
            "components": [],
            "props": {},
            "hooks_defined": [],
            "hooks_used": [],
        }
    props = find_props(source)
    props.update(find_inline_component_props(source))
    return {
        "exists": True,
        "components": find_components(source),
        "props": props,
        "hooks_defined": find_hooks_defined(source),
        "hooks_used": find_hooks_used(source),
    }
