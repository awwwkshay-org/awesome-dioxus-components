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

For registry generation, upstream catalog refresh, provenance checks, and
parity reporting, use the future `adico-xtask` commands defined by the active
OpenSpec change. Normal CI must use checked-in snapshots and not require live
network access.

Every `registry:ui`/`registry:component` item also carries two offline,
CI-gated classification records: `cargo run -p adico-xtask -- primitive-usage
sync|check|diff` verifies its declared behavior-ownership classification
(`statics/primitive_usage/<item>.json`) against `registry/ui/*.rs` and
`registry.json`, and `cargo run -p adico-xtask -- styling-usage
sync|check|diff` verifies its Tailwind-only and semantic-token classification
(`statics/styling_usage/<item>.json`) the same way. Run `sync` after adding or
changing a registry item's source, hand-review any new/changed record, and run
`check` before committing.

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
