## ADDED Requirements

### Requirement: theme-switcher's palette selection persists across reloads
`theme-switcher`'s selected coordinated palette preset SHALL persist across
a page reload, in addition to staying synced across every
simultaneously-mounted instance on the same page. `theme-builder`'s own
per-token edits are explicitly out of scope and SHALL remain an unpersisted,
transient live-preview surface.

#### Scenario: User selects a preset and reloads
- **WHEN** a user selects a palette preset in `theme-switcher` and reloads
  the page
- **THEN** the same preset is selected on reload, and its colors are applied

#### Scenario: User selects a preset and toggles light/dark
- **WHEN** a user selects a palette preset and then toggles the light/dark
  appearance
- **THEN** that preset's colors are recomputed for the new appearance and
  the selected preset itself is unchanged

#### Scenario: Two ThemeSwitcher instances are mounted at once
- **WHEN** two `theme-switcher` instances are mounted on the same page (for
  example a persistent sidebar instance and a demo-page instance)
- **THEN** both show and drive the same selection, and changing either one
  updates the other immediately
