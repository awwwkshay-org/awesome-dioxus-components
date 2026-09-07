> **Rejected (2026-09-07)** — this change (and this delta) was reverted
> before archiving; see `proposal.md`'s and `design.md`'s "Status: Rejected"
> sections. Nothing below reflects this app's current or intended behavior.

## ADDED Requirements

### Requirement: A control panel can be scoped to one component's group by route
When the current route names a focused subcomponent part, only the
`ControlGroup` whose own label matches that part SHALL render its children;
every other `ControlGroup` on that page SHALL render nothing. Every
`ControlGroup` on a wired page — whether emitted by a generated
`<Component>Controls` panel or hand-written directly on the page — SHALL
participate in this filtering automatically, with no generator change and
no regeneration of any generated control file required. A part with no
controllable props SHALL still show its labeled empty-state group when it
is the focused part.

#### Scenario: Only the focused group renders
- **WHEN** the current route names "Select Trigger" as the focused part on
  the Select page
- **THEN** the "Select Trigger" group's controls render, and every other
  group on that page (`Select`, `Select List`) renders nothing

#### Scenario: No focus means every group renders
- **WHEN** the current route names no focused part (the item's own unscoped
  page, e.g. `/select`)
- **THEN** every `ControlGroup` on that page renders exactly as it did
  before this capability existed

#### Scenario: Generated panels participate without regeneration
- **WHEN** this filtering capability is added
- **THEN** `cargo xtask playground-controls check` continues to pass with
  zero diff against the committed generated files — no generated file
  changes as a result of this capability

#### Scenario: A focused empty-state group still shows its message
- **WHEN** the current route's focused part has no controllable props (for
  example a part whose generated panel renders "No adjustable props.")
- **THEN** that group still renders, showing its empty-state message, when
  it is the one focused by the route
