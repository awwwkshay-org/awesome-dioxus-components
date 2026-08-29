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

## Sub-batch 2 — simple state primitives (complete)

`avatar`, `checkbox`, `collapsible`, `switch`, `toggle`. Each is
self-contained (no shared collection/roving-focus infrastructure, no
`src/portal.rs`), one state hook per item.

- `packages/adico-primitives/src/{avatar,checkbox,collapsible,switch,toggle}.rs`:
  forked from upstream, provenance-tracked in
  `provenance/records/adico-primitives-wave2-state.json`. `switch.rs` and
  `toggle.rs` ported unmodified (no `document::eval`, no
  `dioxus-attributes`/`merge_attributes`). `collapsible.rs` dropped upstream's
  `dioxus-attributes`/`merge_attributes` helper crate and its polymorphic
  `r#as` render-prop escape hatch, rewritten with the plain
  `..props.attributes` spread pattern this crate's `dialog`/`tooltip` modules
  already established (no other primitive here exposes an `r#as` escape
  hatch, so it was dropped rather than ported); it has no `document::eval`
  usage and is SSR-safe as-is. `avatar.rs`'s `AvatarImage` and `checkbox.rs`'s
  internal `BubbleInput` each had one unconditional `document::eval` call —
  Avatar's reconciles cached/very-fast image loads that can complete before
  Dioxus delivers the synthetic onload/onerror event, and Checkbox's
  `BubbleInput` syncs a hidden native `<input type="checkbox">`'s
  `checked`/`indeterminate` DOM properties (Dioxus's `checked` attribute only
  sets the initial default, and `indeterminate` has no HTML attribute
  equivalent at all) — both are now behind this crate's established
  `#[cfg(any(feature = "web", feature = "desktop"))]` target-gated adapter
  pattern with SSR-safe no-op fallbacks, matching `context_menu.rs`/
  `popover.rs`'s Wave 3 precedent.
- `registry/ui/{avatar,checkbox,collapsible,switch,toggle}.rs`: styled
  shadcn-convention facades using `cn` and semantic tokens.
  `collapsible.rs` is a pure re-export, matching `aspect-ratio`'s Sub-batch 1
  precedent — upstream (and real shadcn) ships Collapsible unstyled, layout
  left to consumer composition. `checkbox.rs` adds a Lucide `Check` indicator
  icon, matching shadcn's Checkbox. All five follow Sub-batch 1's discovered
  constraint: no generic `attributes: Vec<Attribute>` passthrough field on a
  facade wrapping a primitive component, only specific typed props forwarded
  by name (`class`, `aria_label`, `checked`, etc.).
- `registry/registry.json`: five new entries. `avatar`/`checkbox` declare the
  `adico-primitives` `web` feature (both need the browser DOM for their
  target-gated eval adapters); `collapsible`/`switch`/`toggle` do not.
- `packages/adico-cli/src/main.rs`: five new `include_bytes!` match arms,
  plus the 25 → 30 item expected-catalog list update (alphabetical
  insertion — this list has no auto-generation and silently drifts, exactly
  as Sub-batch 1 found).
- `tests/installation/wave2-state-consumer`: a real consumer fixture (with
  `Dioxus.toml`, since this sub-batch has genuine keyboard/ARIA behavior
  worth a live browser check), installed and built through the real `adico`
  binary, composing all five items together.
- `tests/playwright/wave2-state.spec.ts`: real-browser coverage via a live
  `dx serve --platform web` run — Checkbox/Switch toggle by click and Space
  with correct `aria-checked`, Toggle toggles `aria-pressed` by click,
  Collapsible expands/collapses via its trigger with correct
  `aria-expanded`, and a whole-page axe scan. Avatar has no dedicated
  assertion (no keyboard interaction of its own — passive image-load state,
  same disposition as Wave 1's non-interactive items) but is rendered on the
  page and covered by the axe scan.

### Verification

```
cargo xtask registry validate
cargo xtask provenance check
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check -p adico-primitives --locked --no-default-features --features web --target wasm32-unknown-unknown
cargo check -p adico-primitives --locked --features desktop
(cd tests/installation/wave2-state-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add avatar checkbox collapsible switch toggle && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave2-state-consumer && dx serve --platform web --port 8793) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8793 npm run test:wave2-state)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 30 item payload(s) (25 existing + 5 new) |
| `cargo xtask provenance check` | 6 imported record(s), 45 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed (66 primitive doctests, 21 `adico-cli` unit tests, 14 CLI integration tests, updated 30-item catalog assertion) |
| `adico-primitives` web+wasm32 / desktop feature checks | both passed |
| `wave2-state-consumer`: `adico init && adico add avatar checkbox collapsible switch toggle` | plan reported `@adico/cn, @adico/avatar, @adico/checkbox, @adico/collapsible, @adico/switch, @adico/toggle`; `adico add complete.` |
| `wave2-state-consumer`: `cargo build` (native) / `cargo check --target wasm32-unknown-unknown` | both succeeded |
| `wave2-state.spec.ts` (5 tests, live `dx serve` + Playwright + axe) | all passed: Checkbox click/Space toggle with `aria-checked`, Switch click/Space toggle with `aria-checked`, Toggle click toggles `aria-pressed`, Collapsible expand/collapse with `aria-expanded`, zero critical axe violations |

## Sub-batch 3 — roving-focus group (complete)

`accordion`, `radio-group`, `tabs`, `toggle-group`. All four already depend
on this crate's existing private `collection` module (`collection_item`,
`use_item`, `CollectionOptions`/`CollectionState`,
`use_collection_provider`/`use_collection_provider_with`) — the same shared
roving-focus infrastructure already imported for Select in M1 and reused by
DropdownMenu/ContextMenu/Menubar in Wave 3, confirmed by reading the actual
upstream source rather than assumed from the migration queue's one-line
descriptions. No new shared primitive was needed; this validates the queue's
own Wave 3 note that this infrastructure generalizes.

- `packages/adico-primitives/src/{accordion,radio_group,tabs,toggle_group}.rs`:
  forked from upstream, provenance-tracked in
  `provenance/records/adico-primitives-wave2-roving-focus.json`.
  `accordion.rs` dropped the one upstream-internal
  `use crate::dioxus_elements::Key;` import (a re-export path specific to
  upstream's own crate layout that does not exist here) since `Key` already
  resolves through the existing `use dioxus::prelude::*` glob import,
  matching every other keyboard-handling primitive in this crate.
  `radio_group.rs` and `tabs.rs` needed no adaptation beyond the doc-example
  import path change. `toggle_group.rs` was deferred within this sub-batch
  until its hard dependency `crate::toggle::Toggle` landed via the
  concurrently-running Wave 2 state sub-batch (commit `6664368`), then ported
  unmodified — it composes the owned `Toggle` primitive directly as a plain
  Rust module dependency, not a registry dependency edge (matching the
  migration queue's Wave 5 `color-picker` composition precedent). None of the
  four uses `document::eval`, a portal, or a DOM measurement API, so all are
  SSR-safe by construction; no target-gated adapter work was needed.
- `registry/ui/{accordion,radio_group,tabs,toggle_group}.rs`: styled
  shadcn-convention facades using `cn` and semantic tokens. `accordion.rs`
  composes a Lucide `ChevronDown` indicator into the trigger. `radio_group.rs`
  renders its checked indicator as a CSS `::before` dot driven by the
  primitive's own `data-state` attribute (no extra child element needed,
  since the composition upstream already puts caller content directly inside
  the radio button rather than beside an indicator the way shadcn's React
  version does — a documented intentional Dioxus composition difference).
  `toggle_group.rs`'s `ToggleItem` facade reuses the same
  `data-[state=on]:bg-accent` semantic surface as the standalone `toggle`
  facade from Sub-batch 2.
- `registry/registry.json`: four new entries.
- `packages/adico-cli/src/main.rs`: four new `include_bytes!` match arms,
  plus the 30 → 34 item expected-catalog list update (alphabetical
  insertion).
- `tests/installation/wave2-roving-focus-consumer` (with `Dioxus.toml`, since
  this batch has real roving-focus/manual-activation keyboard behavior worth
  a live browser check): a real consumer fixture composing all four items,
  installed and built through the real `adico` binary.
- `tests/playwright/wave2-roving-focus.spec.ts`: real-browser coverage via a
  live `dx serve --platform web` run — Accordion opens on click, roves focus
  with ArrowDown, and activates with Enter (closing the other item, since
  `allow_multiple_open: false`); RadioGroup roves focus *and* auto-selects on
  ArrowDown (matches upstream's automatic-activation radio semantics); Tabs
  roves focus with ArrowDown *without* switching the active panel, requiring
  a separate Enter to activate (upstream's manual-activation pattern,
  verified as a real behavioral distinction between RadioGroup and Tabs, not
  assumed); ToggleGroup roves focus with ArrowRight and toggles pressed state
  radio-style (`allow_multiple_pressed` defaults to `false`); plus a
  whole-page axe scan.

### Real concurrency note

This sub-batch was implemented in the same shared working tree as a
concurrently-running Wave 2 state sub-batch (Sub-batch 2, above). Shared
files (`packages/adico-primitives/src/lib.rs`, `registry/registry.json`,
`packages/adico-cli/src/main.rs`) were deliberately left untouched by this
sub-batch's own work until Sub-batch 2's commit (`6664368`) landed, to avoid
clobbering concurrent edits; new files (the four primitive/registry sources,
the provenance record, the consumer fixture) were written and compile-checked
independently in the meantime via a disposable scratch crate outside the
main workspace.

### Verification

```
cargo xtask registry validate
cargo xtask provenance check
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo check -p adico-primitives --locked --no-default-features --features web --target wasm32-unknown-unknown
cargo check -p adico-primitives --locked --features desktop
(cd tests/installation/wave2-roving-focus-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add accordion radio-group tabs toggle-group && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave2-roving-focus-consumer && dx serve --platform web --port 8792) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8792 npm run test:wave2-roving-focus)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 34 item payload(s) (30 existing + 4 new) |
| `cargo xtask provenance check` | 6 imported record(s), 46 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed (68 primitive doctests, updated 34-item catalog assertion) |
| `adico-primitives` web+wasm32 / desktop feature checks | both passed |
| `wave2-roving-focus-consumer`: `adico add accordion radio-group tabs`, then `adico add toggle-group` | both plans applied cleanly; `cargo build` (native) and `cargo check --target wasm32-unknown-unknown` succeeded after each |
| `wave2-roving-focus.spec.ts` (5 tests, live `dx serve` + Playwright + axe) | all passed: Accordion click/ArrowDown-roving/Enter-activate, RadioGroup ArrowDown-roving-with-auto-select, Tabs ArrowDown-roving-without-switching then Enter-activate, ToggleGroup ArrowRight-roving with radio-style press, zero critical axe violations |

## Sub-batch 4 — named-risk items (remaining)

`alert-dialog`, `scroll-area`, `slider`, `toast`. Each carries a named risk
per the migration queue: alert-dialog may delegate to the owned `dialog`
primitive's focus-trap/Escape internals rather than duplicating them (decide
at import time); scroll-area needs SSR fallback verification; slider needs
`pointer.rs`/`move_interaction.rs` and has unverified desktop pointer-capture
behavior; toast needs the target-gated timeout adapter matching
`adico-primitives::time`. Worked one at a time, not as a single batch.

Task 4.5a's checkbox in `tasks.md` is marked complete only once Sub-batch 4
above is done and independently verified.
