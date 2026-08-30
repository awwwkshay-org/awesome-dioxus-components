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
under `pages/`.

#### Scenario: A page file exists with no matching route
- **WHEN** a file exists under `pages/` whose page component has no
  corresponding `#[route(...)]` variant in `routes.rs`
- **THEN** this is a defect to fix by adding the missing route, not
  evidence that routes are derived from the file tree
