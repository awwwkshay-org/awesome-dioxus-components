## ADDED Requirements

### Requirement: ColorPicker exposes a value-reflecting trigger composition
The `color-picker` registry item SHALL export a popover composition
(`ColorPickerPopover`, `ColorPickerTrigger`, `ColorPickerContent` via the
installed `popover` item) whose trigger renders a swatch reflecting the
currently selected color, following the same styled-popover-root injection
pattern `date-picker` already uses to let a consumer or the playground swap in
a custom popover root. The existing flat `ColorPicker`/`ColorArea`/
`AreaTrack`/`AreaThumb`/`HueSlider` composition SHALL remain available
unchanged for consumers who want the picker always visible.

#### Scenario: A consumer composes ColorPicker behind a trigger
- **WHEN** a consumer renders `ColorPickerTrigger` inside `ColorPickerPopover`
- **THEN** the trigger displays a filled swatch matching the current selected
  color, and activating it opens the popover containing the color-editing
  controls

#### Scenario: The swatch tracks color changes made in the open popover
- **WHEN** a user adjusts hue, saturation, or value while the popover is open
- **THEN** the trigger's swatch fill updates to match, without requiring the
  popover to close and reopen

#### Scenario: The swatch is accessible without seeing color
- **WHEN** an assistive-technology user reaches the trigger
- **THEN** it is labeled with the color's name (as returned by the existing
  `color_name` helper), not only conveyed through its visual fill
