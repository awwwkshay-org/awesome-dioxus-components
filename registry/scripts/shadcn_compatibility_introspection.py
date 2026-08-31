#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Sync `registry/shadcn_compatibility.json`: a structured, hooks-and-props
-level view of every shadcn-mapped registry item, generated from this
repo's own current state rather than hand-audited prose (that's what
`parity.json`'s per-dimension notes already are -- this file complements
it, not replaces it).

For every component tracked in `parity.json`'s `components` map, this
introspects:
  - the registry facade file(s) named in `registry/registry.json` (the
    styled, source-installed component under `registry/ui/`), and
  - the underlying `adico-primitives` module it composes (found via the
    registry item's `moduleExports`/import statements, not guessed from
    naming),
extracting each one's public component functions, `#[derive(Props...)]`
struct fields, and which shared `use_*` hooks the primitive layer uses.
It also carries over parity.json's own `status` and per-dimension
pass/fail summary, so this file answers "what props/hooks exist today"
while parity.json keeps answering "why a dimension doesn't pass yet."

Commands:
    sync    Regenerate shadcn_compatibility.json from current repo state (default).
    check   Exit 1 if the on-disk JSON would change without writing it -- for CI drift checks.

Usage:
    uv run registry/scripts/shadcn_compatibility_introspection.py sync
    uv run registry/scripts/shadcn_compatibility_introspection.py check
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import date
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from rust_introspect import introspect_file  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
REGISTRY_ROOT = REPO_ROOT / "registry"
PRIMITIVES_SRC = REPO_ROOT / "packages" / "adico-primitives" / "src"
PARITY_PATH = REPO_ROOT / "parity.json"
REGISTRY_MANIFEST_PATH = REGISTRY_ROOT / "registry.json"
OUTPUT_PATH = REGISTRY_ROOT / "shadcn_compatibility.json"

PRIMITIVE_MODULE_IMPORT_RE = re.compile(r"use\s+adico_primitives::(\w+)")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def dimension_summary(dimensions: dict) -> dict:
    applicable = {name: dim for name, dim in dimensions.items() if dim.get("applicable")}
    passed = {name: dim for name, dim in applicable.items() if dim.get("passed")}
    return {
        "applicable_dimensions": len(applicable),
        "passed_dimensions": len(passed),
        "failing_dimensions": sorted(name for name in applicable if name not in passed),
    }


def find_registry_manifest_item(manifest_items: list[dict], name: str) -> dict | None:
    for item in manifest_items:
        if item["name"] == name:
            return item
    return None


def introspect_registry_files(item: dict) -> dict:
    """Introspect every `registry/ui/*.rs` file this registry item installs."""
    components: list[str] = []
    props: dict[str, list[dict[str, str]]] = {}
    for file_entry in item.get("files", []):
        source_path = REGISTRY_ROOT / file_entry["source"]
        result = introspect_file(source_path)
        components.extend(result["components"])
        props.update(result["props"])
    return {"components": components, "props": props}


def find_primitive_modules(item: dict) -> list[str]:
    """The `adico_primitives::<module>` modules this registry item's facade
    files actually import, per `moduleExports` first (authoritative) and
    falling back to scanning `use adico_primitives::` lines."""
    modules = {export["module"] for export in item.get("moduleExports", [])}
    for file_entry in item.get("files", []):
        source_path = REGISTRY_ROOT / file_entry["source"]
        source = source_path.read_text(encoding="utf-8") if source_path.exists() else ""
        modules.update(PRIMITIVE_MODULE_IMPORT_RE.findall(source))
    return sorted(modules)


def introspect_primitive_modules(module_names: list[str]) -> dict:
    """Introspect each candidate `packages/adico-primitives/src/<module>.rs`
    (or `<module>/` directory) primitive module."""
    components: list[str] = []
    props: dict[str, list[dict[str, str]]] = {}
    hooks_used: set[str] = set()
    files: list[str] = []
    for module in module_names:
        single_file = PRIMITIVES_SRC / f"{module}.rs"
        module_dir = PRIMITIVES_SRC / module
        rust_files: list[Path] = []
        if single_file.exists():
            rust_files = [single_file]
        elif module_dir.is_dir():
            rust_files = sorted(module_dir.rglob("*.rs"))
        for rust_file in rust_files:
            result = introspect_file(rust_file)
            components.extend(result["components"])
            props.update(result["props"])
            hooks_used.update(result["hooks_used"])
            hooks_used.update(result["hooks_defined"])
            files.append(str(rust_file.relative_to(REPO_ROOT)))
    return {
        "files": files,
        "components": components,
        "props": props,
        "hooks": sorted(hooks_used),
    }


def build_document() -> dict:
    parity = load_json(PARITY_PATH)
    manifest = load_json(REGISTRY_MANIFEST_PATH)
    manifest_items = manifest["items"]

    entries = []
    for name, parity_entry in sorted(parity["components"].items()):
        manifest_item = find_registry_manifest_item(manifest_items, name)
        entry: dict = {
            "name": name,
            "registry_item": parity_entry.get("registryItem"),
            "parity_status": parity_entry.get("status"),
            "parity_dimensions": dimension_summary(parity_entry.get("dimensions", {})),
        }
        if manifest_item is None:
            entry["registry_manifest_found"] = False
            entries.append(entry)
            continue

        entry["registry_manifest_found"] = True
        registry_introspection = introspect_registry_files(manifest_item)
        entry["registry_facade"] = {
            "files": [f["source"] for f in manifest_item.get("files", [])],
            "components": registry_introspection["components"],
            "props": registry_introspection["props"],
        }

        primitive_modules = find_primitive_modules(manifest_item)
        primitive_introspection = introspect_primitive_modules(primitive_modules)
        entry["adico_primitive"] = {
            "modules": primitive_modules,
            "files": primitive_introspection["files"],
            "components": primitive_introspection["components"],
            "props": primitive_introspection["props"],
            "hooks_used": primitive_introspection["hooks"],
        }
        entries.append(entry)

    return {
        "$schema_note": "Fully generated by shadcn_compatibility_introspection.py from parity.json + registry.json + live Rust source. Edit parity.json's per-dimension notes for prose audit findings; this file only reflects current prop/hook shape.",
        "synced_at": date.today().isoformat(),
        "generator": "registry/scripts/shadcn_compatibility_introspection.py",
        "summary": {
            "total_components": len(entries),
            "passing_all_applicable_dimensions": sum(
                1
                for e in entries
                if e.get("parity_dimensions", {}).get("failing_dimensions") == []
            ),
        },
        "components": entries,
    }


def cmd_sync(_args: argparse.Namespace) -> int:
    document = build_document()
    OUTPUT_PATH.write_text(json.dumps(document, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"Wrote {OUTPUT_PATH.relative_to(REPO_ROOT)}")
    print(f"  components: {document['summary']['total_components']}")
    print(f"  passing all applicable parity dimensions: {document['summary']['passing_all_applicable_dimensions']}")
    return 0


def cmd_check(_args: argparse.Namespace) -> int:
    document = build_document()
    existing_raw = OUTPUT_PATH.read_text(encoding="utf-8") if OUTPUT_PATH.exists() else ""
    existing = json.loads(existing_raw) if existing_raw else {}
    existing.pop("synced_at", None)
    comparable = dict(document)
    comparable.pop("synced_at", None)
    if existing != comparable:
        print("shadcn_compatibility.json is stale; run the `sync` command to regenerate.", file=sys.stderr)
        return 1
    print("shadcn_compatibility.json is up to date.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    subparsers = parser.add_subparsers(dest="command")
    subparsers.add_parser("sync", help="regenerate shadcn_compatibility.json (default)").set_defaults(func=cmd_sync)
    subparsers.add_parser("check", help="exit 1 if the on-disk JSON is stale, without writing").set_defaults(func=cmd_check)

    args = parser.parse_args()
    if not getattr(args, "func", None):
        args.func = cmd_sync
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
