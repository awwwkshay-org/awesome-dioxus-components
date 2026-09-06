## Why

Almost every popup-family registry component (Popover, Tooltip, HoverCard,
DropdownMenu, ContextMenu, Menubar, NavigationMenu, Select, Combobox, Dialog,
AlertDialog, Sheet, Drawer, Command, DatePicker) is already demoed in the
playground the way a consumer actually uses it: a trigger the user activates,
opening a floating surface. Three are not — `pages/calendar.rs`,
`pages/color_picker.rs`, and `pages/theme_builder.rs` render their component
flat and pre-opened, which both misrepresents how consumers install and use
these components and never exercises the anchored-popup rendering path at all.

Auditing that rendering path surfaced a genuine, independent defect: one
overlay (DatePicker) is opted out of the shared `Positioner` anchoring
mechanism by an app-level CSS `!important` override in
`apps/playground/tailwind.css`, making it the only popup that can be clipped
by an ancestor's scroll container and can drift from its trigger. Three
registry component class strings were also found stale relative to how their
content is actually positioned today. These are corrected as part of the same
audit rather than filed separately, since they were found while doing this
work and directly affect whether triggers behave correctly once added.

Separately, `pages/calendar.rs` computes "today" from `OffsetDateTime::now_utc()`
even though the crate already has a local-timezone helper
(`LocalDateExt::now_local_date()`) used nowhere. This makes the playground's
own calendar demo wrong for any user not at UTC.

## What Changes

- `registry/ui/color_picker.rs` gains `ColorPickerPopover`, `ColorPickerTrigger`,
  and `ColorPickerSwatch` — a trigger that renders a filled swatch reflecting
  the live selected color, following the same styled-popover-root injection
  shape `date-picker` already uses.
- `pages/calendar.rs` keeps its existing flat calendar (needed to demo
  Calendar's own props) and gains a second, popover-wrapped instance whose
  trigger shows the formatted selected date.
- `pages/color_picker.rs` is rebuilt on the new swatch-trigger parts.
- `pages/theme_builder.rs` renders the existing `ThemeBuilderLauncher`
  (already used in the sidebar footer) instead of the raw component.
- `pages/sheet.rs` gains the open-state control every other popup-family page
  with controllable open state already has. (`pages/menubar.rs` is
  intentionally excluded: `Menubar`'s primitive has no externally
  controllable open state at all — see design.md.)
- `pages/calendar.rs` switches from `OffsetDateTime::now_utc()` to
  `LocalDateExt::now_local_date()` so "today" reflects the device's local
  timezone.
- **Registry defect fixes**, restoring already-specified anchoring behavior:
  - `registry/ui/navigation_menu.rs`'s `NavigationMenuContent` gains `z-50`,
    matching every sibling overlay.
  - `registry/ui/date_picker.rs`'s `DatePickerContent` drops the dead
    `z-[1000]` and `adico-date-picker-popover` hook class.
  - `apps/playground/tailwind.css`'s `.adico-date-picker-popover` /
    `.playground-date-picker-popover-root` rules (which use `!important` to
    override `Positioner`'s inline `position: fixed`, defeating it for
    DatePicker alone) are removed, along with the now-dead class reference in
    `pages/date_picker.rs`.
  - `registry/ui/dropdown_menu.rs` and `registry/ui/menubar.rs` drop stale
    `absolute`/`absolute left-0 top-full` positioning classes now superseded
    by `Positioner`'s inline `position: fixed`.
- `registry/registry.json`'s `theme-builder` entry's documented `usage` field
  is corrected from a non-existent `ThemeBuilderLauncher {}` (playground-only
  code, not a registry export) to the real export, `ThemeBuilder {}`.
- No breaking changes: every new part is additive, and the class/CSS fixes
  restore intended anchoring behavior rather than changing any component's
  public prop surface.

## Capabilities

### New Capabilities
(none — this change adds requirements to existing capabilities rather than
introducing a new one)

### Modified Capabilities
- `adico-existing-components`: ColorPicker gains a trigger/popover
  composition whose trigger reflects the live selected color, the same
  pattern already required implicitly by how DatePicker composes Popover.
- `adico-playground-structure`: popup-family playground demo pages must
  present their component via a real trigger interaction (not pre-opened,
  flat rendering), and any playground default derived from "now" must use the
  device's local timezone rather than UTC.
- `adico-registry`: a registry item's documented `usage` example must name an
  export the item actually ships, so a consumer can copy it verbatim.

## Impact

- `registry/ui/color_picker.rs`, `registry/ui/navigation_menu.rs`,
  `registry/ui/date_picker.rs`, `registry/ui/dropdown_menu.rs`,
  `registry/ui/menubar.rs`, `registry/registry.json`
- `registry/generated/**` (regenerated via `registry build`)
- `apps/playground/tailwind.css`, `apps/playground/src/pages/{calendar,color_picker,theme_builder,date_picker,menubar,sheet}.rs`
- Installed copies under `apps/playground/src/components/ui/`,
  `examples/basic-{spa,ssr}/src/components/ui/`, and
  `tests/installation/*/src/components/ui/`, refreshed through the `adico`
  CLI install path (never hand-copied)
- `statics/component_compatibility.json`, `statics/primitive_usage/color-picker.json`,
  `statics/styling_usage/color-picker.json`, `statics/prop_parity/color-picker.json`
  (regenerated to reflect the new ColorPicker parts)
- `tests/playwright/wave5-color-picker.spec.ts` (exercises the changed
  color-picker consumer fixture) and `tests/playwright/playground-enriched-demos.spec.ts`
- No CLI, registry schema, or database changes. No new Cargo dependency.
