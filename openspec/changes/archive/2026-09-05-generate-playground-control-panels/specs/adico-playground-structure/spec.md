## MODIFIED Requirements

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
