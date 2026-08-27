## Purpose

Make every currently installed adico registry component dependable, idiomatic,
and easy to explore by closing evidence-backed shadcn/Dioxus gaps before new
component development resumes.

## ADDED Requirements

### Requirement: Every installed component has a hardening record
The project SHALL maintain an evidence-backed hardening record for each of the
21 current registry UI items: Button, Badge, Card, Input, Textarea, Skeleton,
Item, Pagination, Dialog, Sheet, Select, Combobox, Tooltip, Popover, Hover
Card, Dropdown Menu, Context Menu, Menubar, Calendar, Date Picker, and
Sidebar. The record SHALL assess public composition/API, variants and states,
semantic themes and dark mode, keyboard/pointer/focus behavior, accessibility,
responsive behavior, consumer examples, and applicable platform checks.

#### Scenario: Component audit identifies a gap
- **WHEN** the audit finds a missing or divergent applicable dimension
- **THEN** its record identifies the evidence, owning source boundary, and a
  bounded remediation task or explicit dependency block

#### Scenario: Upstream feature inventory is complete
- **WHEN** a maintainer reviews any current registry component
- **THEN** its record maps every applicable Dioxus Components part, public
  prop, state, interaction, and accessibility behavior to an adico API,
  intentional Dioxus alternative, or named target/platform block

#### Scenario: A component is missing an upstream capability
- **WHEN** the Dioxus Components reference exposes a capability not available
  from the corresponding adico registry component
- **THEN** the change includes a bounded primitive or registry-source task for
  it before the component can be marked complete, unless the ledger records a
  user-visible reason that the capability cannot apply

### Requirement: Components align with shadcn contracts and Dioxus idioms
Each improved component SHALL provide applicable current shadcn visual
variants, sizes, semantic tokens, and interactive states while exposing an
idiomatic Dioxus composition and native attribute model. Intentional departures
from React-only shadcn APIs SHALL document a Dioxus-safe alternative.

#### Scenario: Button is hardened first
- **WHEN** the Button implementation slice is completed
- **THEN** it supports caller-composed text, icon-only, and icon-plus-text
  children; all current shadcn variants and sizes; native Dioxus button
  attributes/events; semantic states; and semantic-link styling

#### Scenario: React-only composition cannot preserve semantics
- **WHEN** a shadcn API depends on a React-only mechanism
- **THEN** adico documents an intentional Dioxus alternative instead of
  exposing a misleading no-op prop

#### Scenario: Upstream behavior is presented idiomatically
- **WHEN** an upstream feature depends on composition, children, attributes,
  or Dioxus signals rather than a React-style render callback
- **THEN** adico SHALL expose the compositional or typed Dioxus contract and
  document it as the feature-equivalent public API

#### Scenario: First-wave primitives retain their behavior ownership
- **WHEN** Select, Combobox, Calendar, Date Picker, or Sidebar is hardened
- **THEN** keyboard, selection, focus, ARIA, and controlled-state behavior
  remains in `adico-primitives` and registry source owns only the documented
  visual façade and composition

#### Scenario: First-wave shadcn-style props are explored live
- **WHEN** a user opens a first-wave component in the playground
- **THEN** the route exposes only its applicable typed controls: Button
  variant/size/type/composition; Pagination active page and presentation;
  Select/Combobox value/open/disabled/selection mode; Calendar/Date Picker
  date state and constraints; and Sidebar open/side/collapsible/active state

#### Scenario: User enables multiple selections
- **WHEN** a user selects multi-select mode on the Select or Combobox
  playground route
- **THEN** the route renders the corresponding installed multi-select API,
  allows several options to remain selected, and updates the typed selected
  values without changing the popup's keyboard, focus, or ARIA behavior

#### Scenario: Playground validates an installed component API
- **WHEN** a component's registry-source API is changed
- **THEN** the playground receives that source through the CLI-managed
  installation path, imports no registry source directly, and exposes every
  meaningful supported prop through typed live controls before the component
  is marked hardening-complete

#### Scenario: A registry component composes another registry component
- **WHEN** a component uses another component as a visible action surface
- **THEN** it declares that component in `registryDependencies`, receives it
  through the CLI installation path, and reuses its public visual and native
  contract rather than duplicating an ad-hoc native equivalent

### Requirement: Playground exposes chosen component controls
Every playground component route SHALL render a centered, logical-size example
of the actual installed component and SHALL explicitly define the supported
props, options, types, values, and states that users can modify live. Controls
SHALL remain strongly typed by the route's Dioxus state rather than attempting
runtime reflection over component props. Controls that cannot be demonstrated
safely or meaningfully SHALL be documented as unavailable with their reason.

#### Scenario: User explores Button options
- **WHEN** a user opens the Button playground route
- **THEN** they can change its variant, size, disabled state, button type, and
  documented text/icon composition options and immediately see the installed
  Button update

#### Scenario: User explores a selected component
- **WHEN** a component has a closed option or value set such as side, align,
  appearance, or selection state
- **THEN** its route presents the applicable options as live controls and the
  rendered component updates without a page reload

### Requirement: Improvements preserve source ownership and validation
Every improvement SHALL originate in registry source or the owned primitive
layer as appropriate. Playground and consumer fixtures SHALL be refreshed
through the CLI-managed installation path rather than direct edits to copied
components. Completed components SHALL have proportionate Rust, consumer
compile, browser, keyboard, accessibility, and applicable platform evidence;
unavailable checks SHALL be recorded as skipped.

#### Scenario: Shared overlay behavior is corrected
- **WHEN** an overlay component needs a focus, dismissal, or ARIA correction
- **THEN** reusable behavior is corrected in `adico-primitives`, visual source
  is corrected in the registry, and refreshed consumer copies receive it
  through the installer path

#### Scenario: Component is marked hardened
- **WHEN** a component is marked hardening-complete
- **THEN** its live playground controls, public documentation, CLI-refreshed
  fixture, and applicable evidence are available without claiming skipped
  validation passed

### Requirement: Current registry scope reaches complete applicable parity
The 21 registry UI items—Button, Badge, Card, Input, Textarea, Skeleton, Item,
Pagination, Dialog, Sheet, Select, Combobox, Tooltip, Popover, Hover Card,
Dropdown Menu, Context Menu, Menubar, Calendar, Date Picker, and Sidebar—SHALL
each reach complete applicable parity with their Dioxus Components reference
surface and their shadcn semantic visual contract before this change is
complete. This requirement SHALL NOT imply that adico adds any catalog item
which is not already in the registry.

#### Scenario: Existing-registry hardening is complete
- **WHEN** this change is proposed for completion
- **THEN** all 21 entries have a complete feature ledger, a CLI-refreshed
  source fixture, a typed live playground example, and proportionate behavior,
  accessibility, and target validation; any remaining unavailable capability
  is visibly recorded as a named block rather than counted as parity
