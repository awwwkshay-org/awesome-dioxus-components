# M3 existing-component migration queue

Status: accepted for implementation

This turns the M1 upstream inventory
([`../../upstreams/dioxus-components/inventory.md`](../../upstreams/dioxus-components/inventory.md))
into an ordered migration queue for every upstream item classified suitable
for reuse. It satisfies task 4.1: every item has a migration decision —
registry item type, target source file, theme assets, Cargo dependencies, and
platform limitations — ordered by reusable primitive availability so early
batches require no new `adico-primitives` work and later batches build on
primitives imported by earlier ones.

`navbar` (`NEEDS_PARITY_UPDATES`) is excluded from this queue by the M1
record: its upstream composition/naming differs enough from shadcn's
navigation-menu that it needs its own audit rather than a routine migration
decision, and is scheduled separately in M4+.

All items use registry item type `registry:ui` (styled, composable source) and
target `registry/ui/<name>.rs`, matching the M2 vertical slice. Registry item
names are kebab-case (the schema forbids underscores), so upstream's
`snake_case` module names are translated at the item-name boundary only —
e.g. `alert_dialog` installs as `@adico/alert-dialog`. Every item's theme
requirement defaults to `semanticTokens: true, radiusToken: true,
utilities: ["cn"]`, matching Button/Dialog/Select, unless noted otherwise.
Every item's Cargo requirement always includes `dioxus =0.7.9`; only
additional requirements are listed below.

Unless noted, platform expectation for every item below is: **web required
for initial parity**; SSR-safety and desktop capability are proven per item
following the target-gated adapter pattern already used by
`adico-primitives` (`#[cfg(any(feature = "web", feature = "desktop"))]`,
target-gated `time`/`gloo-timers`), and are recorded progressively in
`parity.json` rather than assumed. Any item whose upstream primitive uses
`document::eval`, a portal, or measurement APIs (ResizeObserver-equivalent)
needs that adapter work before desktop/SSR can be claimed — this repeats the
exact gap already recorded for Dialog/Select's desktop feature selection
(see [`m2-vertical-slice.md`](m2-vertical-slice.md)), not a new kind of risk.

## Wave 1 — zero new primitive (reuses only what M2 already owns)

Independent of any not-yet-migrated item; installable as soon as task 4.3
begins. `sheet` composes the already-owned `dialog` primitive directly
(upstream groups `dialog`, `sheet`, and `alert_dialog` under one "Dialog,
Sheet, Alert Dialog foundation," but only `sheet` has no dedicated upstream
primitive source file — `alert_dialog.rs` exists upstream and is deferred to
Wave 2).

| Item | Registry name | Cargo (beyond `dioxus`) | Notes |
| --- | --- | --- | --- |
| badge | `badge` | none | Pure styled source, no primitive. |
| card | `card` | none | Pure styled source, no primitive. |
| input | `input` | none | Pure styled source, no primitive. |
| item | `item` | none | Uses only the shared attribute-forwarding pattern Button already proves. |
| pagination | `pagination` | none | Pure styled source, no primitive. |
| skeleton | `skeleton` | none | Pure styled source, no primitive; theme: no `cn` utility needed beyond base tokens. |
| textarea | `textarea` | none | Pure styled source, no primitive. |
| sheet | `sheet` | `adico-primitives =0.1.0` (`web`/`desktop` features, matching Dialog) | Composes the owned `dialog` primitive under a slide-in styled variant; registry dependency on `cn` only, no new registry-dependency edge on `dialog` itself since the primitive is a Cargo dependency, not a registry item. |

## Wave 2 — single self-contained new primitive (task 4.3, continued)

Each needs one upstream primitive module ported into `adico-primitives`
(task 4.2), but none needs shared overlay/positioning/collection
infrastructure — each is independent of every other Wave 2 item.

| Item | Registry name | Upstream primitive source | Notes |
| --- | --- | --- | --- |
| accordion | `accordion` | `src/accordion.rs` | Keyboard-navigable collapsible group; SSR-safe expected (no DOM interop). |
| alert-dialog | `alert-dialog` | `src/alert_dialog.rs` | Own primitive despite the shared "dialog foundation" note; confirm at import time whether it can delegate to the owned `dialog` primitive's focus-trap/escape internals instead of duplicating them. |
| aspect-ratio | `aspect-ratio` | `src/aspect_ratio.rs` | CSS-ratio wrapper; minimal/no JS behavior. |
| avatar | `avatar` | `src/avatar.rs` | Image-load-state primitive (loading/error fallback). |
| checkbox | `checkbox` | `src/checkbox.rs` | Keyboard/ARIA checkbox state. |
| collapsible | `collapsible` | `src/collapsible.rs` | Single-panel open/close state; shares shape with Accordion but ships as its own primitive upstream. |
| label | `label` | `src/label.rs` | `for`/`id` association helper. |
| progress | `progress` | `src/progress.rs` | ARIA progressbar semantics. |
| radio-group | `radio-group` | `src/radio_group.rs` | Roving-focus radio semantics. |
| scroll-area | `scroll-area` | `src/scroll_area.rs` | Custom scrollbar styling; verify SSR fallback renders native scroll. |
| separator | `separator` | `src/separator.rs` | `role="separator"`/orientation only. |
| slider | `slider` | `src/slider.rs`, `src/pointer.rs`, `src/move_interaction.rs` | Pointer-drag value primitive; desktop pointer-capture behavior needs explicit verification (Dioxus desktop pointer events differ from web). |
| switch | `switch` | `src/switch.rs` | Keyboard/ARIA switch state. |
| tabs | `tabs` | `src/tabs.rs` | Roving-focus tab list, matches Select's typeahead-adjacent keyboard patterns already proven. |
| toast | `toast` | `src/toast.rs` | Queue/timeout-driven visibility; target-gate the timeout adapter the same way `adico-primitives::time` already does for wasm vs. native. |
| toggle | `toggle` | `src/toggle.rs` | Pressed-state ARIA button. |
| toggle-group | `toggle-group` | `src/toggle_group.rs` | Roving-focus group of `toggle`. |

## Wave 3 — overlay/layer-dependent (task 4.4)

Upstream's own grouping ("Layered/positioned menus and overlays"). All six
depend on `src/portal.rs` and upstream's `document::eval`-based positioning,
neither of which is in `adico-primitives` yet. This wave should not start
until a shared portal + positioning + dismissable-layer primitive lands
(task 4.2 for this wave specifically, ahead of M6's broader positioning work,
since these six are needed for existing shadcn parity rather than a new
missing component).

| Item | Registry name | Upstream primitive source | Notes |
| --- | --- | --- | --- |
| context-menu | `context-menu` | `src/context_menu.rs`, `src/portal.rs` | Right-click-triggered layer; reuses Dialog-adjacent dismiss/escape behavior plus positioning. |
| dropdown-menu | `dropdown-menu` | `src/dropdown_menu.rs`, `src/portal.rs` | Click-triggered layer with roving-focus menu items. |
| hover-card | `hover-card` | `src/hover_card.rs`, `src/portal.rs` | Hover/focus-delay-triggered layer. |
| menubar | `menubar` | `src/menubar.rs`, `src/portal.rs` | Multiple coordinated dropdown-menu-like layers with roving focus across the bar. |
| popover | `popover` | `src/popover.rs`, `src/portal.rs` | Foundation shape for hover-card/hover interactions; consider porting first in this wave since others may compose it. |
| tooltip | `tooltip` | `src/tooltip.rs`, `src/portal.rs` | Hover/focus-delay-triggered layer, typically the simplest positioning consumer — a reasonable first item in this wave. |

## Wave 4 — collection/selection/navigation-dependent (task 4.5)

| Item | Registry name | Upstream primitive source | Cargo (beyond `dioxus`) | Notes |
| --- | --- | --- | --- | --- |
| combobox | `combobox` | `src/combobox/**`, reuses `src/collection.rs`, `src/listbox.rs`, `src/selectable.rs`, `src/selection.rs` (already imported for Select — import with its dependent slice rather than duplicate, per the M1 record) | `dioxus-icons =0.1.0` | Filterable/typeahead variant of Select's already-owned collection machinery. |
| calendar | `calendar` | `src/calendar.rs` | `dioxus-icons =0.1.0`, `time =0.3.44` | Date-grid primitive; matches the M0 toolchain decision's Calendar dependency note. |
| date-picker | `date-picker` | `src/date_picker.rs` | none beyond Calendar's | Registry-dependent on `calendar`; no separate primitive beyond composing it. |
| sidebar | `sidebar` | no dedicated primitive file; composes shared `use_controlled` state and portal/positioning types | `adico-primitives =0.1.0` | Off-canvas layout, not a floating layer; depends on the same `use_controlled` public-facade candidate noted in M1, and on Wave 3's positioning primitive for its overlay (mobile/sheet) mode. |

## Wave 5 — Dioxus-only extras (task 4.6, not shadcn parity)

`EXISTING_DIOXUS_EXTRA` items. Migrated with the same provenance/registry
rigor as shadcn-equivalents, but registry/docs metadata must label them
explicitly as extras so they never count toward shadcn parity credit
(task 4.6's verification requirement).

| Item | Registry name | Upstream primitive source | Cargo (beyond `dioxus`) | Notes |
| --- | --- | --- | --- | --- |
| color-picker | `color-picker` | `src/color_picker.rs`, `src/color_picker/color_naming.rs` | `adico-primitives =0.1.0` | Composes `label`, `popover`, and `slider` — depends on Wave 2 (slider, label) and Wave 3 (popover); migrate last within this wave. |
| drag-and-drop-list | `drag-and-drop-list` | `src/drag_and_drop_list.rs` | `adico-primitives =0.1.0` | Pointer-drag list reordering; desktop pointer-capture behavior unverified, same open question as Slider. |
| form | `form` | none (no dedicated upstream primitive file; styled composition only) | none | Could plausibly migrate in Wave 1 by primitive availability, but stays in this wave so its extras labeling ships with a deliberate registry/docs decision rather than slipping in as if it were shadcn parity. |
| tag-group | `tag-group` | `src/tag_group.rs` | none beyond primitive | Grouped multi-select tag input. |
| toolbar | `toolbar` | `src/toolbar.rs` | none beyond primitive | Roving-focus toolbar container, similar shape to Menubar without the layered popups. |
| virtual-list | `virtual-list` | `src/virtual/**`, `src/virtual_list.rs` | none beyond primitive | Needs viewport/scroll measurement; web-only until an observer-bridge primitive exists (M6 territory), document as a target-support skip rather than a silent omission until then. |

## Verification

Every one of the 41 items above (45 total upstream styled components, minus
Button/Dialog/Select already migrated in M2, minus `navbar` handled
separately) has a row with a registry item type, target source file,
Cargo dependency set, and platform note — satisfying task 4.1's "every
suitable upstream item has a migration decision" requirement. `cargo xtask
registry validate` and the parity dimensions in `parity.json` are the
ongoing enforcement mechanism as each wave actually migrates (tasks 4.3–4.9).
