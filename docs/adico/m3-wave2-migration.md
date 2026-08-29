# M3 Wave 2 migration (task 4.5a)

Status: in progress

This records task 4.5a: the sixteen-item Wave 2 batch from the
[M3 migration queue](m3-migration-queue.md#wave-2--single-self-contained-new-primitive-task-43-continued) —
`accordion`, `alert-dialog`, `aspect-ratio`, `avatar`, `checkbox`,
`collapsible`, `label`, `progress`, `radio-group`, `scroll-area`, `slider`,
`switch`, `tabs`, `toast`, `toggle`, `toggle-group`. `separator` (also Wave 2)
was already migrated as a primitive during Wave 4 as a hard dependency of
`sidebar` (see [`m3-wave4-migration.md`](m3-wave4-migration.md)); it is not a
public `registry:ui` item, and adding one is out of this task's explicit
sixteen-item scope.

Each item is independent (per the queue's own framing), so this batch is
worked as smaller sub-batches grouped by what the port actually costs, not
alphabetically, following the M3 precedent of dedicated per-wave records.

## Sub-batch 1 — no-behavior styled wrappers (complete)

`aspect-ratio`, `label`, `progress`. Chosen first to prove the pipeline
end-to-end on trivial cases: none of the three uses `document::eval`, a
portal, or any DOM measurement API, so none needed target-gated adapter work
and all three are SSR-safe by construction.

- `packages/adico-primitives/src/{aspect_ratio,label,progress}.rs`: forked
  from upstream unmodified (only the doc-example import path changed from
  `dioxus_primitives`/`dioxus_primitives::aspect_ratio` etc. to
  `adico_primitives`), provenance-tracked in
  `provenance/records/adico-primitives-wave2-simple.json`.
- `registry/ui/{aspect_ratio,label,progress}.rs`: styled shadcn-convention
  facades. `aspect-ratio` is a pure re-export (upstream ships it unstyled —
  nothing to compose). `label`/`progress` compose `cn` and semantic tokens.
  Both discovered and worked around the same real constraint: a registry
  facade wrapping an `adico-primitives` **component** (not a native HTML
  element) cannot forward a generic `#[props(extends = GlobalAttributes)]
  attributes: Vec<Attribute>` field to that primitive component via
  `..props.attributes` — Dioxus's rsx spread-into-component-call syntax only
  accepts spreading another Props struct (`..props`), not a raw
  `Vec<Attribute>`; `..attributes` spread is only valid targeting a native
  HTML element. This is why the codebase's existing component-wrapping
  facades (`tooltip.rs`, `button.rs`, `dialog.rs`) never declared a generic
  `attributes` passthrough field in the first place — `progress.rs`
  originally did and failed to compile in the consumer fixture with `no
  field ... on type Vec<Attribute>` errors; fixed by dropping the generic
  field and adding a typed `aria_label: Option<String>` prop forwarded by
  name instead (Dioxus lets you set an extended global attribute by name
  directly on a component call site).
- `registry/registry.json`: three new entries (`aspect-ratio` has no
  `registryDependencies`, matching its pure re-export; `label`/`progress`
  depend on `cn`). Neither needs the `web` adico-primitives feature — no DOM
  interop.
- `packages/adico-cli/src/main.rs`: three new `include_bytes!` match arms,
  plus the 24 → 25 item expected-catalog list in
  `discovery_uses_default_and_explicit_configured_sources_without_mutation`.
- `tests/installation/wave2-simple-consumer`: a real consumer fixture
  (native web dependencies only, no `Dioxus.toml` — matching `wave1-consumer`
  precedent for a non-interactive batch with no Playwright spec), installed
  and built through the real `adico` binary.

### Verification

```
cargo xtask registry validate
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check -p adico-primitives --locked --no-default-features --features web --target wasm32-unknown-unknown
cargo check -p adico-primitives --locked --features desktop
cargo xtask provenance check
(cd tests/installation/wave2-simple-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add aspect-ratio label progress && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 25 item payload(s) (22 existing + 3 new) |
| `cargo xtask provenance check` | 4 imported record(s), 37 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed, including new `aspect_ratio`/`label`/`progress` doctests and the updated 25-item catalog assertion |
| `adico-primitives` web+wasm32 / desktop feature checks | both passed |
| `wave2-simple-consumer`: `adico init && adico add aspect-ratio label progress` | plan reported `@adico/aspect-ratio, @adico/cn, @adico/label, @adico/progress`; `adico add complete.` |
| `wave2-simple-consumer`: `cargo build` (native) / `cargo check --target wasm32-unknown-unknown` | both succeeded |

No dedicated Playwright spec: none of the three items has meaningful
keyboard/focus behavior (AspectRatio is a pure CSS wrapper, Label is a
`for`/`id` association with no own interactivity, Progress has no user
interaction), matching the precedent already set for Badge/Card/Input/
Textarea/Skeleton/Item in Wave 1.

## Remaining sub-batches

- **Sub-batch 2 — simple state primitives**: avatar, checkbox, collapsible,
  switch, toggle.
- **Sub-batch 3 — roving-focus group**: accordion, radio-group, tabs,
  toggle-group.
- **Sub-batch 4 — named-risk items**: alert-dialog, scroll-area, slider,
  toast.

Task 4.5a's checkbox in `tasks.md` is marked complete only once all four
sub-batches above are done and independently verified.
