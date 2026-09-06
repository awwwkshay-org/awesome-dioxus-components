## 1. Registry: ColorPicker trigger/swatch parts

- [x] 1.1 Add `ColorPickerSwatch`, `ColorPickerTrigger`, `ColorPickerPopover` to
  `registry/ui/color_picker.rs`, following the `popover_root` injection prop
  pattern in `registry/ui/date_picker.rs:92`. Verify: `cargo check --locked
  --workspace` compiles the new exports; `rustfmt --edition 2024 --check
  registry/ui/color_picker.rs` passes.
- [x] 1.2 Render the swatch fill via inline `style` from
  `adico_primitives::color_picker::ColorPickerContext::color()`, labelled with
  the existing `color_name()` helper. Verify: an SSR test asserting the
  rendered `style` attribute contains the expected `background-color` for a
  known color, and that the accessible label text matches `color_name`.

## 2. Registry defect fixes

- [x] 2.1 Add `z-50` to `NavigationMenuContent`'s class string in
  `registry/ui/navigation_menu.rs`. Verify: existing navigation-menu tests
  still pass; visual check in task 6.
- [x] 2.2 Remove `z-[1000]` and `adico-date-picker-popover` from
  `DatePickerContent`'s class string in `registry/ui/date_picker.rs:241`.
  Verify: `cargo test -p adico-primitives` (date-picker suite) still passes.
- [x] 2.3 Remove the stale `absolute left-0 top-full` fragment from
  `registry/ui/dropdown_menu.rs:111` and the stale `absolute` fragment from
  `registry/ui/menubar.rs:85`. Update the `#[cfg(test)]` assertion at
  `registry/ui/dropdown_menu.rs:410-412` that pins the old string. Verify:
  `cargo test --locked -p adico-primitives` (or the relevant registry-source
  test target) passes with the updated assertion.
- [x] 2.4 Fix `registry/registry.json`'s `theme-builder` entry:
  `documentation.usage` changes from `"ThemeBuilderLauncher {}"` to
  `"ThemeBuilder {}"`; leave `registryDependencies: ["cn"]` unchanged. Verify:
  `cargo run -p adico-xtask -- registry validate` passes.

## 3. Remove the CSS override that defeats Positioner

- [x] 3.1 Delete the `.adico-date-picker-popover` and
  `.playground-date-picker-popover-root` rules from
  `apps/playground/tailwind.css` (~lines 365-381).
- [x] 3.2 Remove the now-dead `class: "playground-date-picker-popover-root"`
  from `DatePickerPopover` in `apps/playground/src/pages/date_picker.rs:30`.
  Verify: `dx serve` in `apps/playground` rebuilds `assets/tailwind.css`
  without the removed rules (grep the generated file to confirm).

## 4. Propagate registry changes

- [x] 4.1 `cargo run -p adico-xtask -- registry build`; verify
  `registry/generated/items/color-picker.json` (and any touched item) reflects
  the new content.
- [x] 4.2 Reinstall `color-picker`, `navigation-menu`, `date-picker`,
  `dropdown-menu`, `menubar`, and `theme-builder` through the `adico` CLI into
  `apps/playground`, `examples/basic-spa`, `examples/basic-ssr`, and every
  affected `tests/installation/*` fixture (name them: at minimum
  `wave5-color-picker-consumer` for color-picker, `wave4-consumer` for
  date-picker). Never hand-copy; never point a fixture at `registry/` via a
  workspace path. Verify: `git diff` shows only the installed-copy files
  changing, matching the registry source edit.

  Done for: `apps/playground`, `examples/basic-spa`, `examples/basic-ssr`,
  `wave5-color-picker-consumer` (color-picker), `wave4-consumer`
  (date-picker), plus every other fixture actually carrying a touched item
  (`m7-consumer`: navigation-menu, `m8-consumer` and `theme-consumer`:
  dropdown-menu, `wave3-consumer`: dropdown-menu + menubar). `tests/installation/*`
  component copies are gitignored (`tests/installation/.gitignore`), so no
  diff is expected there — reinstalling was still necessary to exercise the
  fix locally.
- [x] 4.3 `cargo run -p adico-xtask -- component-compat sync`,
  `primitive-usage sync`, `styling-usage sync`, `prop-parity sync`. Verify:
  `component-compat check`, `primitive-usage check`, `styling-usage check`,
  `prop-parity check` all pass afterward.

  `styling-usage sync` flagged `color-picker`'s new inline `background-color`
  style; recorded as a `styleException` in
  `statics/styling_usage/color-picker.json` with the reason already stated in
  the spec (a runtime color has no static Tailwind class). All four `check`
  commands pass.

## 5. Local timezone for "today"

- [x] 5.1 Confirmed empirically via a new `#[test]` in
  `packages/adico-primitives/src/lib.rs` (`local_time_tests::now_local_resolves_on_this_native_target_rather_than_falling_back`):
  on this workspace's native macOS target, under `cargo test`'s default
  multithreaded harness, `OffsetDateTime::now_local()` resolves `Ok` — the
  `unsound_local_offset` gate that would force a UTC fallback does not apply
  on this platform. Documented in a comment next to the test: the fallback is
  real (and is what Linux/BSD native builds hit) but not exercised here; the
  browser/wasm path is unaffected either way (resolves via
  `time/wasm-bindgen`).
- [x] 5.2 Made `LocalDateExt::now_local_date()` `pub` in
  `packages/adico-primitives/src/lib.rs`, keeping its existing UTC-fallback
  behavior unchanged. Verify: `cargo check --locked --workspace` — no
  existing caller's behavior changed, only visibility.
- [x] 5.3 Changed `apps/playground/src/pages/calendar.rs:12` from
  `OffsetDateTime::now_utc()` to `OffsetDateTime::now_local_date()` (via
  `use adico_primitives::LocalDateExt as _;`, matching the primitives crate's
  own import convention). Verify: `calendar_page_builds_its_primitive_tree`
  passes; confirmed live in `dx serve` — the flat and popover calendars both
  open on the correct local month/day.

## 6. Playground trigger conversions

- [x] 6.1 Extracted the shared `CalendarView { ... }` subtree into a private
  `CalendarDemoBody` component in `pages/calendar.rs`; added a popover-wrapped
  second instance above the flat one, sharing state, trigger showing the
  formatted selected date (`"{date}"` via `time::Date`'s `Display` impl, a
  `use_memo`) or "Pick a date", `PopoverContent` given `class: "w-auto p-0"`.
  Added `BoolControl { label: "Popover open" }`. Verified live: both
  instances render and stay in sync; the popover opens over the controls
  card; selecting a date in the popover updates the trigger label and the
  flat calendar's highlighted day.
- [x] 6.2 Rebuilt `pages/color_picker.rs` on the task-1 swatch-trigger parts,
  using the generated `ColorPickerPopoverControls`/`ColorPickerPopoverDemoState`
  panel (`playground-controls sync` emitted it once the new props existed) per
  the existing "Playground exposes chosen component controls" requirement's
  generated-panel convention, rather than a hand-written open signal. Verified
  live: the swatch trigger reflects the live color while dragging the hue
  slider.
- [x] 6.3 Changed `pages/theme_builder.rs` to render
  `ThemeBuilderLauncher {}` instead of raw `ui::ThemeBuilder {}` (and dropped
  the now-unneeded `wide: true` / wrapper div, matching how other
  trigger-only demo pages render their trigger directly). Verified live: the
  "Customize theme" trigger opens the same dialog the sidebar footer uses.
- [x] 6.4 Added the open-state `BoolControl` to `pages/sheet.rs`, matching
  `pages/dialog.rs:24`. **Correction found during implementation:**
  `menubar.rs` does NOT get this control — `packages/adico-primitives/src/menubar.rs`'s
  `MenubarProps` has no `open`/`default_open`/`on_open_change` prop at all;
  which menu (if any) is open is private context state
  (`MenubarContext::open_menu`) shared across sibling `MenubarMenu`s, with no
  externally controllable equivalent. Adding one purely to satisfy this task
  would be exactly the playground-convenience-only registry change the
  existing "Registry components are never modified solely for playground's
  convenience" requirement forbids, so `menubar.rs` is left as-is; the spec's
  "Every popup-family route can be forced open from its controls" requirement
  is scoped to components with controllable open state accordingly (see its
  "A component has no controllable open state to expose" scenario). Verified
  live: the Sheet "Open" toggle forces it open.

## 7. Validation and verification

- [x] 7.1 `cargo fmt --all --check` — clean.
- [x] 7.2 `cargo check --locked --workspace` — clean.
- [x] 7.3 `cargo clippy --locked -p adico-cli -p adico-primitives -p
  adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D
  warnings` — clean.
- [x] 7.4 `cargo test --locked -p adico-cli -p adico-primitives -p
  adico-registry-core -p adico-test-utils -p adico-xtask` — all pass (183 +
  91 in the xtask/primitives-heaviest crates, plus the new local-time test).
  Also ran `cargo test -p adico-playground` (both default and
  `--features server`) since two edited files (`registry/ui/dropdown_menu.rs`'s
  test, `pages/calendar.rs`'s SSR test) only compile as part of an installed
  consumer, per this repo's own registry-testing convention — all pass.
- [x] 7.5 `cargo run -p adico-xtask -- registry validate` and `provenance
  check` — both pass (no new provenance record needed — no upstream import
  occurred).
- [x] 7.6 `openspec validate demo-popup-components-behind-triggers --strict`
  — valid.
- [x] 7.7 `dx serve` in `apps/playground`; screenshotted `/calendar`,
  `/date-picker`, `/color-picker`, `/navigation-menu`, `/menubar`, `/sheet`,
  `/theme-builder` live in a real browser. Confirmed: popups paint over the
  controls card, are not clipped, and the DatePicker popup in particular
  (previously CSS-overridden) now anchors correctly with no regression.
- [x] 7.8 Confirmed live via `dx serve` that `/calendar`'s "today" (September
  2026, matching this machine's local clock) is correct; the empirical
  `now_local()` probe in task 5.1 is the mechanical confirmation that this
  isn't coincidentally-correct UTC.
- [x] 7.9 Ran the relevant specs (no spec in this repo covers menubar,
  dropdown-menu, navigation-menu, date-picker, calendar, theme-builder, or
  sheet directly — confirmed by grepping `tests/playwright/*.spec.ts`):
  `wave5-color-picker.spec.ts` against a freshly served
  `wave5-color-picker-consumer` — **2/2 passed**.
  `playground-enriched-demos.spec.ts` against the running playground — 2/4
  passed; the 2 failures are in `Carousel`'s pointer-drag paging, a feature
  this change never touched (confirmed via `git diff` showing zero changes to
  any carousel file) and last modified in the already-merged commit
  `3b73a8d`. Pre-existing, unrelated to this change; not fixed here.

## 8. Close out

- [x] 8.1 No other doc references the old ColorPicker/Calendar/ThemeBuilder
  playground composition or the removed CSS classes (checked via repo-wide
  grep for `adico-date-picker-popover` and `playground-date-picker-popover-root`
  outside the generated Tailwind output, which rebuilds automatically).
- [ ] 8.2 Sync delta specs into `openspec/specs/{adico-existing-components,
  adico-playground-structure,adico-registry}` and archive this change per the
  `openspec-archive-change` workflow — left for the user to trigger
  explicitly (archiving is a deliberate, separate step per this repo's
  OpenSpec workflow, not bundled into implementation).
