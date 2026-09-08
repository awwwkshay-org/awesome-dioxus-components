## MODIFIED Requirements

### Requirement: The preview/controls split and the nav/content split are user-resizable
The shared demo preview/controls split (`Demo`) and the shared nav/content
split (`Layout`) SHALL each be composed from the installed `resizable`
registry component (`ResizablePanelGroup`/`ResizablePanel`/
`ResizableHandle`), letting a user drag to adjust the relative size of each
pair. Each split's two sides SHALL enforce a minimum and maximum size so
neither side can be dragged to zero or to fully consume the other; for the
preview/controls split these bounds are preview 60–80% and controls 20–40%
(default 70/30), and for the nav/content split they are nav 12–30% and
content 70–88% (default 18/82). Each panel's seeded size, minimum, and
maximum SHALL come from that panel's own configuration regardless of the
order in which sibling panels within the same group finish mounting — no
panel's initial state may be overwritten by another panel's configuration.
A resize handle's pointer hit target SHALL extend beyond its visible
divider line, so a user can grab it without pixel-precise pointer placement.
Resized sizes are NOT required to persist across a full page reload.
Because `Layout` is a persistent router layout (mounted once, not
re-mounted by in-app navigation) while `Demo` is re-mounted fresh by every
page navigation, the nav/content split's resized size MAY persist across
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

#### Scenario: A split renders at its intended default on every page, regardless of mount order
- **WHEN** a playground page mounts a `Demo` (or `Layout`'s) resizable split
  for the first time
- **THEN** each panel renders at its own coded default size — not a size
  copied from a sibling panel — independent of which panel's setup happens
  to complete first

#### Scenario: A resize handle is grabbable without precise pointer placement
- **WHEN** a user positions the pointer near, but not exactly on, a resize
  handle's visible divider line and presses down
- **THEN** the drag starts, because the handle's pointer hit target is wider
  than its rendered line

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
