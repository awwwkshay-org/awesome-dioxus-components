## Purpose

Define measurable component parity and an explicit upstream-tracking workflow
that prioritizes reusable Dioxus Components before missing shadcn features.

## ADDED Requirements

### Requirement: Existing Dioxus Components are inventoried before gap work
Before adico implements a component absent from Dioxus Components, maintainers
SHALL inspect the current upstream repository and record its styled components,
primitives, dependencies, licenses, and provenance. Every existing styled item
SHALL be classified as `EXISTING_SHADCN_EQUIVALENT`, `EXISTING_DIOXUS_EXTRA`,
`NEEDS_PARITY_UPDATES`, `NEEDS_PRIMITIVE_FIX`, or `NOT_SUITABLE_FOR_REUSE`.

#### Scenario: Planning migration of an upstream component
- **WHEN** an upstream styled component is considered for the adico registry
- **THEN** its inventory record identifies its classification, dependencies,
  reusable source status, and provenance obligations

### Requirement: Existing installable components are the first product milestone
The first complete product milestone SHALL make Button, Dialog, and a selected
existing richer component installable through `adico init` and `adico add`,
with source ownership, required runtime dependencies, theme setup, successful
consumer build, and interaction validation. The project SHALL migrate the
remaining appropriate existing catalog before beginning implementation of
missing shadcn components.

#### Scenario: Vertical-slice installation succeeds
- **WHEN** a consumer initializes a normal Dioxus project and adds the selected
  vertical-slice components
- **THEN** the installed project builds and each component passes its defined
  interaction coverage

### Requirement: Parity is machine-readable and multi-dimensional
The repository SHALL maintain a machine-readable parity manifest for every
tracked first-party shadcn component. A component SHALL not be marked complete
until its required source, composition/API, visual/variant/state, keyboard,
accessibility, theme/dark-mode, applicable RTL and responsive behavior,
examples, CLI installation, Cargo resolution, documentation, and applicable
web, desktop, and SSR/hydration validation dimensions pass.

#### Scenario: Component has only source parity
- **WHEN** a component source exists but its accessibility validation is absent
- **THEN** parity reporting identifies the missing dimension and does not mark
  the component complete

### Requirement: Upstream catalog tracking is refreshable and reviewable
The project SHALL retain a checked-in snapshot of the current first-party
shadcn catalog, its source revision and refresh date, and a command that
explicitly refreshes and reports additions, removals, and material changes.
Ordinary CI SHALL calculate parity from checked-in data rather than depend on a
live upstream network request.

#### Scenario: Shadcn adds a component after a snapshot
- **WHEN** a maintainer refreshes the catalog snapshot
- **THEN** the new upstream item appears as untracked or missing until a parity
  record is added and completed

### Requirement: Parity delivery respects dependency groups
After the existing catalog is installable, missing components SHALL be planned
and delivered by shared primitive and composition dependencies rather than by
alphabetical order. The plan SHALL distinguish low-complexity gaps,
foundational primitives, complex interactive components, application-level
components, and newer chat/agent components.

#### Scenario: A missing component needs shared focus behavior
- **WHEN** gap analysis finds a component blocked by absent focus infrastructure
- **THEN** the shared primitive is scheduled and validated before the dependent
  component is declared complete

