## MODIFIED Requirements

### Requirement: Playground shell composes real registry components, not app-specific reimplementations
`apps/playground`'s navigation shell and theme controls SHALL be composed
from installed registry components (`sidebar`, `mode-toggle`,
`theme-switcher`, `theme-builder`, `resizable`) rather than hand-rolled,
app-specific reimplementations of the same behavior. Any playground-specific
wiring needed to compose these components (e.g. a launcher that opens a
dialog containing an installed component) SHALL live under
`apps/playground/src/components/`, SHALL NOT duplicate a registry
component's own logic, and SHALL NOT require any change to
`registry/ui/*.rs` to exist.

The navigation shell's nav column SHALL be composed from `sidebar`'s
structural sub-components (`SidebarHeader`, `SidebarContent`,
`SidebarGroup`, `SidebarGroupContent`, `SidebarMenu`, `SidebarMenuItem`,
`SidebarMenuButton`, `SidebarFooter`) rather than the top-level `Sidebar`/
`SidebarProvider` orchestration components, since the latter's width is
controlled by fixed CSS variables with no way to cooperate with a
drag-resize handle; the nav column's sizing is instead composed from the
installed `resizable` component (see "The preview/controls split and the
nav/content split are user-resizable", below). This is not a departure from
this requirement — the nav column still renders no hand-rolled
reimplementation of any of these sub-components' own behavior.

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
  width in `Layout`
- **THEN** it is the installed `resizable` component's `ResizablePanel`,
  not `Sidebar`'s own CSS-variable-driven width, and not a
  playground-specific width calculation

## ADDED Requirements

### Requirement: The preview/controls split and the nav/content split are user-resizable
The shared demo preview/controls split (`Demo`) and the shared nav/content
split (`Layout`) SHALL each be composed from the installed `resizable`
registry component (`ResizablePanelGroup`/`ResizablePanel`/
`ResizableHandle`), letting a user drag to adjust the relative size of each
pair. Each split's two sides SHALL enforce a minimum and maximum size so
neither side can be dragged to zero or to fully consume the other. Resized
sizes are NOT required to persist across a full page reload. Because
`Layout` is a persistent router layout (mounted once, not re-mounted by
in-app navigation) while `Demo` is re-mounted fresh by every page
navigation, the nav/content split's resized size MAY persist across
in-app navigation within the same session, while the preview/controls
split's resized size is reset by every page navigation — neither behavior
is a defect.

#### Scenario: A user resizes the preview/controls split
- **WHEN** a user drags the handle between the live component preview and
  the "Component controls" panel
- **THEN** the two areas' relative heights change accordingly, within their
  configured minimum/maximum bounds, on every playground page

#### Scenario: A user resizes the nav/content split
- **WHEN** a user drags the handle between the left navigation column and
  the main content area
- **THEN** the two areas' relative widths change accordingly, within their
  configured minimum/maximum bounds

#### Scenario: A resize handle is dragged to its bound
- **WHEN** a user drags a resize handle past its configured minimum or
  maximum size for either side
- **THEN** that side's size stops at the configured bound rather than
  shrinking to zero or growing to consume all available space

#### Scenario: A full page reload resets both splits
- **WHEN** a user resizes either split and then reloads the browser page
- **THEN** both splits render at their original default sizes

#### Scenario: In-app navigation may preserve the nav/content split but not the preview/controls split
- **WHEN** a user resizes the nav/content split, then clicks a different
  nav item without reloading the browser
- **THEN** the nav/content split MAY keep its resized size (`Layout` is not
  re-mounted by in-app navigation), while the new page's preview/controls
  split renders at its own default sizes (`Demo` is re-mounted fresh per
  page) — neither outcome is a defect
