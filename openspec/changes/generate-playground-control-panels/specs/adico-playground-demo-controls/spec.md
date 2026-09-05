## MODIFIED Requirements

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

## ADDED Requirements

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
