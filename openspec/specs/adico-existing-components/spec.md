# Adico Existing Components Specification

## Purpose

Make every currently installed adico registry component dependable, idiomatic,
and easy to explore by closing evidence-backed shadcn/Dioxus gaps before new
component development resumes.

## Requirements

### Requirement: Every installed component has a hardening record
The project SHALL maintain an evidence-backed hardening record for each of the
66 current registry UI items, not only the original 21 (Button, Badge, Card,
Input, Textarea, Skeleton, Item, Pagination, Dialog, Sheet, Select, Combobox,
Tooltip, Popover, Hover Card, Dropdown Menu, Context Menu, Menubar, Calendar,
Date Picker, and Sidebar). The record SHALL assess public composition/API,
variants and states, semantic themes and dark mode, keyboard/pointer/focus
behavior, accessibility, responsive behavior, consumer examples, and
applicable platform checks. For upstream prop-surface completeness
specifically, `statics/prop_parity/<item>.json` (generated and CI-gated by
`cargo xtask prop-parity sync|check|diff`) SHALL be the mechanical evidence
source, superseding manual re-reading of upstream source for that dimension.

#### Scenario: Component audit identifies a gap
- **WHEN** the audit finds a missing or divergent applicable dimension
- **THEN** its record identifies the evidence, owning source boundary, and a
  bounded remediation task or explicit dependency block

#### Scenario: Upstream feature inventory is complete
- **WHEN** a maintainer reviews any current registry component
- **THEN** its record maps every applicable Dioxus Components part, public
  prop, state, interaction, and accessibility behavior to an adico API,
  intentional Dioxus alternative, or named target/platform block

#### Scenario: A component is missing an upstream capability
- **WHEN** the Dioxus Components reference exposes a capability not available
  from the corresponding adico registry component
- **THEN** the change includes a bounded primitive or registry-source task for
  it before the component can be marked complete, unless the ledger records a
  user-visible reason that the capability cannot apply

#### Scenario: A registry item's prop-parity record shows a genuine gap
- **WHEN** `statics/prop_parity/<item>.json` classifies an upstream prop
  `missing` for a registry item
- **THEN** the item's hardening record either implements that prop or
  records it `intentional_difference` with a written reason before the
  item is considered hardening-complete, matching the mechanical record
  rather than a separately maintained manual assessment

### Requirement: Components align with shadcn contracts and Dioxus idioms
Each improved component SHALL provide applicable current shadcn visual
variants, sizes, semantic tokens, and interactive states while exposing an
idiomatic Dioxus composition and native attribute model. Intentional departures
from React-only shadcn APIs SHALL document a Dioxus-safe alternative.

#### Scenario: Button is hardened first
- **WHEN** the Button implementation slice is completed
- **THEN** it supports caller-composed text, icon-only, and icon-plus-text
  children; all current shadcn variants and sizes; native Dioxus button
  attributes/events; semantic states; and semantic-link styling

#### Scenario: React-only composition cannot preserve semantics
- **WHEN** a shadcn API depends on a React-only mechanism
- **THEN** adico documents an intentional Dioxus alternative instead of
  exposing a misleading no-op prop

#### Scenario: Upstream behavior is presented idiomatically
- **WHEN** an upstream feature depends on composition, children, attributes,
  or Dioxus signals rather than a React-style render callback
- **THEN** adico SHALL expose the compositional or typed Dioxus contract and
  document it as the feature-equivalent public API

#### Scenario: First-wave primitives retain their behavior ownership
- **WHEN** Select, Combobox, Calendar, Date Picker, or Sidebar is hardened
- **THEN** keyboard, selection, focus, ARIA, and controlled-state behavior
  remains in `adico-primitives` and registry source owns only the documented
  visual façade and composition

#### Scenario: First-wave shadcn-style props are explored live
- **WHEN** a user opens a first-wave component in the playground
- **THEN** the route exposes only its applicable typed controls: Button
  variant/size/type/composition; Pagination active page and presentation;
  Select/Combobox value/open/disabled/selection mode; Calendar/Date Picker
  date state and constraints; and Sidebar open/side/collapsible/active state

#### Scenario: User enables multiple selections
- **WHEN** a user selects multi-select mode on the Select or Combobox
  playground route
- **THEN** the route renders the corresponding installed multi-select API,
  allows several options to remain selected, and updates the typed selected
  values without changing the popup's keyboard, focus, or ARIA behavior

#### Scenario: Playground validates an installed component API
- **WHEN** a component's registry-source API is changed
- **THEN** the playground receives that source through the CLI-managed
  installation path, imports no registry source directly, and exposes every
  meaningful supported prop through typed live controls before the component
  is marked hardening-complete

#### Scenario: A registry component composes another registry component
- **WHEN** a component uses another component as a visible action surface
- **THEN** it declares that component in `registryDependencies`, receives it
  through the CLI installation path, and reuses its public visual and native
  contract rather than duplicating an ad-hoc native equivalent

### Requirement: Playground exposes chosen component controls
Every playground component route SHALL render a centered, logical-size example
of the actual installed component and SHALL explicitly define the supported
props, options, types, values, and states that users can modify live. Controls
SHALL remain strongly typed by the route's Dioxus state rather than attempting
runtime reflection over component props. A generated `<Component>DemoState`/
`<Component>Controls` pair (`adico-playground-demo-controls`) satisfies this
requirement's strong-typing constraint for every prop the generator supports;
a component's remaining unsupported props, or its live preview when the
component is not single-root and non-generic, stay hand-written. Controls
that cannot be demonstrated safely or meaningfully SHALL be documented as
unavailable with their reason.

#### Scenario: User explores Button options
- **WHEN** a user opens the Button playground route
- **THEN** they can change its variant, size, disabled state, button type, and
  documented text/icon composition options and immediately see the installed
  Button update

#### Scenario: User explores a selected component
- **WHEN** a component has a closed option or value set such as side, align,
  appearance, or selection state
- **THEN** its route presents the applicable options as live controls and the
  rendered component updates without a page reload

#### Scenario: A route's controls come from the generated panel
- **WHEN** a component has a generated `<Component>DemoState`/
  `<Component>Controls` pair covering all of its controllable props
- **THEN** its playground route renders that generated panel bound to a
  `use_signal(<Component>DemoState::default)` rather than a page-local,
  hand-declared signal per prop

### Requirement: Improvements preserve source ownership and validation
Every improvement SHALL originate in registry source or the owned primitive
layer as appropriate. Playground and consumer fixtures SHALL be refreshed
through the CLI-managed installation path rather than direct edits to copied
components. Completed components SHALL have proportionate Rust, consumer
compile, browser, keyboard, accessibility, and applicable platform evidence;
unavailable checks SHALL be recorded as skipped.

#### Scenario: Shared overlay behavior is corrected
- **WHEN** an overlay component needs a focus, dismissal, or ARIA correction
- **THEN** reusable behavior is corrected in `adico-primitives`, visual source
  is corrected in the registry, and refreshed consumer copies receive it
  through the installer path

#### Scenario: Component is marked hardened
- **WHEN** a component is marked hardening-complete
- **THEN** its live playground controls, public documentation, CLI-refreshed
  fixture, and applicable evidence are available without claiming skipped
  validation passed

### Requirement: Current registry scope reaches complete applicable parity
All 66 registry UI items, not only the original 21 (Button, Badge, Card,
Input, Textarea, Skeleton, Item, Pagination, Dialog, Sheet, Select, Combobox,
Tooltip, Popover, Hover Card, Dropdown Menu, Context Menu, Menubar, Calendar,
Date Picker, and Sidebar), SHALL each reach complete applicable parity with
their Dioxus Components reference surface and their shadcn semantic visual
contract before this requirement is satisfied, as verified by
`cargo xtask prop-parity check` reporting no unresolved `missing`
classification for any item (every upstream prop is `present`,
`intentional_difference`, or was never applicable). This requirement SHALL
NOT imply that adico adds any catalog item which is not already in the
registry, and SHALL NOT require an adico-only extension prop (for example
`radius` or `loading`) to have an upstream counterpart at all.

#### Scenario: Existing-registry hardening is complete
- **WHEN** this change is proposed for completion
- **THEN** all 66 entries have a complete feature ledger, a CLI-refreshed
  source fixture, a typed live playground example, and proportionate behavior,
  accessibility, and target validation; any remaining unavailable capability
  is visibly recorded as a named block rather than counted as parity

#### Scenario: Prop-parity check gates completion
- **WHEN** `cargo xtask prop-parity check` is run against the full 66-item
  registry
- **THEN** it passes with zero `missing`-classified props remaining
  unresolved across every item, as the mechanical completion gate for this
  requirement's upstream-parity dimension

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

### Requirement: TimePicker lets a consumer set a time by typing or by popup, always in local time
The registry SHALL provide a `time-picker` item exporting a `TimePicker`
whose trigger is a segmented, directly-typeable field displaying the
currently selected time, and whose popup offers both a scrollable
hour/minute(/second)(/meridiem) columns view and an analog clock-dial view for
setting the same value, selectable via a `view` prop that SHALL default to
the columns view. All resolved and displayed time values SHALL be in the
local timezone of the device running the UI; the component SHALL NOT expose
or default to any other fixed timezone.

#### Scenario: A consumer types a time directly
- **WHEN** a user focuses `TimePicker`'s trigger and types digits
- **THEN** the corresponding hour/minute(/second) segment updates, auto-advancing
  to the next segment on overflow, without requiring the popup to open

#### Scenario: A consumer sets a time from the popup's columns
- **WHEN** a user opens `TimePicker`'s popup with its default view and
  selects an hour and minute from the scrollable columns
- **THEN** the trigger's segmented display updates to match, and the popup
  can be operated entirely by keyboard

#### Scenario: A consumer sets a time from the clock dial
- **WHEN** a user switches `TimePicker`'s popup to the clock view and drags or
  clicks a position on the dial
- **THEN** the selected hour or minute updates to the nearest valid value, and
  the same value remains settable through the columns view or the segmented
  trigger — the dial is one of several ways to reach the same state, not the
  only one
- **AND WHEN** a user cannot use a pointer
- **THEN** they are not blocked, because the columns view and the segmented
  trigger already provide full keyboard access to the same value

#### Scenario: A time value is always local
- **WHEN** `TimePicker` renders a default "current time" value, or a consumer
  reads its selected value
- **THEN** that value is in the local timezone of the device the UI is
  running on, not UTC or any other fixed offset

### Requirement: DateTimePicker composes DatePicker and TimePicker behind one trigger
The registry SHALL provide a `date-time-picker` item exporting a
`DateTimePicker` that composes the existing `date-picker` and new
`time-picker` items behind a single trigger showing the full formatted
date-and-time value, reusing each component's existing calendar and
time-selection surfaces rather than reimplementing them.

#### Scenario: A consumer picks both a date and a time
- **WHEN** a user opens `DateTimePicker`'s popup
- **THEN** they can select a date from the composed calendar and a time from
  the composed time-selection surface, and the trigger updates to show both
  once set

#### Scenario: The combined field supports direct typing
- **WHEN** a user types directly into `DateTimePicker`'s trigger
- **THEN** the date and time segments behave as one continuous
  keyboard-navigable sequence, correctly laid out regardless of whether a
  seconds segment or a 12-hour meridiem segment is present

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
