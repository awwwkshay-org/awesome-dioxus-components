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
dropdown-menu registry item SHALL compose this primitive directly, rather than
maintaining an independent, flat menu implementation. context-menu and
menubar SHALL each implement the same `role="menu"`/`role="menuitem"` contract
this primitive establishes, but MAY remain independent primitives rather than
composing `Menu` directly, when their anchoring/placement or multi-sibling
coordination model differs enough that adding the equivalent capability to
`Menu` would serve no other consumer — matching Base UI's own architecture,
where `Menubar`/`ContextMenu` share only role and keyboard conventions with
`Menu`, not its `Content`/`Item` implementation.

**Correction (2026-09-01):** originally required all three (context-menu,
dropdown-menu, menubar) to compose `Menu`. Implementation found this doesn't
hold for two of the three: context-menu owns click-point placement, Safari
viewport correction, and scroll suppression with no home in `MenuContent`
without new props serving no other consumer; menubar's per-sibling
open-state coordination has no `MenuContext` counterpart, mirroring Base
UI's own decision not to build `Menubar` on `Menu`. Only dropdown-menu — a
straight re-export, per Base UI's own "`Menu` *is* the dropdown menu" — was a
genuine unification. The requirement now describes that as the actual target
shape, not a temporary gap.

#### Scenario: A menu composes the shared Menu primitive
- **WHEN** dropdown-menu needs a nested submenu, checkbox item, or radio item
- **THEN** it is available directly, since dropdown-menu is a re-export of the
  shared `Menu` primitive

#### Scenario: A menu's placement or coordination model has no home on Menu
- **WHEN** context-menu's click-point placement or menubar's per-sibling
  open-state coordination has no equivalent on `Menu`, and adding one would
  serve no other consumer
- **THEN** the registry item implements the same `role="menu"`/
  `role="menuitem"` contract independently rather than being force-fit onto
  `Menu`, and this is not tracked as a parity gap

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
