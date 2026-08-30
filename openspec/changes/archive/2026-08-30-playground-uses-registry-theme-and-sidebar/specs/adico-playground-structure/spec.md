## ADDED Requirements

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
