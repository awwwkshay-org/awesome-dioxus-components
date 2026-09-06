## ADDED Requirements

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
