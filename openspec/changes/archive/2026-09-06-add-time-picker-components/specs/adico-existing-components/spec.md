## ADDED Requirements

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
