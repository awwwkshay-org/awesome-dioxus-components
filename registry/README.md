# Adico registry source

This directory is the authored source distribution layer for adico. It is not a
styled Rust component crate.

`registry.json` is the canonical authored manifest. Component files are kept in
their ordinary source form beneath `ui/`, `hooks/`, `lib/`, or `blocks/` and
their metadata is added to that manifest. This keeps a component reviewable as
source rather than a string embedded in installer code.

Run `cargo xtask registry build` to create the deterministic, checked-in local
distribution output under `generated/`:

- `generated/index.json` maps stable item names to payload paths.
- `generated/items/<name>.json` is the normalized metadata payload for one
  item.

The CLI will embed/read generated output; it never treats the generated index as
the authored source of truth. Organization registries use the same manifest and
payload contract.
