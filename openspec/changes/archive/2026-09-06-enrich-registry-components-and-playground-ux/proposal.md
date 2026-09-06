# Proposal: enrich-registry-components-and-playground-ux

## Why

The playground undersells the ecosystem: menu demos render a single item because the
registry facades expose only 4–5 parts while `adico-primitives::menu` already ships
separators, labels, groups, checkbox/radio items, and submenus; Carousel cannot be
dragged (its documented blocker was formally retracted in
`packages/adico-primitives/src/positioner.rs`); InputOTP cannot mask entered values;
Textarea gives no feedback against `max_length`; and the playground shell itself
(raw-HTML controls, unordered 66-item nav, fixed preview zone, a leaked
"Standalone link" generator artifact in the Pagination demo) does not demonstrate
the components it distributes. Consumers evaluating adico see lean demos and miss
capabilities that already exist one layer down.

## What Changes

- **Menu facades**: `dropdown-menu` gains `DropdownMenuGroup`, `DropdownMenuLabel`,
  `DropdownMenuSeparator`, `DropdownMenuShortcut`, `DropdownMenuCheckboxItem`,
  `DropdownMenuRadioGroup`/`DropdownMenuRadioItem`, and
  `DropdownMenuSub`/`SubTrigger`/`SubContent`. `context-menu` and `menubar` gain
  only the context-free four (`Group`, `Label`, `Separator`, `Shortcut`) —
  checkbox/radio/submenu primitives require `MenuContext`, which only the
  dropdown-menu root provides. No primitives menu changes.
- **Carousel drag**: `carousel` gains pointer-drag slide paging (per-element
  pointer events + transient overlay, the `resizable` pattern; not the global
  `pointer.rs` registry). Buttons and keyboard behavior are unchanged.
- **InputOTP masking**: `adico-primitives` `OtpFieldRoot` gains a `mask` prop
  (renders slot inputs as `type="password"`); the `input-otp` registry facade
  threads a matching `mask` prop.
- **Textarea counter**: `textarea` renders a live "N / max" character counter at
  the bottom-right, only when `max_length` is set; without `max_length` the
  rendered DOM is unchanged.
- **Playground demos** rebuilt to realistic compositions: rich account/context/
  File-Edit-View menus, HoverCard profile card, fuller NavigationMenu panels,
  image-style Carousel slides, InputOTP mask toggle + length control, Input
  password show/hide via `input-group` composition (no Input API change), Textarea
  max-length control, Card with `CardAction` + form content, Dialog/Sheet/Drawer
  with real bodies and footers (including a `DrawerDirection` control), a
  dashboard-style Sidebar demo, and removal of the Pagination demo's
  "Standalone link" artifact.
- **Playground shell**: the preview zone becomes pannable (background drag moves
  the rendered component; a Center button resets it; component starts centered);
  the control widgets (`BoolControl`, `TextControl`, `NumberControl`,
  `SelectControl`, `OptionalBoolControl`) and demo chrome are rebuilt on installed
  registry components with unchanged public signatures; the navigation list is
  reordered into one flat alphabetical list.

Non-goals: checkbox/radio/submenu support in ContextMenu/Menubar (needs a
primitives context redesign); a built-in password toggle prop on `Input`; carousel
autoplay/loop/momentum; extending the `playground-controls` generator's prop-type
allowlist (pages hand-roll controls for skipped prop types, per existing
precedent); any docs-app change (it already sorts alphabetically).

No breaking changes: every registry addition is a new optional prop or new
sub-component; existing consumer markup renders identically (Textarea's wrapper
only appears when the new behavior is exercised via `max_length`).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `adico-existing-components`: dropdown-menu/context-menu/menubar composition
  surface (new grouping/label/separator/shortcut parts; dropdown-only
  checkbox/radio/submenu), carousel drag paging, input-otp `mask` prop, textarea
  character counter tied to `max_length`.
- `adico-primitives`: `OtpFieldRoot` gains a `mask` prop that switches slot
  inputs to password rendering.
- `adico-playground-structure`: navigation must list components in flat
  alphabetical order; the preview zone must support background-drag panning with
  a centered initial position and a Center reset button; playground control
  widgets and demo chrome must be composed from installed registry components;
  demo pages must not render generator-binding artifacts that are not part of
  the demoed component.

## Impact

- **Registry source**: `registry/ui/dropdown_menu.rs`, `context_menu.rs`,
  `menubar.rs`, `carousel.rs`, `input_otp.rs`, `textarea.rs` (+ manual checksum
  updates in `registry/registry.json`, regenerated `registry/generated/*`).
- **Primitives**: `packages/adico-primitives/src/otp_field.rs` (new prop + SSR
  test); wasm32 build must stay green.
- **CLI/consumers**: playground reinstall via `adico add --all --replace` after a
  `cargo build -p adico-cli --locked` (registry is embedded); `adico.lock`
  updates. Other consumer fixtures are unaffected (additive source changes only;
  their pinned checksums are refreshed only by the registry manifest, which is
  rebuilt as part of this change).
- **Generated artifacts**: `apps/playground/src/generated/controls/*`
  (`playground-controls sync`), `statics/component_props.json`,
  `statics/primitive_usage/*`, `statics/styling_usage/*`, prop-parity records —
  all CI-gated `check` commands must pass.
- **Playground app**: `apps/playground/src/routes.rs`, `components/demo.rs`,
  `components/controls.rs`, and ~16 files under `pages/`.
- **Tests**: new adico-primitives SSR test (OTP mask); new Playwright specs for
  carousel drag and OTP masking (additive — none exist today).
- No database, configuration, or deployment changes; no public CLI contract
  changes.
