# adico-playground-structure Specification

## Purpose
Keep `apps/playground`'s router and page components in a fixed, predictable
location — a dedicated routing module and a file-per-route pages directory
named by TanStack Start's file-based routing convention — so adding a page
for a newly-installed component is always "add one file in the right
place," not an edit to a growing shared file.

## Requirements

### Requirement: Router definitions live in routes.rs
`apps/playground/src/routes.rs` SHALL define the `Route` enum, its
navigation list (`nav_items()` or an equivalent), and the routing `Layout`
shell component. `apps/playground/src/main.rs` SHALL NOT define the `Route`
enum, the navigation list, or the routing shell directly; it retains only
the application entrypoint, its asset consts, and the CLI-managed
`adico:start`/`adico:end` module block.

#### Scenario: main.rs is read for routing logic
- **WHEN** a maintainer opens `apps/playground/src/main.rs` looking for how
  routes or navigation are defined
- **THEN** it contains no `#[derive(Routable)]` enum, no route-list
  function, and no layout-shell component — those live in `routes.rs`

#### Scenario: A route is added or removed
- **WHEN** a page is added to or removed from the playground
- **THEN** the corresponding `#[route(...)]` variant and navigation entry
  are edited in `routes.rs`, not in `main.rs` or scattered across page files

### Requirement: One page per file under pages/, named by route segment
Each routed page component SHALL live in its own file under
`apps/playground/src/pages/`, aggregated by `pages/mod.rs`. Each file SHALL
be named after its route's path segment (snake_case, e.g. `hover_card.rs`
for `/hover-card`), following TanStack Start's file-based routing
convention; a directory's index route (`/`) SHALL use the file name
`index.rs`. No single file under `pages/` SHALL define more than one page
component.

#### Scenario: A new component page is added
- **WHEN** a maintainer adds a playground page for a newly-installed
  registry component
- **THEN** they create exactly one new file under `pages/` named after the
  route's path segment, add its `pub mod`/re-export to `pages/mod.rs`, and
  add its `#[route(...)]` variant to `routes.rs` — no existing page file is
  edited to make room

#### Scenario: The root route is located
- **WHEN** a maintainer looks for the component rendered at `/`
- **THEN** it is defined in `apps/playground/src/pages/index.rs`

#### Scenario: pages.rs is searched for
- **WHEN** a maintainer or tool searches the repository for
  `apps/playground/src/pages.rs`
- **THEN** no such file exists — page components live under the `pages/`
  directory established by this requirement

### Requirement: dioxus-router routes stay explicitly declared
Because dioxus-router has no file-system-based route generation, the
`pages/` file-naming convention SHALL NOT be treated as automatically
producing routes. Every route SHALL remain an explicit `#[route(...)]`
variant in `routes.rs`'s `Route` enum, kept in sync by hand with the files
under `pages/`. A generated `<Component>DemoState`/`<Component>Controls`/
`<Component>Preview` triad under
`apps/playground/src/generated/controls/` likewise SHALL NOT be treated as
automatically producing a page or route — a page still explicitly wires
the generated panel in, and its route remains an explicit `routes.rs`
entry.

#### Scenario: A page file exists with no matching route
- **WHEN** a file exists under `pages/` whose page component has no
  corresponding `#[route(...)]` variant in `routes.rs`
- **THEN** this is a defect to fix by adding the missing route, not
  evidence that routes are derived from the file tree

#### Scenario: A generated control panel exists with no matching page
- **WHEN** `apps/playground/src/generated/controls/` contains a
  `<Component>DemoState`/`<Component>Controls` pair for a component with
  no corresponding page under `pages/`
- **THEN** this is not evidence that a page or route exists — a page must
  still be explicitly created under `pages/` and registered in
  `routes.rs` to actually use the generated panel

### Requirement: Playground shell composes real registry components, not app-specific reimplementations
`apps/playground`'s navigation shell and theme controls SHALL be composed
from installed registry components (`sidebar`, `mode-toggle`,
`theme-switcher`, `theme-builder`) rather than hand-rolled, app-specific
reimplementations of the same behavior. Any playground-specific wiring
needed to compose these components (e.g. a launcher that opens a dialog
containing an installed component) SHALL live under
`apps/playground/src/components/`, SHALL NOT duplicate a registry
component's own logic, and SHALL NOT require any change to
`registry/ui/*.rs` to exist.

#### Scenario: A new theme or navigation need arises in playground
- **WHEN** playground needs new theme-editing or navigation behavior
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through
  the normal registry-item process — playground SHALL NOT gain a
  parallel, app-specific implementation of behavior a registry component
  already provides

#### Scenario: A registry component appears unused in playground source
- **WHEN** a registry component is installed in
  `apps/playground/src/components/ui/` but referenced nowhere in
  playground source
- **THEN** this is a defect to fix (either wire it in or determine it's
  genuinely unneeded and stop installing it) — not a state to leave
  indefinitely, since it signals playground has drifted from the
  components it's meant to demonstrate

### Requirement: Registry components are never modified solely for playground's convenience
A confirmed rendering or behavioral defect in a registry component,
discovered while composing it in `apps/playground`, SHALL be fixed in the
registry source directly. A registry component SHALL NOT be modified,
extended, or given a playground-specific escape hatch (e.g. an `as_child`
prop added only so playground can nest a router `Link`) solely because
playground's current composition approach would otherwise be
inconvenient.

#### Scenario: A registry component lacks a feature playground wants
- **WHEN** a registry component's existing API makes a playground
  composition awkward (e.g. `SidebarMenuButton` always rendering a native
  `<button>` with no way to substitute an `<a>`)
- **THEN** playground SHALL compose around the existing API using ordinary
  Dioxus patterns (e.g. `onclick` plus programmatic navigation) rather
  than the registry component being changed to accommodate playground

#### Scenario: A genuine rendering defect is found while using a real component
- **WHEN** composing an installed registry component in playground
  surfaces behavior that is wrong independent of playground (a real bug,
  not a playground-specific inconvenience)
- **THEN** the fix lands in `registry/ui/*.rs` (or the relevant
  `adico-primitives` module) as its own cited change, verified against a
  live render, not worked around in playground's composition

### Requirement: Popup-family components are demoed via trigger and popup composition
A playground route for a component whose registry facade composes a trigger
and a floating or overlay content part (e.g. Popover, Dialog, Menu, Calendar's
popover-composing siblings) SHALL render that component activated through its
trigger, not pre-opened or rendered as a bare, always-visible surface, so the
demo exercises the same anchoring, focus, and dismissal behavior a consumer
gets. Where a trigger has a natural current-value display (a selected date, a
selected color, an edited theme), the trigger SHALL reflect that value rather
than a static label. A route MAY additionally render the component's
always-visible form alongside the trigger composition when doing so is needed
to demonstrate props that are only meaningful while the surface is visible
(e.g. a calendar's month/year navigation controls).

#### Scenario: A component with only a flat form is converted
- **WHEN** a playground route renders an overlay-family component's content
  unconditionally, with no trigger
- **THEN** this is a defect to fix — the route is updated to open that content
  from a trigger, consistent with every other overlay-family route

#### Scenario: A component needs both forms to demo its own props
- **WHEN** a component (e.g. Calendar) has props that only make sense while
  its surface is visible, and hiding it behind a trigger would prevent
  demoing them
- **THEN** the route MAY keep an always-visible instance for those props and
  additionally add a trigger-driven instance whose trigger shows the current
  value, rather than omitting the trigger composition entirely

#### Scenario: Every popup-family route can be forced open from its controls
- **WHEN** a playground route demos a popup-family component whose underlying
  registry item exposes controllable open state (an `open`/`default_open`/
  `on_open_change` prop, as Popover, Dialog, Sheet, Drawer, AlertDialog,
  DropdownMenu, ContextMenu, Select, Combobox, HoverCard, and Tooltip already
  do)
- **THEN** its "Component controls" panel includes a control to force the
  popup open, consistent with the other such routes

#### Scenario: A component has no controllable open state to expose
- **WHEN** a popup-family component's registry item manages its own open
  state entirely internally, with no `open`/`default_open`/`on_open_change`
  prop to control (for example, `menubar`, whose primitive tracks which of
  its menus is open as private context state shared across sibling menus)
- **THEN** the route is not required to add a non-functional control for it,
  and SHALL NOT gain a controlled-open prop added to the registry item solely
  to satisfy this requirement — that would be a playground-convenience-only
  registry change, which the existing "Registry components are never
  modified solely for playground's convenience" requirement already forbids

### Requirement: Playground demo defaults reflect the local timezone of the running device
Any playground demo default value derived from "now" (e.g. a calendar's
initially visible month, a time picker's initial value) SHALL be computed in
the timezone of the device running the UI, not UTC, using the crate's existing
local-time resolution helpers.

#### Scenario: A user views the Calendar demo from a non-UTC timezone
- **WHEN** the Calendar playground route loads with no date already selected
- **THEN** the calendar's initially visible month is the current month in the
  device's local timezone, not UTC

### Requirement: Navigation lists components in flat alphabetical order
The playground navigation list (`nav_items()`) SHALL present all component
entries as a single flat list ordered alphabetically ascending by displayed
label, with no thematic batches or restarted alphabetical runs. A newly
added component page SHALL be inserted at its alphabetical position.

#### Scenario: A user scans the navigation for a component
- **WHEN** a user opens the playground and scans the navigation sidebar
- **THEN** every component entry appears in one continuous A→Z sequence by
  its displayed label

#### Scenario: A new component page is registered
- **WHEN** a maintainer adds a navigation entry for a new component page
- **THEN** the entry is inserted at its alphabetical position in
  `nav_items()`, not appended at the end

### Requirement: The demo preview zone is pannable with a reset control
The shared demo preview zone SHALL render the demoed component centered by
default and SHALL let the user reposition it by dragging the preview
background. A drag that begins on the demoed component (or its immediate
wrapper) SHALL NOT start a pan, so the component's own pointer interactions
are never intercepted. The preview zone SHALL render a "Center" reset
control at the bottom of the preview area, composed from the installed
Button registry component, that restores the component to the centered
position.

#### Scenario: Dragging the background moves the component
- **WHEN** a user presses on the preview zone's background and drags
- **THEN** the rendered component follows the drag offset and remains at the
  released position

#### Scenario: Dragging the component itself does not pan
- **WHEN** a user begins a drag on the demoed component (e.g. a slider thumb
  or a carousel track)
- **THEN** the preview does not pan and the component receives the pointer
  interaction unchanged

#### Scenario: Center resets the position
- **WHEN** a user activates the Center control after panning
- **THEN** the component returns to the centered default position

### Requirement: Playground demo tooling is composed from installed registry components
The playground's shared demo chrome and control widgets (boolean, text,
number, select, and optional-boolean controls) SHALL be composed from
installed registry components (e.g. Switch, Input, NativeSelect, Label,
Card, Button) rather than raw HTML form elements, to the extent an installed
component provides the needed behavior. The control widgets' public
signatures SHALL remain compatible with the generated control panels that
call them.

#### Scenario: A control widget renders
- **WHEN** a demo page renders a boolean, text, number, or select control
- **THEN** the widget's interactive element is an installed registry
  component, not a hand-styled raw HTML input

#### Scenario: Generated panels keep working
- **WHEN** the control widgets' internals are rebuilt on installed components
- **THEN** every generated `<Component>Controls` panel compiles and behaves
  unchanged, without regenerating or modifying the generator

### Requirement: Demo pages render only the demoed component's real composition
A playground demo page SHALL NOT render extra elements that exist only to
bind a generated control panel and are not part of a realistic composition
of the demoed component (e.g. a standalone pagination link rendered beside a
real pagination row solely to exercise its demo-state panel). When a
generated panel cannot be bound to the realistic composition, the page SHALL
omit that panel rather than invent a demo element for it.

#### Scenario: A generated panel has no realistic binding target
- **WHEN** a generated control panel's props cannot be bound to any element
  of the page's realistic composition
- **THEN** the page omits that panel and renders only the realistic
  composition — no placeholder element is added to host the panel's bindings
