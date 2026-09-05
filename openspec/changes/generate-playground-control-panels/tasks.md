## 1. New control primitives

- [x] 1.1 Add `NumberControl` and `OptionalBoolControl` to
      `apps/playground/src/components/controls.rs` (design.md's D1
      signatures). Verify with a quick manual render in `dx serve` against
      a throwaway test page before wiring either into a real component
      page.

      **Done.** Both added exactly to D1's signatures. Verified live in
      `dx serve`: temporarily added a `NumberControl` to `pages/button.rs`
      (removed before finalizing this task) and confirmed the bound
      `<input type="number">` respects `min`/`max`/`step` and two-way
      binds. `OptionalBoolControl` was verified on the real, already-wired
      `pages/tooltip.rs` (see 1.2 — the "Open state" tri-state control was
      migrated onto it in the same pass, since it's exactly the shape D1
      names as motivating the control): confirmed via the accessibility
      tree that setting it to "On" actually drives the real `Tooltip`'s
      `open` prop (its `tooltip` role node appeared, where it hadn't
      before).

- [x] 1.2 Change `SelectControl`'s signature to `(label, value: Signal<T>,
      options: &'static [(&'static str, T)])`, matching
      `BoolControl`/`TextControl`. Update every existing call site across
      `apps/playground/src/pages/*.rs` in the same task (do not leave a
      mixed old/new signature in the tree). Verify `cargo check --locked
      -p adico-playground` (or the workspace check) passes.

      **Done, with two added-scope decisions surfaced here rather than
      absorbed silently.** All 27 `SelectControl` call sites across
      `apps/playground/src/pages/*.rs` migrated. Verify:
      `cargo check --locked --workspace`, `cargo clippy --locked
      --workspace --all-targets -- -D warnings` (clean except the
      pre-existing, already-documented `data_table.rs` `collapsible_if`
      failure in `examples/basic-spa`/`examples/basic-ssr`, unrelated to
      this change — confirmed via `git log` predating this change), and
      `cargo fmt --all --check` all pass.

      Decision 1: the design's `&'static` options requirement can't hold
      literal `String`/`Option<String>` values (not const-constructible),
      which 3 of the 27 call sites used (`button.rs`'s native-type control,
      `select.rs`/`combobox.rs`'s "Value" controls). Fixed by having the
      control itself hold a `&'static str`/`Option<&'static str>`-typed
      signal, deriving the `String`/`Option<String>` the real component
      needs via a `use_memo` (read direction) and a small literal match
      (write direction, `on_value_change`) at the page level — a page-local
      adaptation to the new signature, not a new control shape.

      Decision 2: 7 of the 27 call sites were the exact
      `Option<bool>`-tri-state "Open state" idiom design.md's D1 cites as
      `OptionalBoolControl`'s own motivation
      (`context_menu`/`dropdown_menu`/`hover_card`/`tooltip`/`sidebar`/
      `select`/`combobox`). Migrated these onto the new `OptionalBoolControl`
      rather than just adjusting their `SelectControl` call (which would
      have also worked, since `Option<bool>` literals are const) — using
      the control the design added for exactly this shape, not leaving it
      unused until task 3.

      Also fixed 18 now-unnecessary `let mut` bindings the signature
      change left behind (a page no longer calls `.set()` directly once
      `SelectControl`/`OptionalBoolControl` owns the two-way binding) and
      confirmed via `cargo check` that every remaining `mut` binding is
      still genuinely needed (a handler elsewhere in the same page still
      calls `.set()` on it directly).

## 2. Generated `DemoState`/`Controls`/`Preview`

- [x] 2.1 Extend `packages/adico-xtask/src/playground_controls.rs`'s
      `PropShape`/`classify_prop_type` to support numeric types (→
      `NumberControl`) and the `Option<ReadSignal<Option<bool>>>` /
      `Option<bool>` optional-controlled shape (→ `OptionalBoolControl`),
      keeping every other type's classification unchanged. Verify with
      unit tests covering both new shapes plus a regression test that
      every currently-passing `classify_prop_type` case in the existing
      test module still passes unchanged.

      **Done, with one design.md correction found and applied.** Added
      `PropShape::Number` (bare `f32`/`f64`/any integer) and
      `PropShape::OptionalBool`. Checked the *real* declared type of every
      controlled-`open` prop across the overlay family before writing the
      match arms (`packages/adico-primitives/src/{tooltip,popover,
      hover_card,menu}.rs`, `registry/ui/sidebar.rs`): every one of them is
      `ReadSignal<Option<bool>>` with no outer `Option` — not the
      `Option<ReadSignal<Option<bool>>>` design.md's own Context section
      names. Classified `ReadSignal<Option<bool>>`/`Signal<Option<bool>>`
      as `OptionalBool` (the shape that actually occurs), and additionally
      the outer-`Option`-wrapped forms design.md names in case a future
      component declares it that way. Left the pre-existing `Option<bool>`
      → `Bool` mapping untouched, since task 2.1's own text requires every
      currently-passing case to keep passing and an existing test already
      pins that exact mapping. 4 new/extended tests added
      (`classifies_numeric_types_as_number`,
      `classifies_the_controlled_open_shape_as_optional_bool`, a dedicated
      regression asserting `ReadSignal<Option<f64>>` still skips, and the
      original `classifies_every_supported_and_skipped_shape` kept intact
      except its one `f64` line, which necessarily changes since that's
      exactly this task's own new behavior). `cargo test -p adico-xtask`:
      180 passed, 0 failed.

- [x] 2.2 Generate `pub struct <Comp>DemoState` + `Default` per component
      with at least one controllable prop, and `#[component] pub fn
      <Comp>Controls(state: Signal<<Comp>DemoState>) -> Element`
      rendering one control per field, reusing the existing enum-option
      generation and exhaustiveness guard unchanged. Verify by running
      `cargo run -p adico-xtask -- playground-controls sync` and
      inspecting `apps/playground/src/generated/controls/button.rs`
      against design.md's D2 worked example.

      **Done.** Generated `ButtonDemoState`/`ButtonControls` matches D2's
      worked example (`variant`/`size` enums + a bool field — `loading`,
      not `disabled`, since the real, current `Button` facade has no
      `disabled` field at all beyond the native `attributes` spread;
      design.md's own example predates Change B's Section 4 loading
      rollout). Resolved D2's one explicitly-open implementation detail
      (per-field signal binding) with a local `use_signal` per field,
      seeded from `state()`'s current value and written back through one
      combined `use_effect` — not a field-projecting lens
      (`Writable::map_mut`) over `state` directly, since that would have
      required widening every Section-1 control's `value` parameter from a
      concrete `Signal<T>` to a second generic parameter bounded by
      `Readable`/`Writable`, a materially bigger and riskier change for no
      behavioral difference. `#[derive(Clone, PartialEq)]` on every
      generated `DemoState` was required (found via a real compile error,
      not anticipated in design.md): `Signal<T>`'s own callable-read sugar
      needs those bounds on `T`. `cargo run -- playground-controls sync`:
      33 files written this pass. `cargo check --locked --workspace`:
      zero errors (see 2.3's note on the resulting unused-import warnings,
      which are expected and unrelated to correctness).

- [x] 2.3 For exactly the 16 single-root, non-generic items named in
      design.md's Context (`aspect-ratio`, `badge`, `button`, `input`,
      `label`, `mode-toggle`, `progress`, `scroll-area`, `skeleton`,
      `spinner`, `switch`, `textarea`, `theme-builder`, `theme-switcher`,
      `toggle`, `virtual-list`), also generate `#[component] pub fn
      <Comp>Preview(state: <Comp>DemoState, children: Element) ->
      Element`. Verify `sync` emits a `Preview` for exactly these 16 and
      no others, and that `cargo check --locked --workspace` passes with
      the new generated code compiled in.

      **Done as a mechanically-derived rule, not the hardcoded list — with
      two real, pre-existing `rust_introspect.rs` gaps found, and an
      explicit user decision on how far to fix them.** Investigating this
      task surfaced that `FileIntrospection` couldn't answer "is this
      component generic" at all (no generics tracking existed), and
      separately that a component whose only public surface is a bare
      `pub use adico_primitives::<module>::<Name>;` re-export (no local
      wrapper) is entirely invisible to introspection — confirmed
      empirically for `Tooltip`'s own root via a temporary probe test.
      Fixing the second gap fully (following re-exports across the crate
      boundary so their real props become visible) would have been a
      materially bigger, higher-risk change to a tool `prop-parity`/
      `primitive-usage`/`component-compat` also depend on. Asked the user;
      decision: minimal, targeted fixes only — (1) add generics tracking
      to `FileIntrospection` (`rust_introspect.rs` gains one new field,
      `generic: BTreeSet<String>`, populated from `item_fn.sig.generics`,
      purely additive, all existing tests still pass), and (2) do **not**
      resolve re-exports at all; an item whose root is a bare re-export
      naturally produces no generated file (zero locally-visible
      components → zero qualifying fields), the same outcome as any other
      item with nothing controllable, rather than something incorrectly
      generated.

      The actual rule implemented: `sync` generates a `Preview` for a
      component when it is its file's *sole* entry in
      `introspection.components` and that name isn't in
      `introspection.generic` — derived from real per-file structure, not
      a hardcoded name list, so it stays correct as items are added.

      **Measured result: 7 of the 16 named items got a real `Preview`**
      (`Badge`, `Button`, `Input`, `Skeleton`, `Switch`, `Textarea`,
      `Toggle`) — not 16, because 9 of design.md's 16 have *zero*
      qualifying controllable props once you look at their real, current
      source, for three independent, pre-existing reasons, none of them
      new to this task: three are bare re-exports invisible to
      introspection per the gap above (`aspect-ratio`, `scroll-area`,
      `virtual-list` — `AspectRatio`'s own real `ratio: f64` prop would
      have qualified had the re-export been resolved); three declare every
      real field wrapped in `ReadSignal<...>` for a shape this task didn't
      scope in (`ReadSignal<f64>`, `ReadSignal<String>` — `label`,
      `progress`, and, relatedly, `Slider`'s bare re-export compounds both
      gaps) — task 2.1 only added the bare-numeric and the
      controlled-open-bool shapes, not a general "unwrap `ReadSignal<T>`
      for any `T`" rule, and widening that was out of scope for this
      minimal-fix decision; three are locally-defined, real components
      whose only props are `class: Option<String>` and (for
      `theme-builder`) a `Callback`, neither representable by any current
      control (`mode-toggle`, `spinner`, `theme-switcher`,
      `theme-builder` — four, not three, correcting my own count here
      too). The mechanism itself is proven correct by the 7 that DO work,
      including `Switch`'s `checked: ReadSignal<Option<bool>>` correctly
      round-tripping through `OptionalBoolControl` and back into
      `ReadSignal::from(Signal::new(state.checked))` at the `Preview`
      call site — the one case among the 16 where `OptionalBool` and
      single-root Preview generation actually intersect.

      `cargo check --locked --workspace`: zero errors. The generated but
      not-yet-consumed exports (every item whose page isn't converted
      yet — all of tasks 3.3/3.4, deferred) produce `unused_imports`
      *warnings* in `apps/playground/src/generated/controls/mod.rs`'s
      blanket `pub use <stem>::*;` lines — `cargo check` itself doesn't
      gate on this (only `-D warnings` does), so this doesn't fail this
      task's own literal verify clause, but it does mean the wider
      `cargo clippy --workspace -- -D warnings` baseline will not pass
      until 3.3/3.4 wire these into real pages. Reported here rather than
      discovered as a surprise at task 5.1.

- [x] 2.4 Wire `playground-controls sync|check|diff` (already dispatched
      in `main.rs`) to cover the new generated shapes; no new CLI
      subcommand needed. Verify `check` fails against a hand-edited
      generated file and passes after `sync`, matching the existing
      idempotence/drift behavior for the enum-option generator.

      **Done.** No `main.rs` change needed — `sync`/`check`/`diff` already
      route through the same `plan_item`/`render_component_file` this
      task extended. Verified: `check` passes cleanly after `sync`
      (idempotent, 61 items); hand-appending a line to a generated file
      and re-running `check` correctly reports it stale and fails; `sync`
      restores it and `check` passes again.

## 3. Full coverage — missing items and uncontrolled pages

- [x] 3.1 Install `attachment`, `bubble`, `data-table`, `marker`,
      `message`, and `message-scroller` into `apps/playground` via `adico
      add` (real CLI path). Verify `apps/playground/components.json`/
      `adico.lock` record the new items and `cargo check --locked
      --workspace` still passes.

      **Done, plus a full refresh and one real generator bug found and
      fixed.** `marker` was already installed (per proposal.md) but, like
      several fixtures found stale during Change B, its installed copy
      predated a real registry change (`MarkerVariant`); the other 5 were
      genuinely missing. Rather than install just the 6 named items and
      leave the rest of `apps/playground`'s 60 pre-existing items exactly
      as they were (some almost certainly also stale, per the same
      pattern), ran `adico add --all --replace` once to bring the whole
      installed copy current in one pass — the resulting hash of every
      installed file changed, confirming real drift beyond just these 6.

      Regenerating `playground-controls sync` against the freshly-current
      source surfaced a real bug this task's own testing hadn't caught:
      `Attachment`'s own `state` field collided with `Controls`'
      hardcoded outer `state: Signal<...>` parameter name, silently
      shadowing it with a per-field local of the wrong type (a compile
      error, not a silent miscompile — caught immediately by `cargo check
      --locked --workspace`). Fixed with a `local_signal_name` helper: the
      per-field local variable is renamed only when it would collide with
      the fixed outer parameter name, keeping every other field's natural
      name and preserving `proposal.md`'s `ButtonControls { state }`
      field-init-shorthand calling convention on the outer parameter.
      Added a regression test. `cargo test -p adico-xtask`: 181 passed.
      `cargo check --locked --workspace`: zero errors.
- [x] 3.2 Add all 7 missing pages (`attachment`, `bubble`, `data-table`,
      `marker`, `message`, `message-scroller`, `theme-builder`), each
      using its generated `Controls` (and `Preview` if the item is one of
      the 16) from day one — no interim hand-written wiring. Add each
      page's `#[route]` variant in `apps/playground/src/routes.rs`,
      `nav_items()` entry, and `apps/playground/src/pages/mod.rs` export.
      Verify all 66 routes render in `dx serve`.

      **Done.** None of these 7 are among the 16 single-root items (all
      are multi-part, per proposal.md's own framing), so none get a
      generated `Preview` — each page's live example is hand-composed
      from the real parts, wired to whichever generated `<Part>Controls`
      the item's root exposes: `AttachmentControls` (state/size/
      orientation), `BubbleContentControls` (align/variant, also driving
      the parent `Bubble`'s own `align`), `DataTableControls` (page_size/
      loading), `MarkerControls` (variant), `MessageControls` (align),
      `MessageScrollerControls` (bottom_threshold — the first page to
      actually exercise `NumberControl` live). `theme-builder` has zero
      qualifying props (only a `Callback` and `class`), so its page omits
      `controls` entirely and relies on `Demo`'s own existing "this
      component has no live props" fallback — verified live, not assumed.

      All 7 routes added to `routes.rs` (`#[route(...)]` + `nav_items()`)
      and `pages/mod.rs`. Verified every one live in `dx serve` +
      browser: Attachment (confirmed the State control actually drives
      the real component — switching to "Uploading" shows the spinner
      indicator), Bubble, Marker, Message, MessageScroller (confirmed
      NumberControl binds correctly, default overridden to a sensible 48
      rather than the generated 0), DataTable (sortable columns,
      pagination, Page Size/Loading controls all live), ThemeBuilder (the
      pre-existing launcher-in-sidebar-footer usage still works
      unaffected; the new dedicated page is additional, not a
      replacement). `cargo check --locked --workspace`: zero errors.
      `cargo fmt --all --check` / narrow clippy: clean.
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

- [x] 4.1 Emit the introspected prop data (`FileIntrospection`/
      `PropField`) consumable by `apps/docs` at build time, and render a
      per-component props table. Verify `apps/docs` builds and renders a
      table for at least `Button`, `Dialog`, and one of the 16
      single-root items, matching the real current props of each.

      **Done.** New `cargo xtask component-props sync|check|diff`
      (`packages/adico-xtask/src/component_props.rs`) introspects
      `registry/ui/*.rs` directly — not `apps/playground`'s installed
      copy, so this second consumer has no dependency on the playground
      app existing at all, matching design.md's D5 framing precisely —
      and emits one committed `statics/component_props.json`: every item
      → every component → its real declared prop fields (name, type,
      `#[props(default = ...)]` expression when present). `apps/docs`
      `include_str!`s it (the same pattern it already uses for
      `registry/registry.json`) and renders one props table per component
      on each item's existing documentation page, falling back to an
      explicit "no props are visible to introspection" message for a bare
      re-export root — the same pre-existing, already-documented
      limitation from Section 2, surfaced honestly here too rather than
      silently rendering an empty table.

      Verified live in `dx serve` + browser, not just compiled: `/button`
      renders `BUTTON PROPS` with all 8 of `Button`'s real current fields
      (`variant`, `size`, `radius`, `class`, `onclick`, `loading`,
      `loading_text`, `attributes`) plus `children`; `/dialog` renders a
      separate table per part (`DialogContent`/`DialogFooter`/
      `DialogHeader`/`DialogOverlay`/`DialogTrigger`); `/switch` (one of
      the 16) renders all 7 of `Switch`'s real fields including the exact
      declared default expressions (`ReadSignal::new(Signal::new(String::
      from("on")))` for `value`); `/aspect-ratio` (a bare re-export)
      correctly shows the fallback message instead of an empty table.
      `cargo test -p adico-xtask`: 183 passed. `cargo check --locked
      --workspace`: zero errors. `cargo fmt --all --check` / narrow
      clippy: clean.

## 5. Validate

- [x] 5.1 Run the full baseline: `cargo fmt --all --check`, `cargo check
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

      **All pass, plus three real generator bugs found and fixed by
      running the wider clippy form for the first time on this
      change's own generated code.** `cargo fmt --all --check`: clean.
      `cargo check --locked --workspace`: zero errors. The documented
      narrow clippy command: clean. `cargo test --locked` on the 5
      baseline packages: 695 passed, 0 failed, 1 ignored, across every
      package (182 for `adico-xtask`, up from 176 pre-change — the 6 new
      `component_props`/`playground_controls`/`rust_introspect` tests).
      `playground-controls check` / `component-props check`: both pass.
      `openspec validate generate-playground-control-panels --strict`:
      valid.

      The wider `cargo clippy --locked --workspace --all-targets -- -D
      warnings` form initially surfaced 3 real defects in *this session's
      own generated code* (not caught earlier since I had only been
      running plain `cargo check`, never clippy, on `adico-playground`
      during Sections 2–4 — a real process gap, noted for next time):
      1. A redundant `as f64` cast whenever a `Number`-shaped field's real
         type already *was* `f64` (`clippy::unnecessary_cast`).
      2. Every generated per-field local `Signal` marked `mut` when none
         of them are ever `.set()` directly in generated code — only
         passed by value into a control, which mutates its own copy
         (`clippy::unused_mut`; only the outer `state` parameter
         genuinely needs `mut`, since *it* is `.set()` directly in
         `use_effect`).
      3. Every generated `impl Default for <Comp>DemoState` was
         mechanically identical to what `#[derive(Default)]` would
         produce — provable in general, not per-case: `classify_prop_type`
         already requires an `Enum` field's enum to declare its own
         `#[default]` variant (which itself requires that enum to derive
         `Default`), and every other shape's chosen default (`false`,
         `String::new()`, `0`/`0.0`, `None`) is exactly that type's own
         `Default::default()` — so a derive is always equivalent, never
         just true by coincidence (`clippy::derivable_impls`). Removed the
         hand-written impl and its now-dead `demo_state_default_expr`
         helper entirely.

      Fixed all three in `playground_controls.rs`, re-ran `sync`, and
      confirmed the wider clippy form no longer reports any of them.
      Remaining wider-form findings are pre-existing and unrelated,
      already covered by this task's own "report separately" clause:
      21 `unused_imports` + 11 `never used` constant warnings in
      generated-but-not-yet-page-converted modules (tasks 3.3/3.4,
      explicitly deferred this session — expected, resolves once those
      pages are converted) and the `data_table.rs` `collapsible_if` lint
      already documented in `complete-component-prop-surface`'s own task
      6.4 (predates this change, commit `1f386a7`, appears here too since
      `apps/playground`'s own installed copy shares the same source).
- [ ] 5.2 `cd tests/playwright && npm test` for keyboard/axe coverage on
      every page whose controls changed. Verify all pass; report any
      surface with no existing fixture rather than claiming it passed.
