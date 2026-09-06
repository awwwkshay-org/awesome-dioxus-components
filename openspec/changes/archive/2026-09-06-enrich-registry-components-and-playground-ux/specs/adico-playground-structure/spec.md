# adico-playground-structure — Delta

## ADDED Requirements

### Requirement: Navigation lists components in flat alphabetical order
The playground navigation list (`nav_items()`) SHALL present all component
entries as a single flat list ordered alphabetically ascending by displayed
label, with no thematic batches or restarted alphabetical runs. A newly
added component page SHALL be inserted at its alphabetical position.

#### Scenario: A user scans the navigation for a component
- **WHEN** a user opens the playground and scans the navigation sidebar
- **THEN** every component entry appears in one continuous A→Z sequence by
  its displayed label

#### Scenario: A new component page is registered
- **WHEN** a maintainer adds a navigation entry for a new component page
- **THEN** the entry is inserted at its alphabetical position in
  `nav_items()`, not appended at the end

### Requirement: The demo preview zone is pannable with a reset control
The shared demo preview zone SHALL render the demoed component centered by
default and SHALL let the user reposition it by dragging the preview
background. A drag that begins on the demoed component (or its immediate
wrapper) SHALL NOT start a pan, so the component's own pointer interactions
are never intercepted. The preview zone SHALL render a "Center" reset
control at the bottom of the preview area, composed from the installed
Button registry component, that restores the component to the centered
position.

#### Scenario: Dragging the background moves the component
- **WHEN** a user presses on the preview zone's background and drags
- **THEN** the rendered component follows the drag offset and remains at the
  released position

#### Scenario: Dragging the component itself does not pan
- **WHEN** a user begins a drag on the demoed component (e.g. a slider thumb
  or a carousel track)
- **THEN** the preview does not pan and the component receives the pointer
  interaction unchanged

#### Scenario: Center resets the position
- **WHEN** a user activates the Center control after panning
- **THEN** the component returns to the centered default position

### Requirement: Playground demo tooling is composed from installed registry components
The playground's shared demo chrome and control widgets (boolean, text,
number, select, and optional-boolean controls) SHALL be composed from
installed registry components (e.g. Switch, Input, NativeSelect, Label,
Card, Button) rather than raw HTML form elements, to the extent an installed
component provides the needed behavior. The control widgets' public
signatures SHALL remain compatible with the generated control panels that
call them.

#### Scenario: A control widget renders
- **WHEN** a demo page renders a boolean, text, number, or select control
- **THEN** the widget's interactive element is an installed registry
  component, not a hand-styled raw HTML input

#### Scenario: Generated panels keep working
- **WHEN** the control widgets' internals are rebuilt on installed components
- **THEN** every generated `<Component>Controls` panel compiles and behaves
  unchanged, without regenerating or modifying the generator

### Requirement: Demo pages render only the demoed component's real composition
A playground demo page SHALL NOT render extra elements that exist only to
bind a generated control panel and are not part of a realistic composition
of the demoed component (e.g. a standalone pagination link rendered beside a
real pagination row solely to exercise its demo-state panel). When a
generated panel cannot be bound to the realistic composition, the page SHALL
omit that panel rather than invent a demo element for it.

#### Scenario: A generated panel has no realistic binding target
- **WHEN** a generated control panel's props cannot be bound to any element
  of the page's realistic composition
- **THEN** the page omits that panel and renders only the realistic
  composition — no placeholder element is added to host the panel's bindings
