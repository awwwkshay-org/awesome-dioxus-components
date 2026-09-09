## ADDED Requirements

### Requirement: An automated viewport harness measures responsive layout
The project SHALL maintain an automated browser test harness that renders
registry components at a narrow (375×812) viewport and asserts that no element
of any exercised component extends past the viewport's horizontal bounds. The
project SHALL maintain a companion harness at the project's existing desktop
viewport that asserts each exercised component's computed geometry matches its
pre-mobile-first-change baseline, so a responsive override that unintentionally
also changes desktop rendering is caught automatically.

#### Scenario: A component regresses into horizontal overflow
- **WHEN** a change to a registry component causes any of its elements to
  extend past a 375×812 viewport's horizontal bounds
- **THEN** the narrow-viewport harness fails and names the offending element

#### Scenario: A responsive override leaks into desktop rendering
- **WHEN** a change intended to affect only viewports narrower than 640px
  changes a component's computed geometry at the project's desktop viewport
- **THEN** the desktop-invariance harness fails

### Requirement: Responsive layout results are reported honestly
Once the viewport harness exists, validation reporting SHALL record responsive
layout as measured — passing, failing, or explicitly out of scope for a given
component — rather than as unmeasurable. A component report SHALL NOT continue
to claim responsive layout is unmeasurable once the harness covers that
component.

#### Scenario: A previously-unmeasured component gains harness coverage
- **WHEN** the viewport harness is extended to exercise a registry component
  that a prior audit recorded as having unmeasurable responsive behavior
- **THEN** that component's recorded verdict is updated to reflect the
  harness's actual result, not left as unmeasurable

#### Scenario: A component is deliberately out of the harness's scope
- **WHEN** a component's responsive behavior cannot be exercised by the
  harness (for example, native desktop or mobile-platform rendering, which
  this project has no fixture for)
- **THEN** the report states that explicitly as an out-of-scope exception, not
  as a passed check
