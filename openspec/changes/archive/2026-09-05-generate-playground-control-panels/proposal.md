## Why

Measured directly against the current tree: of adico's 66 registry items,
**7 have no playground page at all** (`attachment`, `bubble`, `data-table`,
`marker`, `message`, `message-scroller`, `theme-builder` — 6 of these
aren't even installed into `apps/playground/src/components/ui/` yet;
`theme-builder` is installed but has no page). Of the 59 existing pages,
**23 render zero live controls** (`breadcrumb`, `carousel`, `checkbox`,
`collapsible`, `color_picker`, `command`, `drag_and_drop_list`,
`input_otp`, `kbd`, `label`, `mode_toggle`, `navigation_menu`,
`radio_group`, `resizable`, `scroll_area`, `slider`, `spinner`, `table`,
`tag_group`, `theme_switcher`, `toast`, `toolbar`, `virtual_list`) and the
other 36 hand-write their own `use_signal` + `BoolControl`/`TextControl`/
`SelectControl` wiring per page — boilerplate that has already drifted
twice (the motivating Badge/Item bugs behind the archived
`generate-playground-controls-from-props` change) and will drift further
as `complete-component-prop-surface` (Change B) adds `radius`/`loading`
and closes 228 real prop gaps across 41 items.

The existing `cargo xtask playground-controls sync|check|diff`
(`packages/adico-xtask/src/playground_controls.rs`, from the archived
change) only generates enum-typed option-list constants — 16 files under
`apps/playground/src/generated/controls/` today — leaving the
`use_signal` declarations, the control JSX, and the live-preview wiring
entirely hand-written per page. That archived change's own design.md
explicitly deferred "generate the entire `controls: rsx! { ... }` block"
and "wire controls into the 25 currently-uncontrolled pages," calling out
that doing so needed a real answer for which pages have a composition
simple enough to infer from a props struct — a question this change now
answers with real data (measured single-vs-multi-part composition per
item, not assumed).

`apps/playground/src/components/controls.rs` also only has two control
shapes with a working generic signature (`BoolControl`/`TextControl` take
`Signal<T>`; `SelectControl` takes `value` + `on_change`, a different,
one-way convention) and no numeric or optional-text control at all —
`Slider`/`Progress`/`Resizable` have no live controls today specifically
because there is nothing to render a numeric prop with.

## What Changes

- **Two new control primitives** in
  `apps/playground/src/components/controls.rs`: `NumberControl` (for the
  numeric props Change B's waves and existing components both have — e.g.
  `Slider`'s `min`/`max`/`step`, `Progress`'s `value`/`max`) and a
  tri-state `OptionalBoolControl` (`Uncontrolled`/`On`/`Off`) for the
  optional-controlled-boolean idiom `pages/sidebar.rs` and `pages/select.rs`
  already hand-write today. `SelectControl`'s one-way `value`/`on_change`
  signature is unified onto `BoolControl`/`TextControl`'s `Signal<T>`
  convention, since a generator must emit one calling convention, not two.
- **Extend `cargo xtask playground-controls sync|check|diff`** to emit,
  per component with at least one controllable prop, into
  `apps/playground/src/generated/controls/<item>.rs`: a `pub struct
  <Comp>DemoState` (one field per controllable prop) + `Default`; a
  `#[component] pub fn <Comp>Controls(state: Signal<<Comp>DemoState>)`
  rendering the whole panel; and, for single-root components only (see
  below), a `#[component] pub fn <Comp>Preview(state: <Comp>DemoState,
  children: Element)` invoking the real component with every field. Keeps
  the existing compile-time exhaustiveness guard and `rustfmt`-canonical
  output.
- **Generated `Preview` is scoped to single-root, non-generic components
  only** — measured, not assumed: of the 66 items, **16** expose exactly
  one public, non-generic root component (`aspect-ratio`, `badge`,
  `button`, `input`, `label`, `mode-toggle`, `progress`, `scroll-area`,
  `skeleton`, `spinner`, `switch`, `textarea`, `theme-builder`,
  `theme-switcher`, `toggle`, `virtual-list`) — these get a generated
  `Preview`. The other 50 (multi-part composed items like `Dialog`/
  `Select`/`Sidebar`/`DataTable`, and any generic component like
  `Select<T>`/`Combobox<T>`/`Command<T>`/`TagGroup<T>`/`DropdownMenuItem<T>`)
  keep a hand-written preview, excluded with a recorded reason exactly
  like an unsupported prop shape already is.
- **Full coverage**: install the 6 not-yet-installed items into
  `apps/playground` through the real CLI path, add all 7 missing pages
  (route, `nav_items()` entry, `pages/mod.rs` export), wire generated
  panels into the 23 currently-uncontrolled pages, and convert the 36
  pages with hand-written controls to use the generated panel. Target:
  66/66 pages, every one backed by a generated `DemoState`/`Controls`.
- **A second consumer for the metadata**: `apps/docs` (which already
  reads `registry/registry.json` at compile time but renders no props
  table) gains a per-component props table rendered from the same
  introspected prop data `playground_controls.rs` already extracts.
- **Non-goal, restated from the archived change and still true**: no
  runtime/JSON-driven control renderer — Dioxus RSX component invocations
  are statically typed (`Comp { variant: <expr> }` requires `<expr>:
  CompVariant` at compile time), so generated *compiled* Rust is the only
  viable mechanism, not a new one invented for this change.

## Capabilities

### Modified Capabilities

- `adico-playground-demo-controls`: today's requirement covers only
  generated *enum option lists*; extends to cover the full generated
  `DemoState`/`Controls`/`Preview` triad and the single-root-only scoping
  rule for `Preview`.
- `adico-playground-structure`: today's requirement establishes that
  `pages/` file naming does not auto-generate routes; extends to record
  that a generated `Controls`/`Preview` pair does not imply a generated
  page either — the route, nav entry, and page composition stay
  hand-written and explicitly wire the generated panel in.
- `adico-existing-components`: its "Playground exposes chosen component
  controls" requirement, which already requires controls to "remain
  strongly typed by the route's Dioxus state rather than attempting
  runtime reflection," names generated `DemoState`/`Controls` as the
  compliant mechanism satisfying that requirement at full 66/66 coverage,
  rather than each page separately proving it by hand.

## Impact

- `apps/playground/src/components/controls.rs`: two new control
  primitives, `SelectControl`'s signature unified onto `Signal<T>`.
- `packages/adico-xtask/src/playground_controls.rs`: extended classifier
  (numeric, `Option<String>`, tri-state-optional-bool shapes newly
  supported) and the `DemoState`/`Controls`/`Preview` codegen.
- `apps/playground/src/generated/controls/*.rs`: regenerated for every
  component with a controllable prop (up from 16 files to a number
  determined by the actual controllable-prop count once Change B's prop
  additions have landed — measured at implementation time, not guessed
  here).
- `apps/playground/src/pages/*.rs` (all 66: 7 new, 59 converted),
  `apps/playground/src/routes.rs`, `apps/playground/src/pages/mod.rs`,
  `apps/playground/src/components.json`/`adico.lock` (6 new installs).
- `apps/docs/src/*`: new props-table rendering.
- No change to `adico-cli` or `adico-registry-core`.
