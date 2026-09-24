## Purpose
Keep `apps/home` — adico's public landing page — structured the same way
`apps/playground` already is: a dedicated routing module, CLI-installed
component source, and a shell composed from real registry components,
rather than a one-off, hand-authored app that drifts from how every other
adico consumer is actually built.

## ADDED Requirements

### Requirement: Router definitions live in routes.rs
`apps/home/src/routes.rs` SHALL define the `Route` enum and the routing
`Layout` shell component. `apps/home/src/main.rs` SHALL NOT define the
`Route` enum or the routing shell directly; it retains only the application
entrypoint, its asset consts, the document head, the `Router` mount, and the
CLI-managed `adico:start`/`adico:end` module block.

#### Scenario: main.rs is read for routing logic
- **WHEN** a maintainer opens `apps/home/src/main.rs` looking for how routes
  are defined
- **THEN** it contains no `#[derive(Routable)]` enum and no layout-shell
  component — those live in `routes.rs`

#### Scenario: A route is added
- **WHEN** a new page is added to the landing site
- **THEN** the corresponding `#[route(...)]` variant is added in
  `routes.rs`, not in `main.rs`

### Requirement: One page per file under pages/, named by route segment
Each routed page component SHALL live in its own file under
`apps/home/src/pages/`, aggregated by `pages/mod.rs`. The root route (`/`)
SHALL use the file name `index.rs`.

#### Scenario: The root route is located
- **WHEN** a maintainer looks for the component rendered at `/`
- **THEN** it is defined in `apps/home/src/pages/index.rs`

### Requirement: apps/home is initialized and kept current through the real adico CLI
`apps/home` SHALL be initialized and kept current through the real `adico`
CLI (`adico init`, `adico add`) rather than hand-edited component source.
Its `components.json`, `adico.lock`, `src/components/ui/*`,
`src/adico_lib/*`, and the `adico:theme:start`/`adico:theme:end` block of
`tailwind.css` SHALL be produced by those commands, never authored or
edited by hand — the same convention `apps/playground` and the
`examples/*` fixtures already follow, and consistent with CLAUDE.md's
architecture rule that a consumer-style app must exercise the CLI
installation path rather than import `registry/` source through a
workspace path.

#### Scenario: apps/home is refreshed
- **WHEN** the CLI installer's public commands are re-run against
  `apps/home`
- **THEN** the app's installed component source, `adico.lock`, and
  `components.json` are produced by those commands, not manual edits

#### Scenario: A maintainer searches for a workspace-path import of registry/
- **WHEN** a maintainer or tool searches `apps/home/src/**` for a `use`
  statement or path referencing the workspace's `registry/` directory
  directly
- **THEN** no such import exists — installed component source under
  `src/components/ui/` and `src/adico_lib/` is the only source of UI code

### Requirement: apps/home has its own per-project Tailwind pipeline
`apps/home` SHALL have its own root `tailwind.css` compiled to its own
`apps/home/assets/tailwind.css`, linked via
`document::Stylesheet { href: asset!(...) }`. It SHALL NOT reference or
depend on `apps/playground`'s or `apps/docs`'s compiled stylesheet, since
Tailwind only emits the utility classes actually referenced in each
project's own `src/`.

#### Scenario: apps/home is built standalone
- **WHEN** `apps/home` is built or served on its own, independent of
  `apps/playground` or `apps/docs`
- **THEN** every Tailwind utility class its own source references is
  present in `apps/home/assets/tailwind.css`, with no dependency on another
  app's generated stylesheet

### Requirement: The landing page shell composes real registry components, not app-specific reimplementations
`apps/home`'s page shell (header/navigation, theme controls, buttons, and
any card/layout surfaces) SHALL be composed from installed registry
components rather than hand-rolled, app-specific reimplementations of the
same behavior. Any app-specific wiring needed to compose these components
SHALL live under `apps/home/src/components/`, SHALL NOT duplicate a
registry component's own logic, and SHALL NOT require any change to
`registry/ui/*.rs` to exist.

#### Scenario: A new UI need arises on the landing page
- **WHEN** `apps/home` needs a button, card, navigation, or theme-toggle
  element
- **THEN** an existing installed registry component is composed to provide
  it, or a new generic registry component is proposed and added through the
  normal registry-item process — `apps/home` SHALL NOT gain a parallel,
  app-specific implementation of behavior a registry component already
  provides

#### Scenario: A registry component's defect is discovered while building apps/home
- **WHEN** composing an installed registry component in `apps/home` surfaces
  behavior that is wrong independent of `apps/home` (a real bug, not an
  app-specific inconvenience)
- **THEN** the fix lands in `registry/ui/*.rs` (or the relevant
  `adico-primitives` module) as its own cited change, not worked around in
  `apps/home`'s composition, and a registry component SHALL NOT be given an
  `apps/home`-specific escape hatch solely for this app's convenience
