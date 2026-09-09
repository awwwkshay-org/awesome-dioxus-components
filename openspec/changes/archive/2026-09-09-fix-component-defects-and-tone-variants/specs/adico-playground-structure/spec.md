## MODIFIED Requirements

### Requirement: Demo pages render only the demoed component's real composition
A playground demo page SHALL NOT render extra elements that exist only to
bind a generated control panel and are not part of a realistic composition
of the demoed component (e.g. a standalone pagination link rendered beside a
real pagination row solely to exercise its demo-state panel). When a
generated panel cannot be bound to the realistic composition, the page SHALL
omit that panel rather than invent a demo element for it. The Input page
SHALL render a single `Input` field rather than a second field added only to
demonstrate a hand-composed password-reveal pattern, once that pattern is
built into the `Input` registry component itself. The Drag And Drop List
page's use of `use_drag_and_drop_list_items()` SHALL be called from a scope
that is a descendant of `DragAndDropList`'s context provider, not from an
ancestor scope where the context does not resolve.

#### Scenario: A generated panel has no realistic binding target
- **WHEN** a generated control panel's props cannot be bound to any element
  of the page's realistic composition
- **THEN** the page omits that panel and renders only the realistic
  composition — no placeholder element is added to host the panel's bindings

#### Scenario: The Input page renders one field
- **WHEN** a user opens the Input playground page
- **THEN** exactly one `Input` field is rendered, and its password-reveal
  behavior (when `r#type` is `"password"`) comes from the `Input` component
  itself, not from a second, separately composed field

#### Scenario: The Drag And Drop List page renders without hanging
- **WHEN** a user opens the Drag And Drop List playground page
- **THEN** the page renders successfully and remains interactive, because
  `use_drag_and_drop_list_items()` is called from a scope that can resolve
  `DragAndDropList`'s context
