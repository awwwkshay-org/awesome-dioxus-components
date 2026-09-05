## 1. New control primitives

- [ ] 1.1 Add `NumberControl` and `OptionalBoolControl` to
      `apps/playground/src/components/controls.rs` (design.md's D1
      signatures). Verify with a quick manual render in `dx serve` against
      a throwaway test page before wiring either into a real component
      page.
- [ ] 1.2 Change `SelectControl`'s signature to `(label, value: Signal<T>,
      options: &'static [(&'static str, T)])`, matching
      `BoolControl`/`TextControl`. Update every existing call site across
      `apps/playground/src/pages/*.rs` in the same task (do not leave a
      mixed old/new signature in the tree). Verify `cargo check --locked
      -p adico-playground` (or the workspace check) passes.

## 2. Generated `DemoState`/`Controls`/`Preview`

- [ ] 2.1 Extend `packages/adico-xtask/src/playground_controls.rs`'s
      `PropShape`/`classify_prop_type` to support numeric types (→
      `NumberControl`) and the `Option<ReadSignal<Option<bool>>>` /
      `Option<bool>` optional-controlled shape (→ `OptionalBoolControl`),
      keeping every other type's classification unchanged. Verify with
      unit tests covering both new shapes plus a regression test that
      every currently-passing `classify_prop_type` case in the existing
      test module still passes unchanged.
- [ ] 2.2 Generate `pub struct <Comp>DemoState` + `Default` per component
      with at least one controllable prop, and `#[component] pub fn
      <Comp>Controls(state: Signal<<Comp>DemoState>) -> Element`
      rendering one control per field, reusing the existing enum-option
      generation and exhaustiveness guard unchanged. Verify by running
      `cargo run -p adico-xtask -- playground-controls sync` and
      inspecting `apps/playground/src/generated/controls/button.rs`
      against design.md's D2 worked example.
- [ ] 2.3 For exactly the 16 single-root, non-generic items named in
      design.md's Context (`aspect-ratio`, `badge`, `button`, `input`,
      `label`, `mode-toggle`, `progress`, `scroll-area`, `skeleton`,
      `spinner`, `switch`, `textarea`, `theme-builder`, `theme-switcher`,
      `toggle`, `virtual-list`), also generate `#[component] pub fn
      <Comp>Preview(state: <Comp>DemoState, children: Element) ->
      Element`. Verify `sync` emits a `Preview` for exactly these 16 and
      no others, and that `cargo check --locked --workspace` passes with
      the new generated code compiled in.
- [ ] 2.4 Wire `playground-controls sync|check|diff` (already dispatched
      in `main.rs`) to cover the new generated shapes; no new CLI
      subcommand needed. Verify `check` fails against a hand-edited
      generated file and passes after `sync`, matching the existing
      idempotence/drift behavior for the enum-option generator.

## 3. Full coverage — missing items and uncontrolled pages

- [ ] 3.1 Install `attachment`, `bubble`, `data-table`, `marker`,
      `message`, and `message-scroller` into `apps/playground` via `adico
      add` (real CLI path). Verify `apps/playground/components.json`/
      `adico.lock` record the new items and `cargo check --locked
      --workspace` still passes.
- [ ] 3.2 Add all 7 missing pages (`attachment`, `bubble`, `data-table`,
      `marker`, `message`, `message-scroller`, `theme-builder`), each
      using its generated `Controls` (and `Preview` if the item is one of
      the 16) from day one — no interim hand-written wiring. Add each
      page's `#[route]` variant in `apps/playground/src/routes.rs`,
      `nav_items()` entry, and `apps/playground/src/pages/mod.rs` export.
      Verify all 66 routes render in `dx serve`.
- [ ] 3.3 Wire the generated panel into the 23 currently-uncontrolled
      pages (`breadcrumb`, `carousel`, `checkbox`, `collapsible`,
      `color_picker`, `command`, `drag_and_drop_list`, `input_otp`,
      `kbd`, `label`, `mode_toggle`, `navigation_menu`, `radio_group`,
      `resizable`, `scroll_area`, `slider`, `spinner`, `table`,
      `tag_group`, `theme_switcher`, `toast`, `toolbar`,
      `virtual_list`). Verify each page's controls actually change the
      live preview in `dx serve`, one page at a time.
- [ ] 3.4 Convert the 36 pages with hand-written controls to the
      generated panel (design.md's D4 conversion order — after 3.3, not
      before), diffing each page's exposed props and live behavior in
      `dx serve` before/after conversion so a prop the hand-written
      version exposed isn't silently dropped. Verify every converted
      page still exposes at least the same prop set it did before
      conversion (more is fine if the generator now supports a
      previously-unsupported shape; less requires an explicit, reviewed
      reason).

## 4. Second consumer: `apps/docs` props table

- [ ] 4.1 Emit the introspected prop data (`FileIntrospection`/
      `PropField`) consumable by `apps/docs` at build time, and render a
      per-component props table. Verify `apps/docs` builds and renders a
      table for at least `Button`, `Dialog`, and one of the 16
      single-root items, matching the real current props of each.

## 5. Validate

- [ ] 5.1 Run the full baseline: `cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings` (documented baseline;
      also run the wider `--workspace` form and report any failure
      outside this change's own files separately), `cargo test --locked
      -p adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask`, `cargo run -p adico-xtask --
      playground-controls check`, and `openspec validate
      generate-playground-control-panels --strict`. Verify all pass;
      report any that don't and why.
- [ ] 5.2 `cd tests/playwright && npm test` for keyboard/axe coverage on
      every page whose controls changed. Verify all pass; report any
      surface with no existing fixture rather than claiming it passed.
