## MODIFIED Requirements

### Requirement: Carousel slides respond to pointer drag
The `carousel` registry item SHALL page between slides in response to a
pointer drag on the slide track, in both orientations: a drag whose distance
along the scroll axis exceeds a paging threshold SHALL advance one page in
the drag direction, and a shorter drag SHALL return to the current slide.
Existing previous/next buttons and arrow-key paging SHALL be unaffected.
The implementation SHALL use per-element pointer events, not a global
document-level pointer registry.

`Carousel` SHALL derive its paging state from the slide track's current
geometry. A size measured before the track's layout has settled SHALL NOT
leave the component permanently unpageable: the component SHALL correct such a
measurement when the element's box changes, without requiring a scroll, a
remount, or any consumer action. Correction SHALL use the platform's own
element-resize notification rather than a scripting bridge.

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

#### Scenario: The track's size is not final when it first mounts
- **WHEN** `Carousel` mounts in a context where the slide track's size is not
  yet settled, so its first measurement reports content that does not overflow
- **AND** the track's box subsequently settles to its real size, with content
  that does overflow
- **THEN** the previous/next buttons become enabled and both pointer drag and
  arrow keys page the carousel — the component recovers on its own, with no
  scroll, remount, or consumer intervention

### Requirement: TimePicker lets a consumer set a time by typing or by popup, always in local time
The registry SHALL provide a `time-picker` item exporting a `TimePicker`
whose trigger is a segmented, directly-typeable field displaying the
currently selected time, and whose popup offers both a scrollable
hour/minute(/second)(/meridiem) columns view and an analog clock-dial view for
setting the same value, selectable via a `view` prop that SHALL default to
the columns view. All resolved and displayed time values SHALL be in the
local timezone of the device running the UI; the component SHALL NOT expose
or default to any other fixed timezone.

The clock dial SHALL resolve pointer positions against the face's current
size. A size measured before the popup has finished being placed and animated
SHALL NOT permanently misresolve pointer input: the component SHALL correct
such a measurement when the face's box changes, using the platform's own
element-resize notification.

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

#### Scenario: The dial is measured before the popup settles
- **WHEN** `TimePicker`'s clock dial is first measured while the popup is
  still being placed or animated, so the recorded face size is not its final
  one
- **AND** the face's box subsequently settles
- **THEN** dragging the dial moves the hand to the position under the pointer,
  resolved against the settled size
