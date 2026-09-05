## MODIFIED Requirements

### Requirement: Shared catalog schema across all four axes
All four `statics/catalogs/<axis>.json` files SHALL conform to one shared
schema. For each component or primitive entry, the schema SHALL record: a
stable identifier/name, its composition (the other primitives/parts it is
built from, when applicable), and its prop set with an explicit
`props_source` per prop group, where `props_source` is one of `explicit` (a
concrete list of prop name/type/default/description), `inherits_from:
<axis>.<component>.<part>` (props are a passthrough of another axis's
component and are not re-listed), or `unavailable` (no prop data could be
determined for this entry). For shadcn specifically, a component's
`cva()`-declared variant groups (its dominant prop-declaration pattern, used
by the majority of its components) SHALL be recorded as explicit props, not
left unrepresented alongside an unrelated augmentation object. For
dioxus-components and dioxus-primitives specifically, an inline
function-argument component's parameters SHALL be recorded as explicit
props whenever `rust_introspect.rs` can read that file's parameter list, not
marked `unavailable` merely because the component has no separate
`#[derive(Props)]` struct.

#### Scenario: shadcn passthrough component
- **WHEN** `catalog fetch shadcn` records shadcn's `Dialog` `Trigger` part,
  whose only declared type is `React.ComponentProps<typeof
  DialogPrimitive.Trigger>`
- **THEN** its catalog entry has `props_source: inherits_from:
  base-ui.dialog.trigger` (or the axis shadcn's Dialog composes on for that
  release) rather than a duplicated prop list

#### Scenario: shadcn augmented component
- **WHEN** `catalog fetch shadcn` records shadcn's `Dialog` `Content` part,
  which adds a `showCloseButton` prop on top of the underlying primitive's
  props
- **THEN** its catalog entry records `showCloseButton` explicitly in addition
  to noting the inherited base

#### Scenario: shadcn cva-declared variant props
- **WHEN** `catalog fetch shadcn` records a component whose props type
  includes `VariantProps<typeof <alias>>` for a local `const <alias> =
  cva(<base>, { variants: { <group>: { ... }, ... }, defaultVariants: {
  ... } })` declaration in the same file
- **THEN** its catalog entry records one explicit prop per variant group
  (e.g. `variant`, `size`), each with the group's keys as a literal-union
  type and the matching `defaultVariants` entry as its default, in addition
  to any `& { ... }` augmentation props on the same signature

#### Scenario: shadcn cva-declared variant props on a native-tag passthrough
- **WHEN** `catalog fetch shadcn` records a component whose props type is
  `React.ComponentProps<"div"> & VariantProps<typeof alertVariants>` (a
  native-element passthrough combined with a `cva()` variant group, rather
  than a `React.ComponentProps<typeof OtherComponent>` passthrough)
- **THEN** its catalog entry records the variant group's props explicitly,
  the same as the `VariantProps<typeof buttonVariants>` case, rather than
  falling through to `props_source: unavailable`

#### Scenario: Base UI explicit props
- **WHEN** `catalog fetch base-ui` records a Base UI component part
- **THEN** its catalog entry has `props_source: explicit` with each prop's
  name, type, default, and description as published in Base UI's API
  reference

#### Scenario: Dioxus prop struct
- **WHEN** `catalog fetch dioxus-primitives` or `catalog fetch
  dioxus-components` records a component whose props are a Rust
  `#[derive(Props)]` struct
- **THEN** its catalog entry has `props_source: explicit` with each struct
  field's name, type, and default (when present) as declared in the fetched
  source

#### Scenario: Dioxus inline function-argument component
- **WHEN** `catalog fetch dioxus-components` or `catalog fetch
  dioxus-primitives` records a component declared as `#[component] pub fn
  X(prop_a: T, prop_b: U, children: Element) -> Element` with no separate
  `#[derive(Props)]` struct
- **THEN** its catalog entry has `props_source: explicit` with `prop_a`,
  `prop_b`, and `children` listed, rather than `props_source: unavailable`
