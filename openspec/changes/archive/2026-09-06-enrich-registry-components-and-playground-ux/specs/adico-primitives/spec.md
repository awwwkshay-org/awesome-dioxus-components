# adico-primitives — Delta

## ADDED Requirements

### Requirement: The OTP field primitive supports masked slot rendering
The OTP field primitive root SHALL accept a reactive boolean `mask` input,
defaulting to off. While enabled, each slot input SHALL render as a
password-type input so entered characters are visually obscured; while
disabled, slots SHALL render as plain text inputs. Masking SHALL be purely
presentational: the field's value model, per-slot editing behavior, focus
movement, and value-change/value-complete callbacks SHALL be identical in
both modes, and toggling the flag SHALL NOT clear or reorder entered
characters. The primitive's documentation SHALL no longer list masking as an
out-of-scope feature.

#### Scenario: Masked slots render as password inputs
- **WHEN** the root's `mask` input is true and the field is rendered
- **THEN** every slot input carries the password input type instead of the
  text input type

#### Scenario: Toggling mask preserves state
- **WHEN** a field holds entered characters and `mask` is toggled
- **THEN** the field's value, filled-slot positions, and active-slot focus
  are unchanged; only the visual rendering of the characters differs

#### Scenario: Callbacks are mode-independent
- **WHEN** a user completes entry while masking is enabled
- **THEN** the value-complete callback fires with the same plain-text value
  it would report with masking disabled
