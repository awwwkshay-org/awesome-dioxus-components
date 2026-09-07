## ADDED Requirements

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
