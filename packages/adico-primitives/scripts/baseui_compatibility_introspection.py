#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Sync `packages/adico-primitives/baseui_compatibility.json` against
Base UI's (https://base-ui.com/react/components) component/util inventory
and this crate's actual current source.

What this script automates:
  - Introspects each mapped adico primitive file (component functions, Props
    struct fields, hooks used/defined) via `rust_introspect.py`.
  - Attempts a live fetch of the Base UI components page to flag components
    added/removed upstream since the tracked list below was last reviewed
    (best-effort; the mapped upstream list itself is not derived from the
    fetch, since deciding "is this built in adico" needs human/AI judgment,
    not just a page scrape).
  - Recomputes summary counts and stamps `synced_at` with today's date.

What it does NOT automate (edit `UPSTREAM_COMPONENTS`/`UPSTREAM_UTILS`
below by hand instead):
  - Classifying a component's `status` (built / partial / not_started /
    not_applicable) and which adico file/registry item it maps to.
  - Free-text `notes`.

Commands:
    sync    Regenerate baseui_compatibility.json from current repo state (default).
    check   Exit 1 if the on-disk JSON would change without writing it -- for CI drift checks.
    diff    Print only the upstream drift check (components added/removed on
            base-ui.com since the tracked list was last reviewed); no write.

Usage:
    uv run packages/adico-primitives/scripts/baseui_compatibility_introspection.py sync
    uv run packages/adico-primitives/scripts/baseui_compatibility_introspection.py check
    uv run packages/adico-primitives/scripts/baseui_compatibility_introspection.py diff
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from datetime import date, datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from rust_introspect import introspect_file  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]
PRIMITIVES_SRC = REPO_ROOT / "packages" / "adico-primitives" / "src"
OUTPUT_PATH = REPO_ROOT / "packages" / "adico-primitives" / "baseui_compatibility.json"
BASEUI_COMPONENTS_URL = "https://base-ui.com/react/components"

# name -> (status, adico_file_or_None, adico_registry_item_or_None, notes)
# status: "built" | "partial" | "not_started" | "not_applicable"
UPSTREAM_COMPONENTS: dict[str, tuple[str, str | None, str | None, str]] = {
    "Accordion": ("built", "accordion.rs", "accordion", "Split into Accordion (single)/AccordionMulti, matching this crate's Select/SelectMulti convention rather than a type:'single'|'multiple' prop (task 7.8b)."),
    "Alert Dialog": ("built", "alert_dialog.rs", "alert-dialog", ""),
    "Autocomplete": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "Avatar": ("built", "avatar.rs", "avatar", ""),
    "Button": ("built", None, "button", "Native <button> semantics; no dedicated primitive needed, matching Base UI's own Button."),
    "Checkbox": ("built", "checkbox.rs", "checkbox", ""),
    "Checkbox Group": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "Collapsible": ("built", "collapsible.rs", "collapsible", ""),
    "Combobox": ("built", "combobox/", "combobox", "Multi-file module (combobox/components/*), not a single .rs file."),
    "Context Menu": ("partial", "context_menu.rs", "context-menu", "Flat, independently-implemented menu; not yet migrated onto the unified menu.rs primitive (task 7.8e)."),
    "Dialog": ("built", "dialog.rs", "dialog", ""),
    "Drawer": ("built", None, "sheet", "adico's 'Sheet' registry item is the shadcn/Base-UI Drawer equivalent; no dedicated primitive file (composes dialog.rs internals)."),
    "Field": ("not_started", None, None, "Base-UI-parity tier target, task 7.9 (field semantics)."),
    "Fieldset": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "Form": ("not_started", None, None, "Base-UI-parity tier target, task 7.9. Note: shadcn's own Form was found unportable (wave5-extras) since it has no real component beyond a native <form>; Base UI's Form may differ and needs its own read before starting."),
    "Input": ("built", None, "input", "Native <input> semantics; no dedicated primitive needed."),
    "Menu": ("partial", "menu.rs", None, "Unified Root/Trigger/Content/Item/CheckboxItem/RadioGroup/RadioItem/SubmenuRoot anatomy built (task 7.6a), but not yet composed on positioner::Positioner, not yet wired to use_typeahead, and not yet consumed by any registry item (task 7.8e migrates context-menu/dropdown-menu/menubar onto it)."),
    "Menubar": ("partial", "menubar.rs", "menubar", "Flat, independently-implemented; not yet migrated onto menu.rs (task 7.8e)."),
    "Meter": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "Navigation Menu": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "Number Field": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "OTP Field": ("not_started", None, None, "Base-UI-parity tier target, task 7.9."),
    "Popover": ("built", "popover.rs", "popover", "Now composes positioner::Positioner for real collision-aware placement (task 7.8c); sideOffset/Arrow/sub-component prop gaps vs shadcn remain (parity.json)."),
    "Preview Card": ("not_started", None, None, "Base-UI-parity tier target, task 7.9 -- the original inspiration for this crate's shared-primitive redesign (design.md)."),
    "Progress": ("built", "progress.rs", "progress", ""),
    "Radio": ("built", "radio_group.rs", "radio-group", "adico names it RadioGroup/RadioItem, matching Radix's own naming more than Base UI's bare 'Radio'."),
    "Scroll Area": ("built", "scroll_area.rs", "scroll-area", "Native-overflow/CSS toggle, not a custom-styled scrollbar-thumb sub-component (parity.json gap vs shadcn)."),
    "Select": ("built", "select/", "select", "Multi-file module (select/components/*, select/context.rs, select/mod.rs)."),
    "Separator": ("built", "separator.rs", "separator", ""),
    "Slider": ("built", "slider.rs", "slider", "Pointer-drag on web is suspected non-functional (task 7.7 finding, unverified without a browser); keyboard control is tested and works."),
    "Switch": ("built", "switch.rs", "switch", ""),
    "Tabs": ("built", "tabs.rs", "tabs", ""),
    "Toast": ("built", "toast.rs", "toast", "Its F6 focus-region shortcut uses the same long-lived document::eval listener pattern confirmed broken elsewhere (task 7.4d finding); likely non-functional on web, unverified."),
    "Toggle": ("built", "toggle.rs", "toggle", ""),
    "Toggle Group": ("built", "toggle_group.rs", "toggle-group", ""),
    "Toolbar": ("built", "toolbar.rs", "toolbar", ""),
    "Tooltip": ("built", "tooltip.rs", "tooltip", "Now composes positioner::Positioner for real collision-aware placement (task 7.8c); TooltipProvider/sideOffset/Arrow gaps vs shadcn remain (parity.json)."),
}

UPSTREAM_UTILS: dict[str, tuple[str, str | None, str]] = {
    "CSP Provider": ("not_applicable", None, "React inline-style injection concern; no Dioxus/Tailwind equivalent (design.md §8a)."),
    "Direction Provider": ("built", "direction.rs", "Direction/DirectionProvider/use_direction (task 7.3a)."),
    "mergeProps": ("not_applicable", None, "Maps to #[props(extends = GlobalAttributes)] + Element composition, already this crate's established pattern (design.md §8a)."),
    "useRender": ("not_applicable", None, "Same mapping as mergeProps."),
}

# Primitives/registry items adico has that Base UI has no equivalent for.
ADICO_ONLY_EXTRAS: dict[str, str] = {
    "DatePicker": "date_picker.rs",
    "ColorPicker": "color_picker.rs",
    "DragAndDropList": "drag_and_drop_list.rs",
    "TagGroup": "tag_group.rs",
    "HoverCard": "hover_card.rs",
    "VirtualList": "virtual_list.rs",
    "Calendar": "calendar.rs",
    "AspectRatio": "aspect_ratio.rs",
    "Label": "label.rs",
    "ThemeMode": "theme_mode.rs",
}


def fetch_upstream_component_names() -> list[str] | None:
    """Best-effort live check of Base UI's own component list, to flag
    upstream drift. Returns None (not a failure) if unreachable -- this
    script must still work fully offline."""
    try:
        request = urllib.request.Request(
            BASEUI_COMPONENTS_URL, headers={"User-Agent": "adico-sync-script/1.0"}
        )
        with urllib.request.urlopen(request, timeout=10) as response:
            html = response.read().decode("utf-8", errors="replace")
    except (urllib.error.URLError, TimeoutError, OSError):
        return None

    import re

    # The nav renders each component as a link like /react/components/accordion.
    slugs = sorted(set(re.findall(r"/react/components/([a-z0-9-]+)", html)))
    # Next.js embeds internal page-cache-id links matching this same URL
    # shape (e.g. "page-694493450857fab0"); those aren't real component
    # slugs and must be filtered out, not treated as upstream additions.
    slugs = [slug for slug in slugs if not re.fullmatch(r"page-[0-9a-f]+", slug)]
    return [slug.replace("-", " ").title() for slug in slugs]


def build_component_entry(name: str, status: str, file_name: str | None, registry_item: str | None, notes: str) -> dict:
    entry = {
        "name": name,
        "status": status,
        "adico_primitive_file": None,
        "adico_registry_item": registry_item,
        "notes": notes,
    }
    if file_name:
        path = PRIMITIVES_SRC / file_name
        entry["adico_primitive_file"] = f"packages/adico-primitives/src/{file_name}"
        if file_name.endswith("/"):
            # Multi-file module: introspect every .rs file under it.
            introspection = {"exists": path.is_dir(), "components": [], "props": {}, "hooks_defined": [], "hooks_used": []}
            if path.is_dir():
                for rust_file in sorted(path.rglob("*.rs")):
                    sub = introspect_file(rust_file)
                    introspection["components"].extend(sub["components"])
                    introspection["props"].update(sub["props"])
                    introspection["hooks_defined"].extend(sub["hooks_defined"])
                    introspection["hooks_used"].extend(sub["hooks_used"])
                introspection["hooks_defined"] = sorted(set(introspection["hooks_defined"]))
                introspection["hooks_used"] = sorted(set(introspection["hooks_used"]))
        else:
            introspection = introspect_file(path)
        entry["adico_components"] = introspection["components"]
        entry["adico_props"] = introspection["props"]
        entry["adico_hooks_defined"] = introspection["hooks_defined"]
        entry["adico_hooks_used"] = introspection["hooks_used"]
    return entry


def build_util_entry(name: str, status: str, file_name: str | None, notes: str) -> dict:
    entry = {"name": name, "status": status, "adico_primitive_file": None, "notes": notes}
    if file_name:
        path = PRIMITIVES_SRC / file_name
        entry["adico_primitive_file"] = f"packages/adico-primitives/src/{file_name}"
        introspection = introspect_file(path)
        entry["adico_hooks_defined"] = introspection["hooks_defined"]
    return entry


def build_document(*, check_upstream: bool) -> tuple[dict, dict | None]:
    components = [
        build_component_entry(name, status, file_name, registry_item, notes)
        for name, (status, file_name, registry_item, notes) in UPSTREAM_COMPONENTS.items()
    ]
    utils = [
        build_util_entry(name, status, file_name, notes)
        for name, (status, file_name, notes) in UPSTREAM_UTILS.items()
    ]
    extras = [
        {"name": name, "adico_primitive_file": f"packages/adico-primitives/src/{file_name}"}
        for name, file_name in ADICO_ONLY_EXTRAS.items()
    ]

    counts: dict[str, int] = {}
    for entry in components:
        counts[entry["status"]] = counts.get(entry["status"], 0) + 1

    upstream_drift = None
    if check_upstream:
        upstream_live = fetch_upstream_component_names()
        if upstream_live is not None:
            tracked = {name.lower() for name in UPSTREAM_COMPONENTS}
            live = {name.lower() for name in upstream_live}
            added = sorted(name for name in upstream_live if name.lower() not in tracked)
            removed = sorted(name for name in UPSTREAM_COMPONENTS if name.lower() not in live)
            upstream_drift = {
                "checked_at": datetime.now(timezone.utc).isoformat(),
                "added_upstream": added,
                "removed_upstream": removed,
            }

    document = {
        "$schema_note": "Hand-maintain UPSTREAM_COMPONENTS/UPSTREAM_UTILS/ADICO_ONLY_EXTRAS in baseui_compatibility_introspection.py; everything else here is regenerated from repo introspection.",
        "source": BASEUI_COMPONENTS_URL,
        "synced_at": date.today().isoformat(),
        "generator": "packages/adico-primitives/scripts/baseui_compatibility_introspection.py",
        "summary": {
            "total_upstream_components": len(components),
            "total_upstream_utils": len(utils),
            "adico_only_extras": len(extras),
            **{f"components_{status}": count for status, count in sorted(counts.items())},
        },
        "upstream_drift_check": upstream_drift,
        "components": components,
        "utils": utils,
        "adico_only_extras": extras,
    }
    return document, counts


def cmd_sync(_args: argparse.Namespace) -> int:
    document, counts = build_document(check_upstream=True)
    OUTPUT_PATH.write_text(json.dumps(document, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(f"Wrote {OUTPUT_PATH.relative_to(REPO_ROOT)}")
    print(f"  components: {len(document['components'])} tracked ({counts})")
    drift = document["upstream_drift_check"]
    if drift is None:
        print("  upstream drift check skipped (network unreachable)")
    elif drift["added_upstream"] or drift["removed_upstream"]:
        print(f"  upstream drift detected: added={drift['added_upstream']} removed={drift['removed_upstream']}")
    else:
        print("  upstream drift check: no changes detected")
    return 0


def cmd_check(_args: argparse.Namespace) -> int:
    document, _ = build_document(check_upstream=False)
    existing_raw = OUTPUT_PATH.read_text(encoding="utf-8") if OUTPUT_PATH.exists() else ""
    existing = json.loads(existing_raw) if existing_raw else {}
    existing.pop("synced_at", None)
    existing.pop("upstream_drift_check", None)
    comparable = dict(document)
    comparable.pop("synced_at", None)
    comparable.pop("upstream_drift_check", None)
    if existing != comparable:
        print("baseui_compatibility.json is stale; run the `sync` command to regenerate.", file=sys.stderr)
        return 1
    print("baseui_compatibility.json is up to date.")
    return 0


def cmd_diff(_args: argparse.Namespace) -> int:
    upstream_live = fetch_upstream_component_names()
    if upstream_live is None:
        print("Could not reach base-ui.com; drift check skipped.", file=sys.stderr)
        return 1
    tracked = {name.lower() for name in UPSTREAM_COMPONENTS}
    live = {name.lower() for name in upstream_live}
    added = sorted(name for name in upstream_live if name.lower() not in tracked)
    removed = sorted(name for name in UPSTREAM_COMPONENTS if name.lower() not in live)
    if not added and not removed:
        print("No upstream drift: tracked list matches base-ui.com.")
        return 0
    if added:
        print(f"Added upstream (not yet tracked): {added}")
    if removed:
        print(f"Removed upstream (still tracked): {removed}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    subparsers = parser.add_subparsers(dest="command")
    subparsers.add_parser("sync", help="regenerate baseui_compatibility.json (default)").set_defaults(func=cmd_sync)
    subparsers.add_parser("check", help="exit 1 if the on-disk JSON is stale, without writing").set_defaults(func=cmd_check)
    subparsers.add_parser("diff", help="print only the upstream drift check; no write").set_defaults(func=cmd_diff)

    args = parser.parse_args()
    if not getattr(args, "func", None):
        args.func = cmd_sync
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
