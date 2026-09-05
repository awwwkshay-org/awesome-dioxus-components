# Development

Use the pinned Rust toolchain and committed lockfile.

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
openspec validate build-adico-component-ecosystem --strict
```

The documentation, playground, and examples are intentionally Dioxus projects.
Their default web configurations keep the basic workspace checks portable.
Run feature-specific Dioxus, browser, SSR/hydration, and desktop checks only
when the affected milestone provides the required fixtures; record skipped
checks with their reason.

Do not make consumer examples import `registry/` source using workspace paths.
The installation fixtures must invoke the locally built `adico` executable once
the CLI vertical slice is implemented.

## Maintainer workflows: registry, upstream catalogs, provenance, compat

Normal CI runs entirely offline against the snapshots and generated files
already checked into the repository (`registry/generated/`,
`statics/catalogs/*.json`, `statics/primitive_compatibility.json`,
`statics/component_compatibility.json`, `statics/primitive_usage/*.json`,
`statics/styling_usage/*.json`) -- none of the commands below need network
access except the one explicitly marked otherwise, and CI never runs that one.

- **Registry generation** -- after adding or editing a `registry/ui/*.rs`
  item or its `registry/registry.json` entry: `cargo run -p adico-xtask --
  registry build` regenerates `registry/generated/*` from source; `cargo run
  -p adico-xtask -- registry validate` (CI-gated) fails if it's stale.
- **Upstream catalog refresh** (the only network-touching command in this
  list; run only on explicit maintainer request, never in CI): `cargo run -p
  adico-xtask -- catalog fetch <shadcn|base-ui|dioxus-components|dioxus-primitives|all>
  [--revision <sha>]` writes a revision-pinned snapshot to
  `statics/catalogs/<axis>.json`. This replaced the older, now-removed
  `cargo xtask upstream dioxus-components`/`upstreams/` directory (see
  `openspec/changes/build-adico-component-ecosystem/design.md` §9's own
  "Replaced 2026-08-31 (catalog-fetch-tooling)" record) -- there is no
  separate "inventory refresh" command any more, `catalog fetch
  dioxus-components` is it.
- **Compat/parity reporting against those snapshots** (offline, CI-gated):
  `cargo run -p adico-xtask -- primitive-compat sync|check|diff` (Base UI +
  dioxus-primitives axes) and `cargo run -p adico-xtask -- component-compat
  sync|check` (shadcn + dioxus-components axes) regenerate/verify
  `statics/primitive_compatibility.json`/`statics/component_compatibility.json`
  from the checked-in catalog snapshots and live Rust source introspection.
  There is no `cargo xtask parity` command and no `parity.json` file --
  both were deliberately removed 2026-08-31 per explicit user instruction in
  favor of these current-state snapshots rather than a hand-maintained,
  multi-dimension completion ledger (design.md §9's own removal record).
  Run `sync` after a registry/primitive change that could shift built/not-
  started counts, hand-review the diff, and run `check` before committing.
- **Provenance checks** (offline, CI-gated): `cargo run -p adico-xtask --
  provenance check` verifies `provenance/records/*` against `UPSTREAMS.md`'s
  obligations.
- **Company/organization registry validation**: a configured organization
  registry (a named local-path or static-HTTPS source registered in a
  consumer's `components.json`) is validated the same way the official
  registry's consumers are -- through a real `tests/installation/*`
  fixture that runs `adico add` against it and then `cargo check`/`cargo
  test`. `tests/installation/awwwkshay-consumer` is this repo's own such
  fixture; add a new one the same way when testing a new organization
  registry's source or its default-switching behavior.

Every `registry:ui`/`registry:component` item also carries two offline,
CI-gated classification records: `cargo run -p adico-xtask -- primitive-usage
sync|check|diff` verifies its declared behavior-ownership classification
(`statics/primitive_usage/<item>.json`) against `registry/ui/*.rs` and
`registry.json`, and `cargo run -p adico-xtask -- styling-usage
sync|check|diff` verifies its Tailwind-only and semantic-token classification
(`statics/styling_usage/<item>.json`) the same way. Run `sync` after adding or
changing a registry item's source, hand-review any new/changed record, and run
`check` before committing.

Every `registry:ui`/`registry:component` item also carries an offline,
CI-gated prop-parity record: `cargo run -p adico-xtask -- prop-parity
sync|check|diff` joins the item's own declared props against each catalog
axis's (`base-ui`, `dioxus-components`, `dioxus-primitives`, `shadcn`)
resolved upstream props and classifies every upstream prop as `present`,
`missing`, `intentional_difference`, or `adico_extension`
(`statics/prop_parity/<item>.json`). Unlike `primitive-usage`/
`styling-usage`, this record is 100% derived from source on every run --
`intentional_difference`/`adico_extension` reasons live in fixed tables in
`packages/adico-xtask/src/prop_parity.rs`, never hand-edited into the
generated JSON, so `sync` never preserves stale prose and `check` is a
plain regenerate-and-compare. Run `sync` after a registry/primitive prop
change or a `catalog fetch` refresh, and run `check` before committing.

The playground's demo-page enum controls are generated, not hand-typed:
`cargo run -p adico-xtask -- playground-controls sync|check|diff` introspects
`apps/playground/src/components/ui/*.rs` for enum-typed props with a
`#[default]` variant and writes one file per component under
`apps/playground/src/generated/controls/`, each a `pub const
<ENUM>_OPTIONS: &[(&str, <Enum>)]` plus a compile-time exhaustiveness guard
over its source enum — so an added/removed/renamed variant that isn't
regenerated fails `cargo check --locked --workspace`, not just this
command's own `check`. Run `sync` after adding or changing a playground UI
component's enum-typed prop, and run `check` before committing.
