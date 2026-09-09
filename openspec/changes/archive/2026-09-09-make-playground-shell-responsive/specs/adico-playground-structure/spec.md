## MODIFIED Requirements

### Requirement: Playground shell composes real registry components, not app-specific reimplementations
`apps/playground`'s navigation shell and theme controls SHALL be composed
from installed registry components (`sidebar`, `sheet`, `mode-toggle`,
`theme-switcher`, `theme-builder`, `resizable`) rather than hand-rolled,
app-specific reimplementations of the same behavior. Any playground-specific
wiring needed to compose these components (e.g. a launcher that opens a
dialog containing an installed component) SHALL live under
`apps/playground/src/components/`, SHALL NOT duplicate a registry
component's own logic, and SHALL NOT require any change to
`registry/ui/*.rs` to exist.

At viewports `>= md`, the navigation shell's nav column SHALL be composed
from `sidebar`'s structural sub-components (`SidebarHeader`,
`SidebarContent`, `SidebarGroup`, `SidebarGroupContent`, `SidebarMenu`,
`SidebarMenuItem`, `SidebarMenuButton`, `SidebarFooter`) rather than the
top-level `Sidebar`/`SidebarProvider` orchestration components, since the
latter's width is controlled by fixed CSS variables with no way to
cooperate with a drag-resize handle; the nav column's sizing is instead
composed from the installed `resizable` component (see "The
preview/controls split and the nav/content split are user-resizable",
below). This is not a departure from this requirement — the nav column
still renders no hand-rolled reimplementation of any of these
sub-components' own behavior.

At viewports `< md`, the same nav content (the same shared nav-list
rendering, not a reimplementation of it) SHALL instead be composed inside
the installed `sheet` component, opened by an ordinary click-driven
control — never by measuring or reacting to the viewport at runtime (see
the new "Navigation is reachable and usable below the `md` breakpoint"
requirement, below, for the full behavior contract).

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

#### Scenario: The nav column's sizing mechanism is located
- **WHEN** a maintainer looks for what controls the left nav column's
  width in `Layout` at a viewport `>= md`
- **THEN** it is the installed `resizable` component's `ResizablePanel`,
  not `Sidebar`'s own CSS-variable-driven width, and not a
  playground-specific width calculation

#### Scenario: The nav's presentation mechanism is located, at `< md`
- **WHEN** a maintainer looks for how navigation is presented at a
  viewport `< md`
- **THEN** it is the installed `sheet` component opened from a hamburger
  control, not a narrowed copy of the `>= md` `ResizablePanel`, and not a
  JS-driven viewport check

## ADDED Requirements

### Requirement: Navigation is reachable and usable below the `md` breakpoint
Below the `md` breakpoint, `Layout` SHALL present a mobile top bar (at
minimum: the playground logo/home link and a hamburger control) and SHALL
NOT render the `>= md` `ResizablePanelGroup` nav column, since that
column's minimum width truncates every nav label to an unreadable
fragment at phone widths. Activating the hamburger control SHALL reveal
the full nav list (every entry `nav_items()` produces, with the current
route's active state) in a dismissible overlay. The nav list content
itself SHALL be rendered by the same underlying nav-list rendering used
by the `>= md` tree — not a second, independently maintained copy — so a
route added to `nav_items()` appears in both presentations automatically.
Selecting a nav entry from the overlay SHALL both navigate to that route
and dismiss the overlay. The mechanism selecting between the `< md` and
`>= md` presentations SHALL be pure CSS (breakpoint-scoped utility
classes) — no JavaScript viewport or media-query detection SHALL be used,
consistent with the documented non-functional `document::eval`
viewport-detection pattern this project avoids elsewhere.

#### Scenario: A phone-width viewer opens the playground
- **WHEN** the playground is loaded at a viewport narrower than `md`
- **THEN** the `>= md` resizable nav column is not rendered, a top bar
  with a hamburger control is visible, and no element on the page causes
  horizontal document overflow

#### Scenario: A phone-width viewer opens navigation
- **WHEN** the viewer activates the hamburger control
- **THEN** every entry from `nav_items()` becomes visible, each fully
  legible (not truncated to a fragment of its label), with the current
  route visually indicated as active

#### Scenario: A phone-width viewer picks a page from the overlay
- **WHEN** the viewer selects a nav entry while the overlay is open
- **THEN** the router navigates to that entry's route and the overlay is
  no longer visible

#### Scenario: A route is added to nav_items()
- **WHEN** a maintainer adds a new entry to `nav_items()` for a newly
  routed page
- **THEN** the entry appears in both the `>= md` nav column and the
  `< md` overlay with no additional per-breakpoint wiring, because both
  render the same shared nav-list rendering

### Requirement: The `>= md` layout is unaffected by mobile support
Adding the `< md` mobile presentation SHALL NOT change the markup,
classes, sizing, or behavior of the existing `>= md`
`ResizablePanelGroup` layout (nav column via `resizable`, content panel,
resize handle). The `>= md` tree remains exactly the composition already
required by "Playground shell composes real registry components, not
app-specific reimplementations" and "The preview/controls split and the
nav/content split are user-resizable".

#### Scenario: A desktop-width viewer opens the playground after this change
- **WHEN** the playground is loaded at a viewport `>= md`
- **THEN** the rendered nav column, content panel, and resize handle are
  unchanged from their pre-mobile-support geometry and behavior
