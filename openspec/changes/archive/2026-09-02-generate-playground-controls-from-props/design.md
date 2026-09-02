## Context

See `proposal.md` - Why for the motivating drift bugs (Badge, Item).

Relevant existing pieces:
- `packages/adico-xtask/src/rust_introspect.rs` (`syn`-based) already
  extracts `#[derive(Props)]` struct fields (name, type, `#[props(default =
  ...)]`) from a Rust source file, used by `primitive_compat.rs`/
  `component_compat.rs`. It has no enum-variant extraction today.
- `packages/adico-xtask` already has a `sync|check|diff` idiom
  (`primitive_usage.rs`, `styling_usage.rs`): `sync` regenerates and writes,
  `check` regenerates in memory and diffs against committed output without
  writing, `diff` prints the delta. Each emits one committed file per
  registry item under `statics/<kind>/<item>.json`.
- `apps/playground/src/components/ui/*.rs` is a byte-identical, CLI-installed
  copy of `registry/ui/*.rs` (confirmed via `diff -rq`) tracked by
  `apps/playground/components.json`/`adico.lock`. It is a real workspace
  member compiled by `cargo check --locked --workspace`.
- `apps/playground/src/components/controls.rs` has `BoolControl`,
  `TextControl`, `SelectControl<T: Clone + PartialEq>(label, value: T,
  options: Vec<(&'static str, T)>, on_change: EventHandler<T>)`.
  `SelectControl` selects by list position, so `T` needs no `Display`/
  `FromStr` - any `Clone + PartialEq` type works, including enums.
- Dioxus RSX component invocations are statically typed:
  `components::ui::Button { variant: <expr> }` requires `<expr>: ButtonVariant`
  at compile time. There is no supported way to drive that field from a
  runtime-typed value (`serde_json::Value`, a `String`, etc.) without a
  macro that expands to the same statically-typed call. This rules out a
  JSON-descriptor-driven runtime control renderer for the actual live demo.

## Goals / Non-Goals

**Goals:**
- Make an enum-typed prop's playground option list impossible to leave
  stale relative to its source enum.
- Reuse the existing `syn`-based introspection and `sync|check|diff`
  xtask idiom rather than inventing a parallel mechanism.
- Keep generated output inside a real, compiled workspace member so
  `cargo check --locked --workspace` is sufficient to catch drift - no
  separate CI job is load-bearing for correctness, only for convenience.

**Non-Goals:**
- Do not generate the `use_signal` declarations, the `SelectControl`/
  `BoolControl` JSX-like blocks, or the wiring of a signal into a live
  component call. These stay hand-written per page; see Decisions below
  for why removing them isn't worth it here.
- Do not build a runtime/JSON-driven generic control renderer.
- Do not wire controls into the 25 currently-uncontrolled pages as part of
  this change.
- Do not add a numeric control primitive; numeric props stay unsupported
  and excluded-with-reason.

## Decisions

### Generated output location: `apps/playground/src/generated/controls/`, not `registry/generated/` or `statics/`
`registry/generated/*` is registry metadata consumed by `adico-registry-core`
and the CLI; it is explicitly never `cargo check`'d and would ship as dead
scaffolding to every consumer if the CLI ever installed it. `statics/*`
(the `primitive_usage`/`styling_usage` pattern) is JSON consumed by tooling
and docs, not compiled Rust. Neither location gets us a `cargo check`
failure on drift. Putting generated `.rs` files directly in the `apps/
playground` crate under a `generated/` module (mirroring how `registry
build` writes derived-but-committed output elsewhere in this repo) makes
the exhaustiveness guard (below) a real compile-time check, and keeps the
artifact scoped to the one app that consumes it - it has no reason to be
registry metadata since it says nothing about the registry, only about
what the playground currently demos.

Each generated file is one-per-component, e.g.
`apps/playground/src/generated/controls/button.rs`, mirroring
`apps/playground/src/components/ui/button.rs`, aggregated by a generated
`mod.rs` the same way `components/ui/mod.rs` aggregates components today.

### `rust_introspect.rs` gains enum extraction; it is not replaced
Add an `Item::Enum` arm to `walk_items` alongside the existing `Item::Fn`/
`Item::Struct` arms, extracting: the enum name, each variant's identifier in
declaration order, and which variant (if any) carries `#[default]`. Variant
doc comments are not extracted or used: inspecting the real registry source
(`button.rs`, `badge.rs`, `item.rs`) showed they are full descriptive
sentences ("Extra-small text button.", "Primary status or category."), not
short labels, and using them verbatim would read worse in a compact control
than a mechanically humanized identifier. The label is instead derived
purely from the identifier by splitting PascalCase into space-separated
words (`IconXs` -> `Icon Xs`, `Destructive` -> `Destructive`) - a pure
function of the enum, so it's exercised entirely by
`playground_controls.rs`'s own tests and needs no new field on
`FileIntrospection`. This is additive to the existing `FileIntrospection`/
`PropField` types used by `primitive_compat.rs`/`component_compat.rs` -
those callers are unaffected.

This is a deliberate, user-confirmed departure from matching today's
hand-tuned labels exactly (e.g. today's page hand-writes "Extra small" for
`ButtonSize::Xs`; the generated label will read "Xs"). See Risks below.

### Compile-time exhaustiveness guard, generated alongside each option constant
Each generated file emits, next to its `pub const FOO_VARIANT_OPTIONS: &[(&str,
FooVariant)]`, a guard of the shape:

```rust
const _: () = {
    fn _exhaustive(value: FooVariant) {
        match value {
            FooVariant::Default => {}
            FooVariant::Destructive => {}
            // ... one arm per variant, in source order
        }
    }
};
```

This is the mechanism that turns "a CI job would catch this" into "the
build cannot succeed without it": `syn`/rustc, not this project's own
tooling, enforces exhaustiveness on that `match`. Regenerating after a
source-enum change is the only way to make the guard compile again.

### Prop-to-control mapping is a fixed allowlist, not a heuristic
`bool`/`Option<bool>` -> `BoolControl`; `String` -> `TextControl`;
enum-with-`#[default]` -> `SelectControl` + generated constant. Everything
else (`children`, `attributes: Vec<Attribute>`, `class: Option<String>`,
`EventHandler<_>`, `Signal<_>`/`ReadSignal<_>`-wrapped types, numeric types)
is recorded as skipped with a fixed reason string, mirroring
`primitive_usage.rs`'s `reason` field. A fixed allowlist (vs. "try to
render anything") keeps `check` meaningful: a prop type the tool doesn't
recognize is a visible skip, not a silent gap discovered later.

### Only the enum option lists are generated; page wiring stays hand-written
Considered generating the entire `controls: rsx! { ... }` block per page
(via a marker-region convention like `packages/adico-cli/src/modules.rs`'s
`// adico:start`/`// adico:end`, which already exists for CLI-managed
import blocks). Rejected for this change: the demo composition ordering,
which props a given page chooses to expose, and non-prop page-local knobs
(e.g. `pages/button.rs`'s `ButtonContent` enum, used only to vary demo
children) are genuinely bespoke per page and not something to infer from a
Props struct. Marker-region codegen for the option lists specifically
(inside an otherwise hand-written `controls: rsx! { ... }` block) was also
considered and rejected in favor of importing a named constant: a plain
`use crate::generated::controls::button::BUTTON_VARIANT_OPTIONS;` plus
`options: BUTTON_VARIANT_OPTIONS.to_vec()` is simpler than a marker-managed
region inside a file `adico-xtask` doesn't otherwise own, and needs no new
merge/write-back logic.

A `macro_rules!`/proc-macro layer to also shrink the per-page `use_signal` +
`SelectControl { ... }` boilerplate was considered and deferred (see
proposal.md's Non-goal). It cannot introspect struct fields on its own
(declarative macros have no reflection; a proc-macro crate would need a new
home that doesn't fit `adico-primitives`/`adico-registry-core`/`adico-cli`'s
existing roles), and the option-list generation above already removes the
majority of each page's boilerplate (e.g. `button.rs`'s option lists are
28 of 92 lines today).

## Risks / Trade-offs

- [A generated option constant has no consumer, and CI (`RUSTFLAGS: -D
  warnings` plus `cargo clippy --locked --workspace --all-targets -- -D
  warnings`, covering the whole workspace including `adico-playground` --
  wider than this repo's narrower local baseline package list) fails on the
  `never used` warning] -> Discovered live: 7 of the 15 components that
  gained a generated file (`avatar`, `button_group`, `input_group`,
  `native_select`, `switch`, `toggle`, `toggle_group`) had no existing page
  control at all. Resolved (user-confirmed) by wiring a new control into
  each of those 7 pages rather than suppressing the warning or narrowing
  generation to only pages that already had a control -- every generated
  constant has a real consumer, and those pages gain live variant/size
  switching they didn't have before.
- [Generated file drifts from a manual edit to the playground's installed
  `components/ui/*.rs` copy, independent of the registry] -> `check` command
  catches this in CI the same way `primitive-usage check`/`styling-usage
  check` do today; the exhaustiveness guard additionally catches any drift
  that changes variant identity (add/remove/rename), which is the case that
  actually matters for the option-list contents.
- [Humanized-identifier labels read slightly worse than today's hand-tuned
  prose for abbreviation-heavy variants, e.g. generated `Xs`/`Icon Xs`
  versus today's hand-written "Extra small"/"Icon extra small"] -> Accepted
  (user-confirmed): a mechanical, drift-proof label beats a polished but
  unenforceable one; a separate label-override file was considered and
  rejected as a second source of truth for no strong benefit at this scale.
- [28 pages need a follow-up edit to import and use the new constants,
  a mechanical but nontrivial diff] -> Scoped explicitly in tasks.md as
  one task per page group, verified per-page by checking the option list
  the page renders against the source enum.

## Open Questions

None - the option-list-only scope, generated-file location, and
exhaustiveness-guard mechanism were the load-bearing unknowns and are
resolved above.
