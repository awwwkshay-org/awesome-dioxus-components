## MODIFIED Requirements

### Requirement: Playground exposes chosen component controls
Every playground component route SHALL render a centered, logical-size example
of the actual installed component and SHALL explicitly define the supported
props, options, types, values, and states that users can modify live. Controls
SHALL remain strongly typed by the route's Dioxus state rather than attempting
runtime reflection over component props. A generated `<Component>DemoState`/
`<Component>Controls` pair (`adico-playground-demo-controls`) satisfies this
requirement's strong-typing constraint for every prop the generator supports;
a component's remaining unsupported props, or its live preview when the
component is not single-root and non-generic, stay hand-written. Controls
that cannot be demonstrated safely or meaningfully SHALL be documented as
unavailable with their reason.

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

#### Scenario: A route's controls come from the generated panel
- **WHEN** a component has a generated `<Component>DemoState`/
  `<Component>Controls` pair covering all of its controllable props
- **THEN** its playground route renders that generated panel bound to a
  `use_signal(<Component>DemoState::default)` rather than a page-local,
  hand-declared signal per prop
