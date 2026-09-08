# adico-playground-demo-controls Specification

## Purpose

Keep every playground demo page's control panel for an enum-valued prop in
exact agreement with the real component's variants, so a page can never
silently offer a stale or incomplete subset of what the component actually
supports.

## Requirements

### Requirement: Enum-valued prop controls are generated from the installed component source
For every enum-typed prop with a `#[default]` variant on a playground UI
component that has a demo control panel, the option list offered to the user
SHALL be generated from that component's actual installed source, not
hand-authored. The generated option list SHALL include every public variant
of the enum, in declaration order, each labeled by mechanically converting
its identifier to space-separated words (for example, `IconXs` becomes
`Icon Xs`). Registry source doc comments are prose, not short labels, and
SHALL NOT be used as label text.

#### Scenario: A component's demo page offers enum options
- **WHEN** the Badge demo page renders its Variant control
- **THEN** the control offers all variants declared on `BadgeVariant` in
  `apps/playground/src/components/ui/badge.rs`, including any not previously
  reachable from the playground

#### Scenario: A component's demo page offers Item's variants
- **WHEN** the Item demo page renders its Variant control
- **THEN** the control offers all variants declared on `ItemVariant`,
  including `Outline`

### Requirement: Generated option lists cannot silently go stale
An added, removed, or renamed variant on a playground UI component's
enum-typed prop, without its generated option list being regenerated, SHALL
cause the playground application to fail to build. Detecting the staleness
only through a separate, optional check command is not sufficient.

#### Scenario: A variant is added without regenerating
- **WHEN** a new variant is added to an enum-typed prop's declaration in
  `apps/playground/src/components/ui/*.rs` and the corresponding generated
  option list is not regenerated
- **THEN** `cargo check --locked --workspace` fails

#### Scenario: A variant is removed without regenerating
- **WHEN** a variant referenced by a generated option list is removed from
  its source enum and the generated option list is not regenerated
- **THEN** `cargo check --locked --workspace` fails

### Requirement: Generated output is verifiable without modifying the tree
A command SHALL exist that verifies every generated option list still
matches what regenerating from the current component source would produce,
without writing any file, and SHALL run offline with no network access. This
command SHALL be part of the project's committed validation commands.

#### Scenario: Generated output matches source
- **WHEN** the verification command runs against a tree where every
  generated option list agrees with its source enum
- **THEN** it completes successfully and modifies no file

#### Scenario: Generated output is hand-edited or stale
- **WHEN** a generated option list file has been hand-edited, or its source
  enum has changed without regeneration
- **THEN** the verification command fails and identifies the affected
  component

#### Scenario: Regeneration is idempotent
- **WHEN** the generation command runs twice in succession against an
  unchanged tree
- **THEN** the second run produces no file changes

### Requirement: Unsupported prop shapes are explicitly excluded, not silently dropped
A playground UI component's prop that cannot be represented by an existing
demo control (its type is `children`, `attributes`, `class`, an
`EventHandler<_>`, or a `Signal`/`ReadSignal`-wrapped value other than the
optional-controlled-boolean shape) SHALL be excluded from generation with a
recorded reason, not silently omitted with no trace. Numeric types and the
`Option<bool>`/optional-controlled-boolean shape SHALL be represented by a
`NumberControl`/`OptionalBoolControl` respectively, not treated as
unsupported.

#### Scenario: A component has a prop no control can represent
- **WHEN** a playground UI component declares a prop whose type is not
  `bool`, `Option<bool>`, `String`, a numeric type, the
  optional-controlled-boolean shape, or an enum with a `#[default]` variant
- **THEN** that prop does not appear in the generated output, and the
  generation record for that component states why

#### Scenario: A numeric prop is represented by a control
- **WHEN** a playground UI component declares a numeric-typed prop (for
  example `Slider`'s `min`/`max`/`step`)
- **THEN** it is represented by a generated `NumberControl` binding, not
  excluded as unsupported

### Requirement: A generated demo-state struct and control panel exist for every controllable prop
For every playground UI component with at least one prop representable by
an existing demo control, `cargo xtask playground-controls sync` SHALL
generate a `<Component>DemoState` struct (one field per controllable prop,
with a `Default` implementation matching each field's own default) and a
`<Component>Controls` component rendering one bound control per field, in
`apps/playground/src/generated/controls/<item>.rs`. A page SHALL wire this
generated state and panel in rather than hand-declaring its own
`use_signal`/control wiring for props the generator supports.

#### Scenario: A component's demo state and controls are generated
- **WHEN** `cargo xtask playground-controls sync` runs against a component
  with a `bool`, a `String`, and an enum-typed prop
- **THEN** the generated file declares a `DemoState` struct with all three
  fields and a `Controls` component rendering a `BoolControl`,
  `TextControl`, and `SelectControl` respectively, bound to that state

#### Scenario: A page uses the generated panel instead of hand-writing it
- **WHEN** a playground page for a component with a generated `Controls`
  component is written or converted
- **THEN** the page renders the generated `<Component>Controls` component
  bound to a `use_signal(<Component>DemoState::default)` rather than
  declaring its own per-prop signals and control JSX

### Requirement: Generated live preview is limited to single-root, non-generic components
`cargo xtask playground-controls sync` SHALL additionally generate a
`<Component>Preview` component, invoking the real installed component with
every field of its `DemoState`, only for a component that exposes exactly
one public root component with no generic type parameter. A component
composed of multiple public parts, or declared with a generic type
parameter, SHALL NOT have a `Preview` generated for it, and its playground
page keeps a hand-written preview.

#### Scenario: A single-root, non-generic component gets a generated preview
- **WHEN** a playground UI component (for example `Button`) declares
  exactly one public, non-generic root component
- **THEN** `playground-controls sync` generates a `ButtonPreview`
  component invoking `Button` with every field of `ButtonDemoState`

#### Scenario: A multi-part component does not get a generated preview
- **WHEN** a playground UI component is composed of multiple public parts
  (for example `Dialog`'s `DialogTrigger`/`DialogContent`/...) or declares
  a generic type parameter (for example `Select<T>`)
- **THEN** no `Preview` component is generated for it, and its playground
  page's preview stays hand-written

### Requirement: Generated control panels are labeled by their declaring component
Every generated `<Component>Controls` panel produced by
`cargo xtask playground-controls sync` SHALL render its controls under a
visible label naming the exact component whose props are being controlled.
The label SHALL be a human-readable rendering of that component's identifier
— its word boundaries space-separated and each word capitalized (Title
Case), for example `AccordionItem` rendered as "Accordion Item" — not the
literal, unspaced PascalCase Rust identifier. This is a display
transformation of the identifier, not a substitution for it: the label SHALL
still name that exact component and no other. When multiple generated panels
for different components in the same file appear together on a page (for
example `Sidebar`, `SidebarProvider`, and `SidebarMenuButton`), each panel's
controls SHALL remain visually and semantically distinguishable as belonging
to that one component, never appearing merged with another component's
controls or with the page's other, non-generated controls.

#### Scenario: A generated panel is labeled
- **WHEN** `cargo xtask playground-controls sync` generates a
  `<Component>Controls` panel for a component
- **THEN** the panel's rendered output includes a label naming that exact
  component, distinct from any other panel's label

#### Scenario: A multi-word component name is rendered as space-separated Title Case
- **WHEN** `cargo xtask playground-controls sync` generates a panel for a
  component whose identifier is multiple PascalCase words (for example
  `AccordionItem`, `SelectItemIndicator`, or `DialogPrimitiveContent`)
- **THEN** the label reads "Accordion Item", "Select Item Indicator", or
  "Dialog Primitive Content" respectively — space-separated and Title
  Case — not the unspaced identifier and not all-lowercase

#### Scenario: Multiple generated panels appear on one page
- **WHEN** a page composes more than one generated panel (for example
  `SidebarControls`, `SidebarProviderControls`, and
  `SidebarMenuButtonControls`)
- **THEN** each panel's controls are grouped and labeled separately, so a
  viewer can tell which component a given control belongs to without
  consulting source

#### Scenario: A part with no controllable props still gets a labeled panel
- **WHEN** a component (for example `AccordionTrigger` or `AccordionContent`)
  has no prop representable by an existing demo control
- **THEN** `playground-controls sync` still generates a panel for it, labeled
  with its component name, rendering a plain "no adjustable props" message
  instead of any control — the component is never omitted solely for having
  nothing to adjust

### Requirement: Every component a registry file exposes is discovered, whether defined locally or re-exported
`cargo xtask playground-controls sync` SHALL treat a component as eligible
for a generated panel whether it is defined locally in the playground UI file
or exposed only through a `pub use adico_primitives::...` re-export
(including a renamed re-export, e.g. `DialogRoot as Dialog`). A re-exported
name SHALL be included only if it resolves to an actual renderable component
in its defining module; a re-export that resolves to a non-component item
(a type, enum, or hook) SHALL be excluded, not shown as a component. A
re-exported name that cannot be resolved to any item in its defining module
SHALL cause `sync` and `check` to fail, naming the unresolved item, rather
than silently proceeding as if the component did not exist.

#### Scenario: A bare re-exported root component is discovered
- **WHEN** a playground UI file's only public surface for a component is a
  `pub use adico_primitives::<module>::<Name>;` re-export with no local
  wrapper (for example `Accordion` in `accordion.rs`)
- **THEN** `playground-controls sync` generates a panel for that component,
  labeled with its local, public name, the same as it would for a locally
  defined component

#### Scenario: A non-component re-export is excluded
- **WHEN** a playground UI file re-exports a name from `adico_primitives`
  that resolves to a type, enum, or hook rather than a component (for
  example `checkbox.rs`'s `CheckboxState`, `slider.rs`'s `SliderProps`, or
  `toast.rs`'s `use_toast`)
- **THEN** no panel is generated for that name — it is not treated as a
  component

#### Scenario: An unresolvable re-export fails the build
- **WHEN** a playground UI file re-exports a name from `adico_primitives`
  that does not correspond to any item in its defining module
- **THEN** `cargo xtask playground-controls sync` and
  `cargo xtask playground-controls check` both fail, identifying the
  unresolved name, rather than generating output that silently omits it
