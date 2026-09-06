## ADDED Requirements

### Requirement: CopyButton copies a given text value with visible confirmation
The registry SHALL provide a `copy-button` item exporting `CopyButton`,
which copies a supplied text value to the clipboard when activated and shows
a visible confirmation (e.g. a checkmark replacing the copy icon) for a short
time afterward. It SHALL be usable standalone next to any displayed text
value, not coupled to any one other component.

#### Scenario: A user copies a displayed value
- **WHEN** a user activates `CopyButton` next to a displayed text value
- **THEN** that value is copied to the clipboard and the button visibly
  confirms success, reverting to its resting state shortly after

#### Scenario: The copy fails
- **WHEN** the underlying clipboard operation fails (denied permission,
  unsupported target)
- **THEN** `CopyButton` reflects that failure rather than showing a false
  success confirmation

### Requirement: ThemeBuilder's CSS export can be copied
`ThemeBuilder`'s generated CSS-variables export SHALL be copyable via a
`CopyButton` next to it, composing the registry's `copy-button` item, rather
than requiring the user to manually select the read-only textarea's contents.

#### Scenario: A user copies the generated theme CSS
- **WHEN** a user activates the copy control next to `ThemeBuilder`'s CSS
  export
- **THEN** the exact text shown in the export textarea is copied to the
  clipboard

### Requirement: ColorPicker exposes format-aware text fields alongside the visual picker
The `color-picker` registry item SHALL export a `ColorPickerFields` part
composable inside `ColorPicker`, providing a compact format selector (HEX,
RGB, HSL) and the matching editable text/numeric field(s) for the currently
selected format, kept in sync with the same color state `ColorArea`,
`HueSlider`, and `ColorPickerSwatch` already read and write. It SHALL include
a `CopyButton` for the currently displayed formatted value. `ColorPickerFields`
is additive and composable: existing `ColorPicker` usages that do not include
it SHALL be unaffected.

#### Scenario: A user reads the color as HEX, RGB, or HSL
- **WHEN** a user selects a format from `ColorPickerFields`' format selector
- **THEN** the field(s) show the current color correctly converted into that
  format

#### Scenario: A user types a new value into a format field
- **WHEN** a user enters a valid value into `ColorPickerFields`' field(s) for
  the currently selected format
- **THEN** the picker's color updates to match, and `ColorArea`, `HueSlider`,
  and `ColorPickerSwatch` (if composed) all reflect the new value

#### Scenario: A user copies the current formatted value
- **WHEN** a user activates `ColorPickerFields`' copy control
- **THEN** the currently displayed formatted text (in whichever format is
  selected) is copied to the clipboard

#### Scenario: An existing ColorPicker composition is unaffected
- **WHEN** a `ColorPicker` is composed without `ColorPickerFields` (e.g. the
  compositions added by `demo-popup-components-behind-triggers`)
- **THEN** its rendered output and behavior are unchanged by this capability
