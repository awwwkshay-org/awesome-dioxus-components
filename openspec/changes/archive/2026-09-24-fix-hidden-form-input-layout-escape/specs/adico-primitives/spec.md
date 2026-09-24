## ADDED Requirements

### Requirement: A primitive's hidden form-participation input does not affect layout
A primitive that renders a hidden native input purely so its control
participates in real form submission SHALL ensure that input occupies no
layout space and contributes nothing to any scroll extent, including the
document's own.

It is not sufficient for such an input to be merely invisible. An input that
is transparent but still occupies a box, and whose containing block is the
initial containing block, is clipped by no ancestor scroll container and
therefore extends the document's scrollable area from wherever the control
happens to sit. Consumers that deliberately scroll inside a container rather
than at the document SHALL NOT have that contract broken by placing such a
control on a long page.

The input SHALL remain a real, form-participating native input: its type,
name, value, checked state, and disabled state SHALL be unchanged by whatever
mechanism removes it from layout.

#### Scenario: A control is placed low on a page that scrolls inside a container
- **WHEN** a consumer renders a primitive with a hidden form-participation
  input near the bottom of content that scrolls inside a container, in a
  layout whose document is not itself scrollable
- **THEN** the document's scrollable extent is unchanged, and the document
  does not become scrollable

#### Scenario: The control is submitted as part of a form
- **WHEN** a form containing the control is submitted
- **THEN** the control's name and value are submitted exactly as they were
  before the input was removed from layout, reflecting its current checked
  state
