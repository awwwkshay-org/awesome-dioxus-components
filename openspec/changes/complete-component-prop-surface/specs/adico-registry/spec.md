## ADDED Requirements

### Requirement: Every styled registry item follows the shared prop conventions
Each `registry:ui`/`registry:component` item that renders a styled visual
surface SHALL follow a fixed set of prop conventions: a `class:
Option<String>` field kept separate from any `attributes` field; a
`disabled` representation matching its category (`Option<bool>` for a
native-leaf component, `ReadSignal<bool>` for a primitive-backed
component, with any exception to this documented in the component's own
doc comment); `#[props(extends = GlobalAttributes)]` on an `attributes:
Vec<Attribute>` field when the component renders a real element; a
handler-naming convention matching its category (unprefixed native names
for a native-leaf component, `on_*` semantic names for a primitive-backed
component); and, when the component holds controllable state, the
controlled trio expressed as `ReadSignal<Option<T>>` rather than a doubly
optional `Option<ReadSignal<Option<T>>>`, unless a specific case's
deviation is recorded with a reason.

#### Scenario: A native-leaf component's disabled prop
- **WHEN** a registry item renders a native HTML element directly with no
  `adico-primitives` module backing its interactive state
- **THEN** its `disabled` field is `Option<bool>`, not `ReadSignal<bool>`,
  unless the component's own doc comment records why it deviates

#### Scenario: A primitive-backed component's disabled prop
- **WHEN** a registry item's interactive state is owned by an
  `adico-primitives` module it composes
- **THEN** its `disabled` field is `ReadSignal<bool>`, matching the
  primitive's own controllable-prop convention

#### Scenario: A styled component has no class field
- **WHEN** a registry item renders a styled visual surface but declares no
  `class: Option<String>` field, including when its props type is a bare
  re-export of its primitive's own props type
- **THEN** the field is added — to the registry facade directly, or to the
  primitive's props type when the facade is a bare re-export, so the
  facade can remain a re-export rather than becoming a diverging wrapper

#### Scenario: A component's handler name doesn't match its category
- **WHEN** a primitive-backed registry item declares a handler prop named
  `on_click` or another React-style name instead of a semantic `on_*`
  name matching adico's own convention
- **THEN** the handler is renamed to a component-appropriate semantic name

#### Scenario: A controlled component uses a doubly optional value
- **WHEN** a registry item's controlled-value prop is
  `Option<ReadSignal<Option<T>>>` rather than `ReadSignal<Option<T>>`
- **THEN** it is migrated to the single-`Option` shape, unless the
  specific case's deviation is recorded with a reason

### Requirement: Prop convention compliance is mechanically verified and CI-gated
An adico-xtask command SHALL verify every `radius`-bearing registry item's
source contains no `rounded-*` Tailwind literal outside its `Radius`
enum's own `class()` implementation, running fully offline with no
dependency on fetched upstream catalogs. This command SHALL run as part
of continuous integration, not only as a locally invocable check.

#### Scenario: A radius-bearing component still hard-codes a rounded class
- **WHEN** a registry item that declares a `radius: Radius` prop also
  contains a `rounded-*` Tailwind literal in its base class string or in
  a variant/size `class()` implementation, outside `Radius::class()`
  itself
- **THEN** the verification command fails and identifies the item and the
  offending literal

#### Scenario: A radius-bearing component correctly delegates to Radius
- **WHEN** a registry item's only `rounded-*` literals are inside its own
  `Radius`-typed field's `class()` match arms
- **THEN** the verification command passes for that item
