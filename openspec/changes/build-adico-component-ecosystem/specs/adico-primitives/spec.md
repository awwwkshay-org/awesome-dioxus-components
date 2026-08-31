## Purpose

Define `adico-primitives` as a documented, public, shared behavior layer that
registry components compose, instead of each component reimplementing
positioning, menu behavior, focus management, or dismissal independently. This
capability formalizes the inventory and boundary decided in design.md §8a,
prompted by the 2026-08-30 shadcn props parity audit finding real API gaps in
29 of 38 tracked components traceable to missing or non-reusable shared
behavior.

## ADDED Requirements

### Requirement: Shared behavior is a public, documented primitive surface
Cross-cutting behavior used by more than one registry component (controllable
state, unique id generation, presence/animated-open, focus scope and trap,
dismissable layers, collection and roving-focus management, selection,
portals, and pointer/gesture tracking) SHALL be exposed as a public,
documented API of `adico-primitives`, not as a private module or
crate-private (`pub(crate)`) function reachable only from within the crate.

#### Scenario: A new registry component needs existing shared behavior
- **WHEN** a registry component requires behavior another component already
  implements (for example, outside-click dismissal or roving focus)
- **THEN** it composes the existing public primitive API instead of
  duplicating the behavior internally

#### Scenario: A shared primitive is still crate-private
- **WHEN** a primitive's behavior is only reachable through a private module
  or a `pub(crate)`/private `fn`
- **THEN** it does not satisfy this requirement even if the behavior itself is
  correct, and is tracked as a promotion gap

### Requirement: Anchored-overlay components share one positioning implementation
Every component that positions floating content against an anchor element
(popover, hover-card, tooltip, select, combobox, dropdown-menu, context-menu,
menubar) SHALL compose a single shared `Positioner` primitive supporting side,
align, offset, collision boundary and padding, sticky behavior, and anchor
tracking, plus a shared `Arrow` part. No component SHALL implement its own
placement math.

#### Scenario: Two anchored components need the same positioning behavior
- **WHEN** both popover and tooltip need `sideOffset` support
- **THEN** the fix lands once in the shared `Positioner` and both components
  gain the behavior without component-specific changes

### Requirement: The menu primitive supports arbitrarily nested submenus
`adico-primitives` SHALL provide a single `Menu` primitive supporting
`SubmenuRoot`/`SubmenuTrigger` nesting to arbitrary depth, `CheckboxItem`,
`RadioGroup`/`RadioItem`, `Group`/`GroupLabel`, and `Separator`. The
context-menu, dropdown-menu, and menubar registry items SHALL compose this
primitive rather than each maintaining an independent, flat menu
implementation.

#### Scenario: A menu needs a submenu
- **WHEN** a registry menu component needs a nested submenu, checkbox item, or
  radio item
- **THEN** it is available directly from the shared `Menu` primitive without
  new menu-specific behavior being written in the registry component

### Requirement: Controlled and uncontrolled state follow one uniform pattern
Every primitive that exposes stateful behavior a consumer may want to control
(open/closed, value, checked) SHALL use the crate's existing controllable-state
pattern (`use_controlled`) rather than a bespoke internal state mechanism.

#### Scenario: A primitive only supports uncontrolled state
- **WHEN** a primitive exposes internal state with no controlled `value`/
  `on_value_change`-shaped escape hatch (for example, accordion's current
  implementation)
- **THEN** it is tracked as a parity gap until it adopts the uniform
  controllable-state pattern

### Requirement: Primitives are independently validated
Each primitive in `adico-primitives` SHALL have tests that exercise its
behavior without requiring a specific registry/ui consumer component to exist,
so primitive correctness is not only verified indirectly through a downstream
styled component.

#### Scenario: A primitive is added before its consuming component
- **WHEN** a shared primitive (for example, the unified `Menu` or the
  `Positioner`) is implemented ahead of every registry component that will use
  it
- **THEN** it has its own passing tests demonstrating correct behavior in
  isolation
