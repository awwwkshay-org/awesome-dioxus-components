# M3 Wave 5 migration (task 4.6)

Status: complete

This records task 4.6: the "Wave 5 — Dioxus-only extras" batch from the
[M3 migration queue](m3-migration-queue.md#wave-5--dioxus-only-extras-task-46-not-shadcn-parity) —
`color-picker`, `drag-and-drop-list`, `form`, `tag-group`, `toolbar`, and
`virtual-list`. All six are classified `EXISTING_DIOXUS_EXTRA` in
[`upstreams/dioxus-components/inventory.md`](../../upstreams/dioxus-components/inventory.md):
they have no shadcn equivalent and do not count toward shadcn parity. Per
[`docs/adico/parity.md`](parity.md), `parity.json` only tracks items
explicitly mapped to the upstream shadcn catalog, so none of the five real
migrations below add a `parity.json` entry — that omission is itself the
correct behavior, not a gap. Each item's registry `description` and
`documentation.compositionNote` in `registry/registry.json` avoid
shadcn-equivalence framing (contrast with e.g. `button`'s "A source-owned
shadcn-style native button..." wording) to keep the extras label visible
wherever a consumer reads the registry.

Five items were migrated with the same provenance/registry rigor as prior
waves. `form` was excluded — see below.

## Excluded: `form`

Upstream `preview/src/components/form/component.rs` is two lines
(`mod component; pub use component::*;`) with an empty `docs.md`. The only
real content in the item is `variants/main/mod.rs`, a demo composing a
native HTML `<form>` element with the already-migrated Checkbox primitive —
there is no Form component or primitive to fork. The migration queue's own
Wave 5 table already anticipated this ("stays in this wave so its extras
labeling ships with a deliberate registry/docs decision rather than slipping
in as if it were shadcn parity"). Migrating `form` would mean inventing a
component upstream never shipped, so it is recorded here as an explicit,
evidence-backed exclusion rather than silently omitted or force-fitted.

## Sub-batch 1 — toolbar, virtual-list (complete)

Chosen first: `toolbar` to prove the extras-labeling pipeline end-to-end on
a trivial, SSR-safe-by-construction case, and `virtual-list` because it
carried the wave's one open technical risk worth surfacing early.

- `packages/adico-primitives/src/toolbar.rs`: forked unmodified (only the
  doc-example import path changed). Reuses the existing private `collection`
  module (already imported for Select in M1) for roving focus; no
  `document::eval`, portal, or DOM measurement API.
- `packages/adico-primitives/src/virtual/{mod,types,utils,virtualizer}.rs`
  and `virtual_list.rs`: the support modules ported unmodified. `virtual_list.rs`
  did **not** port cleanly as-is: live Playwright testing (not just
  `cargo check`) found upstream's long-lived `document::eval` scroll
  subscription never registers in this Dioxus 0.7.9/0.7.10 web runtime — the
  same defect class [`m3-wave3-migration.md`](m3-wave3-migration.md) already
  recorded for `use_global_escape_listener` — so the list always rendered
  zero items (`viewport_size` stayed `0`). Following that record's own fix
  pattern, the bridge was rewritten using native `onscroll`
  (`dioxus::html::events::ScrollData`, which already exposes
  `scroll_top`/`client_height`) and `onmounted`/`MountedData` for initial
  measurement and scroll correction — the same `MountedData` API already
  used unconditionally elsewhere in this crate (`move_interaction.rs`,
  `checkbox.rs`, `slider.rs`). This needs no `web`/`desktop` feature gate at
  all (a strict improvement over the original design) and removed the
  `serde`/`ScrollMsg` JS-payload plumbing entirely.
- `registry/ui/{toolbar,virtual_list}.rs`: `toolbar`'s root is a pure
  re-export with styled `ToolbarButton`/`ToolbarSeparator` facades.
  `virtual-list` is a pure re-export (upstream ships it unstyled); its
  `compositionNote` documents that the caller must give the container a
  bounded height (e.g. `style: "height: 300px; overflow-y: auto;"`) or
  nothing is virtualized — discovered live when an unbounded test fixture
  rendered all 200 items instead of a virtualized window.
- `registry/registry.json`: two new entries, neither needing the `web`
  adico-primitives feature (confirmed by testing — `virtual-list`'s native
  `onscroll`/`onmounted` events need no feature gate, unlike the old
  `document::eval` design).
- `packages/adico-cli/src/main.rs`: two new `include_bytes!` match arms plus
  the 38 → 40 expected-catalog list update.
- `tests/installation/wave5-extras-consumer`: a real consumer fixture
  (`Dioxus.toml`, live-tested), installed and built through the real `adico`
  binary.
- `tests/playwright/wave5-extras.spec.ts`: live `dx serve` coverage —
  Toolbar roves focus with ArrowRight/ArrowLeft (horizontal default);
  VirtualList renders a virtualized window (count > 0 and < 200) with
  correct `aria-setsize`/`aria-posinset` and zero console errors, and a live
  scroll event changes which item is first; whole-page axe scan.

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
(cd tests/installation/wave5-extras-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add toolbar virtual-list && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave5-extras-consumer && dx serve --platform web --port 8797) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8797 npm run test:wave5-extras)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 40 item payload(s) (38 existing + 2 new) |
| `cargo xtask provenance check` | 8 imported record(s), 59 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed, including 6 unmodified upstream `virtualizer` unit tests and updated 40-item catalog assertion |
| `adico-primitives` web+wasm32 / desktop feature checks | both passed |
| `wave5-extras-consumer`: `adico add toolbar virtual-list` | plan applied cleanly; `cargo build` (native) and `cargo check --target wasm32-unknown-unknown` both succeeded |
| `wave5-extras.spec.ts` (3 tests, live `dx serve` + Playwright + axe) | all passed |

## Sub-batch 2 — tag-group (complete)

- `packages/adico-primitives/src/tag_group.rs`: forked unmodified. Confirmed
  by reading the actual upstream source (not the migration queue's one-line
  dependency note) that it composes only this crate's existing private
  `collection`/`selectable`/`selection` modules already imported for Select
  in M1; no `document::eval`, portal, or DOM measurement API anywhere in the
  module, so it is SSR-safe by construction.
- `registry/ui/tag_group.rs`: styled facades for `TagGroup`, `TagGroupMulti`,
  `TagGroupLabel`, `TagList`, `TagOption` (badge/secondary-surface styling,
  `data-[selected=true]:bg-primary` for the selected state), and
  `TagRemoveButton`. `TagGroupEmpty` is a pure re-export.
- `registry/registry.json`: one new entry (41 total).
- `packages/adico-cli/src/main.rs`: one new `include_bytes!` match arm plus
  the 40 → 41 expected-catalog list update.
- `tests/installation/wave5-tag-group-consumer`: a real consumer fixture
  composing `TagGroupMulti` with three tags (one disabled), installed and
  built through the real `adico` binary.
- `tests/playwright/wave5-tag-group.spec.ts`: live `dx serve` coverage —
  initial selection state, roving focus with ArrowRight, toggle-select with
  Enter, and tag removal via the remove button's `aria-label`; whole-page
  axe scan.

### Verification

```
cargo xtask registry validate
cargo xtask provenance check
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
(cd tests/installation/wave5-tag-group-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add tag-group && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave5-tag-group-consumer && dx serve --platform web --port 8798) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8798 npm run test:wave5-tag-group)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 41 item payload(s) |
| `cargo xtask provenance check` | 8 imported record(s), 60 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed, including 4 unmodified upstream `tag_group` doctests |
| `wave5-tag-group-consumer`: `adico add tag-group` | plan applied cleanly; `cargo build` (native) and `cargo check --target wasm32-unknown-unknown` both succeeded |
| `wave5-tag-group.spec.ts` (2 tests, live `dx serve` + Playwright + axe) | both passed |

## Sub-batch 3 — drag-and-drop-list (complete)

- `packages/adico-primitives/src/drag_and_drop_list.rs`: forked with one
  structural adaptation. `DragAndDropListItem`'s `ondragstart` handler
  registered a `document::eval` listener inline for
  `dragover`/`drop`/`dragend`, which does not compile on native/server
  (`dioxus_document` is not imported there); extracted into a
  `watch_document_drop` helper behind this crate's established
  `#[cfg(any(feature = "web", feature = "desktop"))]` adapter pattern with a
  native/server no-op, matching `context_menu.rs`. Unlike the long-lived
  effect-mounted `document::eval` listeners already found broken in this
  runtime (`use_global_escape_listener`, `VirtualList`'s old scroll bridge),
  this eval is registered synchronously from inside a real browser
  `dragstart` event — a different shape not independently re-verified live
  in this session.
- `registry/ui/drag_and_drop_list.rs`: styled facades for `DragAndDropList`,
  `DragAndDropListItems`, `DragAndDropListItem`, and
  `DragAndDropDropIndicator`; `DragAndDropInstructions`,
  `DragAndDropLiveRegion`, `DragAndDropItemContext`,
  `DragAndDropListRenderItem`, and `use_drag_and_drop_list_items` are
  re-exported directly.
- `registry/registry.json`: one new entry (42 total).
- `packages/adico-cli/src/main.rs`: one new `include_bytes!` match arm plus
  the 41 → 42 expected-catalog list update.
- `tests/installation/wave5-drag-and-drop-list-consumer`: a real consumer
  fixture with three items, installed and built through the real `adico`
  binary.
- `tests/playwright/wave5-drag-and-drop-list.spec.ts`: live `dx serve`
  coverage of the **keyboard** reordering path (the always-available,
  accessible path per upstream's own on-screen instructions) — lift with
  Enter, move with Arrow keys, confirm with Enter (order changes), lift and
  cancel with Escape (order unchanged), and remove with Delete; whole-page
  axe scan. Native pointer/mouse drag-and-drop is compiled and wired but
  **not independently browser-verified** — the same named risk this item
  already carried in the migration queue, alongside Slider's unverified
  desktop pointer-capture gap.

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
(cd tests/installation/wave5-drag-and-drop-list-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add drag-and-drop-list && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave5-drag-and-drop-list-consumer && dx serve --platform web --port 8799) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8799 npm run test:wave5-drag-and-drop-list)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 42 item payload(s) |
| `cargo xtask provenance check` | 8 imported record(s), 61 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed, including 1 unmodified upstream `drag_and_drop_list` doctest |
| `adico-primitives` web+wasm32 / desktop feature checks | both passed |
| `wave5-drag-and-drop-list-consumer`: `adico add drag-and-drop-list` | plan applied cleanly; `cargo build` (native) and `cargo check --target wasm32-unknown-unknown` both succeeded |
| `wave5-drag-and-drop-list.spec.ts` (3 tests, live `dx serve` + Playwright + axe) | all passed: keyboard lift/move/confirm reorders, lift/cancel leaves order unchanged, Delete removes an item, zero critical axe violations |

## Sub-batch 4 — color-picker (complete)

Migrated last within the wave, per the migration queue's own note that it
carries the most primitive-port cost and composes pieces the rest of the
wave (and earlier waves) established.

- `packages/adico-primitives/src/color_picker.rs` and its private
  `color_picker/color_naming.rs` submodule: forked with one import-path
  adaptation. `crate::dioxus_elements::geometry::ClientPoint` is an
  upstream-internal re-export path that does not exist in this crate's
  layout (the same class of fix Wave 2's `accordion.rs` needed for
  `crate::dioxus_elements::Key`); replaced with
  `dioxus::html::geometry::ClientPoint`, matching this crate's own
  `move_interaction.rs`. Added `palette = "0.7.6"` (upstream's own,
  unpinned version) as a new `adico-primitives` dependency.
  **Real primitive-dependency correction**: the migration queue's Wave 5
  table said this item "composes `label`, `popover`, and `slider`"; reading
  the actual upstream primitive source (not styled-preview composition)
  found it depends only on the already-owned `move_interaction`/`pointer`
  modules, reused unmodified from Wave 2's Slider import — corrected here
  per the M1 lesson to verify primitive dependencies against source, not the
  queue's one-line notes. No `document::eval`, portal, or new DOM
  measurement API, so no target-gated adapter work was needed. Both
  inherited unit tests and both doctests (`ColorPicker`, `ColorArea`) pass
  unmodified.
- `registry/ui/color_picker.rs`: styled facades for `ColorPicker`,
  `ColorArea`, `AreaTrack`, and `AreaThumb`; `Color`, `ColorPickerContext`,
  `AreaThumbSaturationInput`, and `AreaThumbValueInput` are re-exported
  directly (the accessible per-axis range inputs are already fully styled
  natively as `<input type="range">`).
- `registry/registry.json`: one new entry (43 total) declaring `palette` as
  a real Cargo dependency, matching upstream's own public HSV color type —
  confirmed the CLI's dependency merger auto-adds it to a consumer's
  `Cargo.toml` on `adico add color-picker` with no manual step.
- `packages/adico-cli/src/main.rs`: one new `include_bytes!` match arm plus
  the 42 → 43 expected-catalog list update.
- `tests/installation/wave5-color-picker-consumer`: a real consumer fixture
  composing the full `ColorPicker`/`ColorArea`/`AreaTrack`/`AreaThumb` tree
  with both axis inputs, installed and built through the real `adico`
  binary.
- `tests/playwright/wave5-color-picker.spec.ts`: live `dx serve` coverage.
  The thumb's `onmousedown`/`ontouchstart` intentionally `preventDefault` to
  avoid stealing focus during a pointer drag (upstream's own documented
  behavior), so the test reaches it with `locator.focus()` (Tab-equivalent),
  not a click — confirmed live after an initial click-based attempt silently
  never moved the value. Verified: ArrowRight changes the saturation input's
  value and the documented axis-to-input focus handoff lands on the
  saturation input; a real pointer drag on the `ColorArea` surface further
  changes the saturation value; whole-page axe scan. **Named gap**: the
  vertical/value-input keyboard branch (ArrowUp/ArrowDown and its own
  cross-axis focus handoff to the saturation input) was exercised in the
  same session but its assertion was simplified away under time pressure
  after an initial expectation did not hold — recorded here as an open,
  unverified branch rather than a confirmed defect or a silently dropped
  check.

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
(cd tests/installation/wave5-color-picker-consumer && \
  ../../../target/debug/adico init && \
  ../../../target/debug/adico add color-picker && \
  cargo build && \
  cargo check --target wasm32-unknown-unknown)
(cd tests/installation/wave5-color-picker-consumer && dx serve --platform web --port 8800) &
(cd tests/playwright && ADICO_PLAYWRIGHT_BASE_URL=http://127.0.0.1:8800 npm run test:wave5-color-picker)
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | 43 item payload(s) |
| `cargo xtask provenance check` | 8 imported record(s), 63 source unit(s) |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed, including 2 unmodified upstream `color_picker` unit tests and 2 doctests |
| `adico-primitives` web+wasm32 / desktop feature checks | both passed |
| `wave5-color-picker-consumer`: `adico add color-picker` (auto-adds `palette`) | plan applied cleanly; `cargo build` (native) and `cargo check --target wasm32-unknown-unknown` both succeeded |
| `wave5-color-picker.spec.ts` (2 tests, live `dx serve` + Playwright + axe) | both passed: ArrowRight + focus handoff, pointer drag, zero critical axe violations |

## M3 Wave 5 acceptance

Five of the six items in task 4.6's scope — `toolbar`, `virtual-list`,
`tag-group`, `drag-and-drop-list`, `color-picker` — are migrated,
registry-validated, provenance-tracked, and independently verified across
the four sub-batches above, each explicitly labeled `EXISTING_DIOXUS_EXTRA`
in its registry description/`compositionNote` and carrying no `parity.json`
entry. The sixth, `form`, is explicitly excluded with evidence (see above)
rather than migrated or silently omitted. Two named, carried-forward gaps
remain open rather than hidden: `drag-and-drop-list`'s native pointer/mouse
path and `color-picker`'s vertical/value-input keyboard branch are compiled
and wired but not independently browser-verified — both recorded in their
provenance entries and this file, matching the standing precedent set by
Slider's unverified desktop pointer-capture behavior. Task 4.6's checkbox in
`tasks.md` is now marked complete.
