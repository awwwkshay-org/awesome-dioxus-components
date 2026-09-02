## Why

Every playground demo page hand-types its control panel, including the
option list for each enum-valued prop (e.g. `vec![("Destructive",
ButtonVariant::Destructive), ...]`). These lists already drift from the real
registry enums with nothing to catch it: `pages/badge.rs` offers 5 of
`BadgeVariant`'s 7 variants (`Ghost`, `Link` are unreachable in the
playground), and `pages/item.rs` offers 3 of `ItemVariant`'s 4 (`Outline` is
missing). A newly added registry variant is invisible in the playground until
someone notices and hand-edits every page that demos it.

## What Changes

- Extend `packages/adico-xtask/src/rust_introspect.rs` with an `Item::Enum`
  arm: extract each public enum's variant identifiers, which variant carries
  `#[default]`, and each variant's doc comment (falling back to the
  identifier) as its control label.
- Add a new `cargo run -p adico-xtask -- playground-controls sync|check|diff`
  command, following the existing `primitive-usage`/`styling-usage`
  `sync|check|diff` idiom. It introspects
  `apps/playground/src/components/ui/*.rs` (the playground's own installed,
  compiled copy of the registry — not `registry/ui/` directly) and generates
  Rust option constants for every enum-typed prop, committed under
  `apps/playground/src/generated/controls/`.
- Each generated option constant ships with a compile-time exhaustiveness
  guard over its source enum, so an added/removed/renamed variant that isn't
  regenerated fails `cargo check --locked --workspace`, not just a CI diff.
- Prop-shape-to-control mapping is an explicit allowlist recorded per prop
  with a reason when skipped (`bool`/`Option<bool>` -> `BoolControl`,
  `String` -> `TextControl`, enum-with-`#[default]` -> `SelectControl` +
  generated constant; `children`, `attributes`, `class`,
  `EventHandler<_>`-typed, and `Signal`/`ReadSignal`-typed props are skipped
  with a reason; numeric props are out of scope, no numeric control exists
  today).
- The 28 playground pages that already hand-write controls are updated to
  consume the generated option constants in place of their hand-typed
  `vec![...]` lists, closing the two confirmed drift cases as part of this
  change. Wiring controls into the other 25 currently-uncontrolled pages is
  out of scope, left as mechanical follow-up.
- **Non-goal**: no `macro_rules!`/proc-macro layer to further shrink the
  per-page `use_signal` + control-wiring boilerplate, and no runtime/JSON-
  driven control renderer — Dioxus component calls are statically typed, so
  the live demoed component's props cannot be wired from a runtime value
  without codegen disproportionate to the value here. Per-page signal
  declarations and control wiring stay hand-written; only the enum option
  lists become generated.

## Capabilities

### New Capabilities
- `adico-playground-demo-controls`: playground demo pages consume
  compile-time-verified, generated option constants for every enum-valued
  registry prop instead of hand-typed lists, with a mechanically-gated
  `sync|check|diff` workflow preventing drift between a registry component's
  real variants and what the playground offers.

### Modified Capabilities
(none — `adico-playground-structure`'s routing/page-file requirements are
unaffected; this change only affects what a page's `controls:` block is
built from)

## Impact

- `packages/adico-xtask/src/rust_introspect.rs`: new `Item::Enum` extraction
  logic and its unit tests.
- `packages/adico-xtask/src/main.rs` and a new
  `packages/adico-xtask/src/playground_controls.rs`: new `playground-controls
  sync|check|diff` subcommand.
- `apps/playground/src/generated/controls/` (new, generated, committed):
  one file per playground UI item with an enum-typed prop.
- `apps/playground/src/pages/*.rs` (the 28 pages with existing controls):
  hand-typed option `vec![...]` literals replaced with generated constants.
- `docs/development.md` / `docs/validation.md`: document the new xtask
  command and add it to the validation matrix alongside `primitive-usage
  check` and `styling-usage check`.
- No consumer-facing or registry-metadata change: `registry.json` and
  `registry/generated/*` are untouched, so `adico add`/`adico init` behavior
  for external consumers does not change.
