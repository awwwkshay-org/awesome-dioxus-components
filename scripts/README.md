# Repository scripts

Keep non-Cargo helper entry points here. Repeatable Rust automation belongs in
`packages/adico-xtask`.

Non-Cargo helpers scoped to one area of the repo live next to that area
instead of here, in their own `scripts/` folder:

- `packages/adico-primitives/scripts/baseui_compatibility_introspection.py`
  (`uv run ... sync|check|diff`) regenerates
  `packages/adico-primitives/baseui_compatibility.json`: which Base UI
  (base-ui.com) components/utils are built in `adico-primitives`, partial,
  or not started yet, with prop/hook detail introspected from the actual
  Rust source.
- `registry/scripts/shadcn_compatibility_introspection.py`
  (`uv run ... sync|check`) regenerates `registry/shadcn_compatibility.json`:
  hook/prop-level detail for every shadcn-mapped registry item, cross-
  referencing `parity.json`'s per-dimension pass/fail status against the
  registry facade's and underlying primitive's actual current props.

Both are self-contained `uv` scripts (PEP 723 inline metadata, stdlib only)
and both carry a `synced_at` date — re-run them after primitive/registry
changes rather than hand-editing the generated JSON.
