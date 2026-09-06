# adico-existing-components — Delta

## ADDED Requirements

### Requirement: Menu components expose grouping, label, separator, and shortcut composition parts
The `dropdown-menu`, `context-menu`, and `menubar` registry items SHALL each
export styled composition parts for grouping related items (`*Group`), naming
a group (`*Label`, with an inset presentation option), visually separating
sections (`*Separator`), and rendering a right-aligned keyboard-shortcut hint
(`*Shortcut`). These parts SHALL compose inside the item's existing content
container without changing the behavior of the existing trigger, content, or
item parts, and SHALL NOT require the consumer to import `adico-primitives`
directly.

#### Scenario: A consumer builds a sectioned menu
- **WHEN** a consumer composes a menu content area using groups, labels,
  separators, and shortcut hints from the same registry item
- **THEN** the menu renders labeled sections divided by separators, shortcut
  hints align to the end of their item row, and keyboard navigation across
  the items behaves exactly as it does without the new parts

#### Scenario: Shortcut hints are presentational only
- **WHEN** a consumer renders a `*Shortcut` part inside a menu item
- **THEN** the hint is visual text only — it does not register a key binding
  and does not intercept keyboard events

### Requirement: Dropdown menu supports checkbox items, radio groups, and submenus
The `dropdown-menu` registry item SHALL export checked-state menu items
(`DropdownMenuCheckboxItem`), a single-select radio group
(`DropdownMenuRadioGroup` with `DropdownMenuRadioItem`), and nested submenu
parts (`DropdownMenuSub`, `DropdownMenuSubTrigger`, `DropdownMenuSubContent`)
styled consistently with existing menu items, supporting both controlled and
uncontrolled checked/open state. These parts are specific to `dropdown-menu`:
`context-menu` and `menubar` SHALL NOT export checkbox, radio, or submenu
parts while their underlying primitives lack the shared menu context those
parts require.

#### Scenario: A checkbox item toggles
- **WHEN** a user activates a `DropdownMenuCheckboxItem` via click or keyboard
- **THEN** its checked indicator toggles, the change callback fires with the
  new state, and the menu behaves per the underlying primitive's
  checkbox-item semantics

#### Scenario: A radio item is selected
- **WHEN** a user activates a `DropdownMenuRadioItem` inside a
  `DropdownMenuRadioGroup`
- **THEN** that item becomes the group's single selected value, its indicator
  renders, and the previously selected item's indicator clears

#### Scenario: A submenu opens from a parent menu
- **WHEN** a user hovers or keyboard-activates a `DropdownMenuSubTrigger`
- **THEN** its `DropdownMenuSubContent` opens adjacent to the parent menu and
  closing it does not close the parent menu

### Requirement: Carousel slides respond to pointer drag
The `carousel` registry item SHALL page between slides in response to a
pointer drag on the slide track, in both orientations: a drag whose distance
along the scroll axis exceeds a paging threshold SHALL advance one page in
the drag direction, and a shorter drag SHALL return to the current slide.
Existing previous/next buttons and arrow-key paging SHALL be unaffected.
The implementation SHALL use per-element pointer events, not a global
document-level pointer registry.

#### Scenario: A drag past the threshold pages the carousel
- **WHEN** a user presses on the slide track and drags along the scroll axis
  by more than the paging threshold before releasing
- **THEN** the carousel advances exactly one page in the drag direction

#### Scenario: A short drag snaps back
- **WHEN** a user drags the slide track by less than the paging threshold and
  releases
- **THEN** the carousel returns to the slide that was current before the drag

#### Scenario: Buttons and keyboard still page
- **WHEN** a user activates the previous/next buttons or presses the
  orientation-appropriate arrow keys on the focused track
- **THEN** the carousel pages exactly as it did before drag support existed

### Requirement: Input OTP can mask entered values
The `input-otp` registry item SHALL accept a reactive `mask` input on its
root. While masking is enabled, every slot SHALL render its entered character
as a password-style obscured value; while disabled, slots SHALL render the
plain character. Toggling masking SHALL NOT clear or alter the entered value,
and value-change/value-complete callbacks SHALL behave identically in both
modes.

#### Scenario: Masking hides entered digits
- **WHEN** a consumer sets `mask` to true on an OTP field containing entered
  characters
- **THEN** each filled slot displays an obscured character instead of the
  digit, and the field's value is unchanged

#### Scenario: Unmasking restores plain rendering
- **WHEN** `mask` transitions from true to false
- **THEN** the same entered characters render as plain text without any
  change to the value or caret/focus behavior

### Requirement: Textarea shows a character counter when a maximum length is set
When the `textarea` registry item's `max_length` prop is set, it SHALL render
a live character counter in the field's bottom-right corner showing the
current character count against the maximum (e.g. `120 / 500`), updating as
the user types and reflecting a controlled `value` when one is provided. The
counter SHALL NOT intercept pointer events. When `max_length` is not set,
the rendered output SHALL be unchanged from the previous behavior — no
counter and no extra wrapping element.

#### Scenario: Typing updates the counter
- **WHEN** a user types into a textarea whose `max_length` is 500
- **THEN** the bottom-right counter updates on each input to show the current
  count against 500, and the browser continues to enforce the native
  maxlength limit

#### Scenario: No max length means no counter
- **WHEN** a consumer renders a textarea without `max_length`
- **THEN** no counter appears and the component's DOM structure is identical
  to the pre-change output

#### Scenario: Controlled value drives the count
- **WHEN** a consumer passes a controlled `value` to a textarea with
  `max_length`
- **THEN** the counter reflects the controlled value's character count, not
  just user keystrokes
