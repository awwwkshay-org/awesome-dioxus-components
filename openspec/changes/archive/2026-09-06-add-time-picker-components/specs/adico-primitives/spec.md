## ADDED Requirements

### Requirement: Segmented numeric/enum field input is a shared, reusable primitive
Segment-by-segment keyboard-editable field input (roving focus across
segments, digit-typing with auto-advance on overflow, arrow-key increment/
decrement with wraparound, clamp-on-blur, and per-segment `role="spinbutton"`
ARIA with `aria-valuemin`/`aria-valuemax`/`aria-valuenow` for numeric segments
and `aria-valuetext` for non-numeric segments) SHALL be exposed as a shared
primitive usable by more than one composed field (at minimum, date segments
and time segments), rather than living as private, date-specific code inside
a single component module. A field composing multiple segments SHALL compute
each segment's position from the number of segments actually present rather
than a hardcoded per-field-kind constant, so composed fields of different
shapes (a single date, a date range, a combined date-and-time value) each lay
out correctly.

#### Scenario: A non-date field reuses the segment primitive
- **WHEN** a time field composes hour and minute segments using the shared
  segment primitive
- **THEN** it gets the same auto-advance, wraparound, and keyboard navigation
  behavior `DatePicker`'s day/month/year segments already have, without
  reimplementing that behavior

#### Scenario: A non-numeric segment is composed
- **WHEN** a field composes a segment whose value is not itself a number (for
  example, an AM/PM meridiem indicator)
- **THEN** the segment still participates in the same roving-focus and
  keyboard-navigation sequence as its numeric siblings, and exposes
  `aria-valuetext` rather than a numeric `aria-valuenow`

#### Scenario: A combined date-and-time field lays out correctly
- **WHEN** a field composes date segments and time segments together (for
  example, a `DateTimePicker`)
- **THEN** each segment's roving-focus position is correct regardless of how
  many date segments or time segments (including an optional seconds segment
  or meridiem segment) are present, and the existing `DateRangePicker`'s
  two-date layout is unaffected

### Requirement: Local-time resolution is a public primitive with an explicit fallback
`adico-primitives` SHALL expose a public API for resolving the current time
(not only the current date) in the local timezone of the device running the
UI, following the same resolve-with-fallback shape as the crate's existing
local-date resolution: resolve the true local offset where the platform
supports it, and fall back to UTC, explicitly and documented, where it does
not (for example, a non-wasm target without a sound OS-provided offset). This
API SHALL be usable by code outside `adico-primitives` itself (registry
components and consuming applications), not restricted to crate-internal
callers.

#### Scenario: A consumer needs the current local time
- **WHEN** a registry component or consuming application needs "now" in the
  device's local timezone (for example, a time picker's default value)
- **THEN** it can call a public `adico-primitives` API for it rather than
  hand-rolling `time`'s local-offset resolution and fallback itself

#### Scenario: Local offset cannot be resolved
- **WHEN** the running platform cannot supply a sound local UTC offset
- **THEN** the API returns UTC rather than panicking or returning an error the
  caller must separately handle, and this fallback behavior is documented at
  the call site
