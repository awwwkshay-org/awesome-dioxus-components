## ADDED Requirements

### Requirement: The playground exposes shell-free routes for viewport harness fixtures
The playground SHALL expose routes outside the desktop-only `Layout` shell that
render registry components full-bleed, with realistic fixture content, for use
by the automated viewport test harness. These routes are additive: they SHALL
NOT change the existing `Layout` shell's desktop-only behavior, and they are
not part of the playground's normal browsing navigation.

#### Scenario: The viewport harness measures a component's real geometry
- **WHEN** the narrow-viewport test harness needs to measure a registry
  component's layout at 375px without the playground shell's own fixed-size
  panels affecting the measurement
- **THEN** it loads a shell-free route that renders that component directly at
  the browser's actual viewport width

#### Scenario: A shell-free route needs fixture content the demo page doesn't provide
- **WHEN** a component's existing playground demo page uses fixture content too
  minimal to exercise a real-world layout failure (for example, a Tabs demo
  with only two tabs, which cannot reproduce horizontal overflow with five)
- **THEN** the shell-free harness route renders that component with separately
  authored, realistic fixture content, rather than reusing the minimal demo
  fixture
