> **Rejected (2026-09-07)** — this change (and this delta) was reverted
> before archiving; see `proposal.md`'s and `design.md`'s "Status: Rejected"
> sections. Nothing below reflects this app's current or intended behavior.

## MODIFIED Requirements

### Requirement: Navigation lists components in flat alphabetical order
The playground navigation list (`nav_items()`) SHALL present all top-level
component entries as a single flat list ordered alphabetically ascending by
displayed label, with no thematic batches or restarted alphabetical runs. A
newly added component page SHALL be inserted at its alphabetical position.

An item whose page renders more than one labeled control group MAY
additionally list those groups as nested entries beneath its top-level
entry, in the order the page's control panel renders them (not
alphabetical) — this ordering reflects the panel layout, not the top-level
alphabetical rule, which governs only the top-level list. The nested list
SHALL be revealed by the current route (the user being on that item's own
page or on one of its part routes), not by independent expand/collapse
state tracked outside the route.

#### Scenario: A user scans the navigation for a component
- **WHEN** a user opens the playground and scans the navigation sidebar's
  top-level entries
- **THEN** every top-level component entry appears in one continuous A→Z
  sequence by its displayed label

#### Scenario: A new component page is registered
- **WHEN** a maintainer adds a navigation entry for a new component page
- **THEN** the entry is inserted at its alphabetical position among
  top-level entries in `nav_items()`, not appended at the end

#### Scenario: A multi-group item's nav entry reveals its parts
- **WHEN** a user navigates to an item whose page renders more than one
  control group (for example `/select`)
- **THEN** that item's nav entry reveals its parts nested beneath it, in
  the same order those groups appear in the control panel, and the nesting
  disappears again when the user navigates to an unrelated item

### Requirement: One page per file under pages/, named by route segment
Each routed page component SHALL live in its own file under
`apps/playground/src/pages/`, aggregated by `pages/mod.rs`. Each file SHALL
be named after its route's path segment (snake_case, e.g. `hover_card.rs`
for `/hover-card`), following TanStack Start's file-based routing
convention; a directory's index route (`/`) SHALL use the file name
`index.rs`. A route with dynamic path segments SHALL be named after those
segments (e.g. `component_part.rs` for `#[route("/:item/:part")]`). No
single file under `pages/` SHALL define more than one page component — a
page component that dispatches among several existing page components by
matching a route parameter is still exactly one page component and
satisfies this rule.

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

#### Scenario: A dynamic-segment dispatcher page is located
- **WHEN** a maintainer looks for the page handling `/:item/:part` routes
- **THEN** it is a single file, `apps/playground/src/pages/component_part.rs`,
  defining exactly one page component that dispatches by matching its route
  parameters — not one file per dispatched item

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

A dynamic route's mapping from a route parameter to the page component it
dispatches to SHALL be an explicit, hand-maintained match — never derived
automatically from the `pages/` directory tree or from generated control
panels. A nav entry's list of nested parts SHALL likewise be
hand-maintained and verified against the actual rendered page source by an
automated test, never generated from the registry's full component surface
— the registry surface is known to diverge from what a page's composition
actually renders (see "A page is part-routed only when all its controls
are grouped").

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

#### Scenario: A nav entry's parts list drifts from its page
- **WHEN** a wired item's page gains, loses, or renames a `ControlGroup`
  and its nav entry's hand-maintained parts list is not updated to match
- **THEN** an automated test fails, naming the mismatch — the drift is
  never silently accepted

## ADDED Requirements

### Requirement: Each control group a page renders can have its own dedicated route
A part-routed item's subcomponent route SHALL render that item's real,
unmodified composition — the same one its unscoped page renders — with the
control panel scoped to exactly the one group that route names. It SHALL
provide a way back to the item's unscoped, all-groups page. An item/part
combination that does not resolve to a real, wired group SHALL render an
explicit not-found state, never a crash and never a silently different
composition than the item's real one.

#### Scenario: A subcomponent route renders the real composition
- **WHEN** a user navigates to a wired item's subcomponent route (for
  example `/select/select-trigger`)
- **THEN** the same live, interactive Select composition renders as on
  `/select`, and the control panel shows only the "Select Trigger" group

#### Scenario: A subcomponent route links back to the full view
- **WHEN** a user is on a subcomponent route
- **THEN** the page offers a way back to that item's unscoped route showing
  every group

#### Scenario: An unmatched item or part is requested
- **WHEN** a URL names an item or part that does not correspond to a wired
  item's actual parts (for example `/select/nonexistent-part`, or an item
  that isn't wired at all)
- **THEN** the dispatcher renders an explicit not-found state rather than
  crashing or rendering an unrelated item's composition

### Requirement: A page is part-routed only when all its controls are grouped
A playground page SHALL be eligible for a nested nav entry and dedicated
subcomponent routes only if every control it renders — generated or
hand-written — is inside a labeled `ControlGroup`. A page with even one
ungrouped, hand-written leaf control SHALL NOT be wired for nested
nav/routes until that control is grouped.

#### Scenario: An ungrouped control would leak across subcomponent routes
- **WHEN** a page has a hand-written control that is not inside any
  `ControlGroup`
- **THEN** wiring that page for subcomponent routes is deferred — an
  ungrouped control has no label to filter by, so it would otherwise render
  on every one of that item's subcomponent routes, misattributing it to
  whichever group happens to be focused

#### Scenario: A fully-grouped page qualifies
- **WHEN** every control a page renders is inside a `ControlGroup` (for
  example `/dialog`, whose page renders ten distinct labeled groups and no
  ungrouped leaves)
- **THEN** that page is eligible to be wired for nested nav entries and
  dedicated subcomponent routes
