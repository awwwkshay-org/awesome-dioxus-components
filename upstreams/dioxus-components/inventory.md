# Dioxus Components upstream inventory

Audited source: <https://github.com/DioxusLabs/dioxus-components> at
`bf007c15d0cf4d04d3181cc46cf12325aa773955` (2026-08-24).

The upstream workspace contains `dioxus-primitives`, a `preview` application
with source-install metadata, `dioxus-attributes`, a test harness, and a
Playwright suite. It is dual licensed MIT OR Apache-2.0. Its preview metadata
generally declares `dioxus-primitives` and the shared
`preview/assets/dx-components-theme.css`; Select, Combobox, and Calendar also
declare `dioxus-icons =0.1.0`, while Calendar declares `time =0.3.44`.

The Dialog/Select primitive closure has been imported into `adico-primitives`
from this exact revision. Its 17 source units, retained license material, and
local adapter changes are recorded in
`provenance/records/adico-primitives-dialog-select.json`. The remaining
inventory is audit input only and has not been imported.

## Styled component inventory

`Primitive dependency` records the top-level primitive or shared primitive API
referenced directly by the upstream styled source. `Browser test` is a direct
upstream Playwright spec at this revision, not a claim of adico parity.

| Upstream item | Classification | shadcn mapping | Primitive dependency | Browser test |
| --- | --- | --- | --- | --- |
| accordion | EXISTING_SHADCN_EQUIVALENT | accordion | accordion | yes |
| alert_dialog | EXISTING_SHADCN_EQUIVALENT | alert-dialog | alert_dialog | yes |
| aspect_ratio | EXISTING_SHADCN_EQUIVALENT | aspect-ratio | aspect_ratio | no direct spec |
| avatar | EXISTING_SHADCN_EQUIVALENT | avatar | avatar, shared attributes | yes |
| badge | EXISTING_SHADCN_EQUIVALENT | badge | none | no direct spec |
| button | EXISTING_SHADCN_EQUIVALENT | button | shared attributes | no direct spec |
| calendar | EXISTING_SHADCN_EQUIVALENT | calendar | calendar | yes |
| card | EXISTING_SHADCN_EQUIVALENT | card | none | no direct spec |
| checkbox | EXISTING_SHADCN_EQUIVALENT | checkbox | checkbox | yes |
| collapsible | EXISTING_SHADCN_EQUIVALENT | collapsible | collapsible, shared attributes | yes |
| color_picker | EXISTING_DIOXUS_EXTRA | none | color_picker, label, popover, slider | yes |
| combobox | EXISTING_SHADCN_EQUIVALENT | combobox | combobox | yes |
| context_menu | EXISTING_SHADCN_EQUIVALENT | context-menu | context_menu | yes |
| date_picker | EXISTING_SHADCN_EQUIVALENT | date-picker | calendar | no direct spec |
| dialog | EXISTING_SHADCN_EQUIVALENT | dialog | dialog, shared attributes | yes |
| drag_and_drop_list | EXISTING_DIOXUS_EXTRA | none | drag_and_drop_list | yes |
| dropdown_menu | EXISTING_SHADCN_EQUIVALENT | dropdown-menu | dropdown_menu, shared attributes | yes |
| form | EXISTING_DIOXUS_EXTRA | none | none | no direct spec |
| hover_card | EXISTING_SHADCN_EQUIVALENT | hover-card | hover_card | yes |
| input | EXISTING_SHADCN_EQUIVALENT | input | none | yes |
| item | EXISTING_SHADCN_EQUIVALENT | item | shared attributes | no direct spec |
| label | EXISTING_SHADCN_EQUIVALENT | label | label | no direct spec |
| menubar | EXISTING_SHADCN_EQUIVALENT | menubar | menubar | yes |
| navbar | NEEDS_PARITY_UPDATES | navigation-menu candidate | navbar | yes |
| pagination | EXISTING_SHADCN_EQUIVALENT | pagination | none | no direct spec |
| popover | EXISTING_SHADCN_EQUIVALENT | popover | popover | yes |
| progress | EXISTING_SHADCN_EQUIVALENT | progress | progress | no direct spec |
| radio_group | EXISTING_SHADCN_EQUIVALENT | radio-group | radio_group | yes |
| scroll_area | EXISTING_SHADCN_EQUIVALENT | scroll-area | scroll_area | no direct spec |
| select | EXISTING_SHADCN_EQUIVALENT | select | select, shared attributes | yes |
| separator | EXISTING_SHADCN_EQUIVALENT | separator | separator | no direct spec |
| sheet | EXISTING_SHADCN_EQUIVALENT | sheet | dialog, shared attributes | yes |
| sidebar | EXISTING_SHADCN_EQUIVALENT | sidebar | shared controlled state, positioning types | yes |
| skeleton | EXISTING_SHADCN_EQUIVALENT | skeleton | none | no direct spec |
| slider | EXISTING_SHADCN_EQUIVALENT | slider | slider | yes |
| switch | EXISTING_SHADCN_EQUIVALENT | switch | switch | yes |
| tabs | EXISTING_SHADCN_EQUIVALENT | tabs | tabs | yes |
| tag_group | EXISTING_DIOXUS_EXTRA | none | tag_group | yes |
| textarea | EXISTING_SHADCN_EQUIVALENT | textarea | none | no direct spec |
| toast | EXISTING_SHADCN_EQUIVALENT | toast | toast | yes |
| toggle | EXISTING_SHADCN_EQUIVALENT | toggle | toggle | yes |
| toggle_group | EXISTING_SHADCN_EQUIVALENT | toggle-group | toggle_group | yes |
| toolbar | EXISTING_DIOXUS_EXTRA | none | toolbar | yes |
| tooltip | EXISTING_SHADCN_EQUIVALENT | tooltip | tooltip, shared attributes | yes |
| virtual_list | EXISTING_DIOXUS_EXTRA | none | virtual_list | yes |

The `EXISTING_SHADCN_EQUIVALENT` classification is a reuse candidate only.
M3/M4 must still audit source-owned installation, composition/API, current
shadcn styling/variants, keyboard behavior, accessibility, theming, and target
support before parity can pass. `navbar` is a navigation-menu candidate but has
different upstream naming/composition and therefore enters M4 as
`NEEDS_PARITY_UPDATES`.

## Primitive inventory

### Public modules

| Module | Primary consumers or role |
| --- | --- |
| accordion, alert_dialog, aspect_ratio, avatar, checkbox, collapsible | Same-named accessible component primitives |
| calendar, date_picker | Date grid/selection and date-picker behavior |
| color_picker | Color value and picker behavior |
| combobox, select | Collection, filtering/typeahead, option selection, triggers/lists |
| context_menu, dropdown_menu, hover_card, menubar, popover, tooltip | Layered/positioned menus and overlays |
| dialog | Dialog, Sheet, Alert Dialog foundation |
| drag_and_drop_list | Dioxus-specific draggable list behavior |
| label, progress, radio_group, scroll_area, separator, slider, switch, tabs, toast, toggle, toggle_group, toolbar | Same-named reusable behavior |
| navbar | Navigation UI behavior with router feature |
| tag_group | Dioxus-specific grouped tag selection |
| virtual_list | Virtualized-list public API |

### Shared/internal modules and APIs

| Module/API | Dependents and reuse assessment |
| --- | --- |
| `collection`, `listbox`, `selectable`, `selection` | Select/Combobox collections and selection; import with their dependent slice rather than duplicate. |
| `pointer`, `move_interaction` | Slider and drag interactions; retain module layout initially. |
| `portal` | Overlay primitives; preserve/replace only after portal tests exist. |
| `virtual` | Virtual-list calculations; isolate to virtual-list scope. |
| `merge_attributes`, `dioxus_attributes` | Styled-source attribute forwarding; required by Button, Dialog, and many migrated items. |
| `use_controlled` | Controlled/uncontrolled state used by Color Picker and Sidebar; public facade candidate. |
| focus-trap JS bundle, global Escape/outside-dismiss helpers | Dialog/layer behavior; requires explicit web/SSR audit before import. |

## Dependency and compatibility findings

- Upstream pins `dioxus =0.7.8`; the adico workspace pins `=0.7.9`. M1 must
  compile each imported slice against adico's pin before it is considered
  reusable.
- `dioxus-attributes` is a small proc-macro crate used by styled source through
  `dioxus-primitives::dioxus_attributes`; include it in the fork only if the
  initial attribute API cannot be replaced without source API churn.
- Upstream uses browser `document::eval` and a bundled focus-trap JavaScript
  asset in shared primitive helpers. These are not automatically accepted for
  SSR/desktop; target-gated adaptation is required by M1.7.
- Upstream's preview uses `dioxus-icons =0.1.0`, matching adico's M0 icon
  decision. Its Playwright package already uses Playwright and axe, making the
  test organization a useful source of patterns but not an adico dependency.

## Current shadcn comparison and first slice

`../shadcn/catalog.json` is the checked-in first-party snapshot at
`ac60ef5c4db4265d71454dd9ecd3f93e255d7211`. The direct mappings above establish
the inherited component set; all snapshot entries absent from that mapping are
future gap candidates and must be grouped by primitives after M4, not implemented
alphabetically.

The first vertical slice is **Button, Dialog, and Select**:

1. Button proves simple source, styling, attributes, module exports, and Cargo
   plumbing.
2. Dialog proves owned overlay/focus/escape/outside-dismiss behavior.
3. Select proves icons, collections, typeahead, selection, keyboard handling,
   transitive primitive dependencies, and upstream browser-test reuse.
