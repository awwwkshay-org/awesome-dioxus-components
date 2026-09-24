## MODIFIED Requirements

### Requirement: Playground shell composes real registry components, not app-specific reimplementations
`PlaygroundLayout`'s navigation shell and theme controls SHALL be composed
from installed registry components (`sidebar`, `sheet`, `mode-toggle`,
`theme-switcher`, `theme-builder`, `resizable`) rather than hand-rolled,
app-specific reimplementations of the same behavior. Any playground-specific
wiring needed to compose these components (e.g. a launcher that opens a
dialog containing an installed component) SHALL live under
`apps/web/src/components/`, SHALL NOT duplicate a registry component's own
logic, and SHALL NOT require any change to `registry/ui/*.rs` to exist.

`PlaygroundLayout` SHALL NOT re-render chrome that `SiteLayout` already
provides for every route — specifically the adico logo and wordmark, the
`ThemeSwitcher`, and the `ModeToggle`. Controls with no equivalent in the site
header, such as the theme-builder launcher, SHALL remain. This applies to both
the `>= md` and `< md` presentations.

The navigation SHALL offer a filter that narrows the visible component
entries by typed text, composed from an installed registry component.

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
the "Navigation is reachable and usable below the `md` breakpoint"
requirement, below, for the full behavior contract).

#### Scenario: A new theme or navigation need arises in the playground
- **WHEN** the playground needs new theme-editing or navigation behavior
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through
  the normal registry-item process — the playground SHALL NOT gain a
  parallel, app-specific implementation of behavior a registry component
  already provides

#### Scenario: A registry component appears unused in playground source
- **WHEN** a registry component is installed in
  `apps/web/src/components/ui/` but referenced nowhere in playground source
- **THEN** this is a defect to fix (either wire it in or determine it's
  genuinely unneeded and stop installing it) — not a state to leave
  indefinitely, since it signals the playground has drifted from the
  components it's meant to demonstrate

#### Scenario: The nav column's sizing mechanism is located
- **WHEN** a maintainer looks for what controls the left nav column's
  width in `PlaygroundLayout` at a viewport `>= md`
- **THEN** it is the installed `resizable` component's `ResizablePanel`,
  not `Sidebar`'s own CSS-variable-driven width, and not a
  playground-specific width calculation

#### Scenario: The nav's presentation mechanism is located, at `< md`
- **WHEN** a maintainer looks for how navigation is presented at a
  viewport `< md`
- **THEN** it is the installed `sheet` component opened from a hamburger
  control, not a narrowed copy of the `>= md` `ResizablePanel`, and not a
  JS-driven viewport check

### Requirement: Navigation is reachable and usable below the `md` breakpoint
Below the `md` breakpoint, `PlaygroundLayout` SHALL present a mobile top bar
(at minimum: the playground logo/home link and a hamburger control) and
SHALL NOT render the `>= md` `ResizablePanelGroup` nav column, since that
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
- **WHEN** `/playground` is loaded at a viewport narrower than `md`
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

### Requirement: Navigation lists components in flat alphabetical order
The playground navigation list (`nav_items()`) SHALL present all component
entries as a single flat list ordered alphabetically ascending by displayed
label, with no thematic batches or restarted alphabetical runs. A newly
added component page SHALL be inserted at its alphabetical position.

When a filter is applied, the entries that remain SHALL keep that same flat
alphabetical order, and clearing the filter SHALL restore the complete list.
A filter SHALL never be the only route to a page.

#### Scenario: A user scans the navigation for a component
- **WHEN** a user opens `/playground` and scans the navigation sidebar
- **THEN** every component entry appears in one continuous A→Z sequence by
  its displayed label

#### Scenario: A user filters the navigation
- **WHEN** a user types into the navigation filter
- **THEN** only entries matching the typed text remain, still in one flat
  A→Z sequence, and clearing the filter restores every entry

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

The pannable area SHALL be visually distinguishable from inert page
background, so a user can tell the space is a workspace rather than empty
margin, even when the demoed component fills very little of it.

#### Scenario: Dragging the background moves the component
- **WHEN** a user presses on the preview zone's background and drags
- **THEN** the rendered component follows the drag offset and remains at the
  released position

#### Scenario: A preview zone holds a component much smaller than itself
- **WHEN** a demoed component occupies a small fraction of the preview zone
- **THEN** the pannable workspace is still identifiable as a workspace rather
  than reading as blank page background

#### Scenario: Dragging the component itself does not pan
- **WHEN** a user begins a drag on the demoed component (e.g. a slider thumb
  or a carousel track)
- **THEN** the preview does not pan and the component receives the pointer
  interaction unchanged

#### Scenario: Center resets the position
- **WHEN** a user activates the Center control after panning
- **THEN** the component returns to the centered default position

### Requirement: The landing page shell composes real registry components, not app-specific reimplementations
The landing page (`/`) content SHALL be composed from installed registry
components (e.g. `Badge`, `Card`, `CopyButton`) rather than hand-rolled,
app-specific reimplementations of the same behavior. Any app-specific
wiring needed to compose these components (e.g. a link component usable
where neither `Button` nor `NavigationMenuLink` can serve) SHALL live under
`apps/web/src/components/`, SHALL NOT duplicate a registry component's own
logic, and SHALL NOT require any change to `registry/ui/*.rs` to exist.

#### Scenario: A new UI need arises on the landing page
- **WHEN** the landing page needs a button, card, or install-command display
  element
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through the
  normal registry-item process — the landing page SHALL NOT gain a
  parallel, app-specific implementation of behavior a registry component
  already provides

The landing page SHALL show real, rendered registry components, not only prose
describing them.

#### Scenario: A visitor lands on the site
- **WHEN** a first-time visitor opens `/`
- **THEN** they can see actual registry components rendered on the page,
  without navigating to another route
