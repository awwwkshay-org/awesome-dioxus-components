## ADDED Requirements

### Requirement: Hover-intent open/close delay is a shared, reusable primitive
Debounced, cancelable open/close-on-hover behavior (a per-open/close-request
generation counter that supersedes a still-pending timer, resolving a zero-delay
request on the next microtask without waiting on a timer) SHALL be exposed as a
shared primitive usable by more than one component (at minimum, the composed `Menu`
submenu, `HoverCard`, and `NavigationMenu`), rather than each component maintaining
its own copy of the same generation-counter/timer logic. The primitive SHALL be
generic over the value being requested (not hardcoded to a boolean open/closed
flag) and SHALL accept the delay to apply as a plain argument per request, so a
caller whose own domain logic determines a particular request should bypass its
usual delay (for example, switching directly from one already-open item to a
sibling item) simply passes zero for that one request — the primitive does not need
its own predicate parameter to support this, since delay selection already varies
per caller and each caller already computes it from its own state before calling.

#### Scenario: Two components need the same hover-intent delay behavior
- **WHEN** both `HoverCard` and `Menu`'s submenu trigger need to open after a
  configurable delay and close after a different configurable delay, canceling a
  still-pending open if the pointer leaves before it elapses
- **THEN** both compose the same shared primitive instead of each maintaining its own
  generation-counter/timer implementation

#### Scenario: A delay of zero still resolves without waiting on a timer
- **WHEN** a component requests an open or close with its configured delay resolving
  to zero
- **THEN** the requested state applies on the next microtask with no timer wait, so a
  component whose delays default to zero observes no timer-driven latency (the
  request is still dispatched as a cancelable async task, consistent with every
  prior hand-rolled implementation this primitive replaces — none of them applied a
  zero-delay request synchronously within the same render either)

#### Scenario: A non-boolean payload is requested
- **WHEN** a component (for example `NavigationMenu`, tracking which of several
  sibling items is open) needs the delay behavior to carry a value other than a
  plain open/closed boolean
- **THEN** the primitive supports that payload type directly, rather than requiring
  the component to layer its own value-tracking around a boolean-only primitive

#### Scenario: A request should bypass its own delay
- **WHEN** a component's own domain logic determines a particular open/close request
  should apply immediately regardless of its configured delay (for example, switching
  directly from one already-open item to a sibling item, where waiting would read as
  laggy)
- **THEN** the component computes zero for that request's delay argument itself
  (closing over whatever ambient state the decision depends on) and passes it to the
  primitive, which still runs its own generation-counter cancellation for that
  request exactly as it would for any other delay value

## MODIFIED Requirements

### Requirement: Controlled and uncontrolled state follow one uniform pattern
Every primitive that exposes stateful behavior a consumer may want to control
(open/closed, value, checked) SHALL use one of the crate's controllable-state
primitives — `use_controlled` for state where a single-level `Option<T>` is
sufficient to represent "not controlled," or `use_optionally_controlled` for
state where the value itself is `Option<T>` and "controlled, but currently
nothing selected" must remain distinguishable from "not controlled at all"
(which `use_controlled`'s single-level `Option` cannot represent, since a
controlled prop reading `None` there always falls back to the default) —
rather than a bespoke internal state mechanism.

#### Scenario: A primitive only supports uncontrolled state
- **WHEN** a primitive exposes internal state with no controlled `value`/
  `on_value_change`-shaped escape hatch (for example, accordion's current
  implementation)
- **THEN** it is tracked as a parity gap until it adopts one of the uniform
  controllable-state patterns

#### Scenario: A primitive's controlled value is itself optional
- **WHEN** a primitive's controllable value is `Option<T>` (for example, an
  accordion's open item, a single-select's selected value, or a radio
  group's selected value), such that `use_controlled` could not distinguish
  a controlled-and-cleared value from an uncontrolled one without silently
  falling back to a default
- **THEN** it uses `use_optionally_controlled` instead of hand-rolling the
  same `Option<ReadSignal<Option<T>>>` distinction independently, and its
  controlled-and-cleared behavior is covered by a test proving the default
  is not substituted in that case

### Requirement: Segmented numeric/enum field input is a shared, reusable primitive
Segment-by-segment keyboard-editable field input (roving focus across
segments, digit-typing with auto-advance on overflow, arrow-key increment/
decrement with wraparound, clamp-on-blur, and per-segment `role="spinbutton"`
ARIA with `aria-valuemin`/`aria-valuemax`/`aria-valuenow` for numeric segments
and `aria-valuetext` for non-numeric segments) SHALL be exposed as a shared,
public primitive module usable by more than one composed field (at minimum, date
segments and time segments), rather than living as private, date-specific code
inside a single component module, and rather than as a crate-private
(`pub(crate)`) module reachable only from within the crate. A field composing
multiple segments SHALL compute each segment's position from the number of
segments actually present rather than a hardcoded per-field-kind constant, so
composed fields of different shapes (a single date, a date range, a combined
date-and-time value) each lay out correctly.

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

#### Scenario: The segment primitive is reachable from outside the crate
- **WHEN** code outside `adico-primitives` (a registry component, a consuming
  application, or a test) needs the segment primitive's types or provider hook
- **THEN** it can import them directly (`adico_primitives::segment::...`)
  rather than the module being reachable only from within the crate

### Requirement: Anchored-overlay components share one positioning implementation
Every component that positions floating content against an anchor element
(popover, hover-card, tooltip, select, combobox, dropdown-menu, context-menu,
menubar) SHALL compose a single shared `Positioner` primitive supporting side,
align, offset, collision boundary and padding, sticky behavior, and anchor
tracking, plus a shared `Arrow` part. No component SHALL implement its own
placement math.

#### Scenario: Two anchored components need the same positioning behavior
- **WHEN** both popover and tooltip need `sideOffset` support
- **THEN** the fix lands once in the shared `Positioner` and both components
  gain the behavior without component-specific changes

**Correction (2026-09-01):** originally required all eight listed components,
with no exception, to compose `Positioner`. Implementation found two
deliberate, permanent exceptions, for two distinct reasons — popover,
hover-card, tooltip, select, combobox, and dropdown-menu all migrated cleanly
(task 7.8c/7.8d/7.8e) and this requirement stands unmodified for them.
context-menu anchors to an arbitrary click point, not a DOM element —
`Positioner` has no anchor-element to key off in that model, and adding
virtual-point anchoring would be a primitive feature addition serving no
other consumer, not a context-menu-sized migration. menubar's exception is
different in kind: `Positioner` computes its position once, from a rect
measured at mount, and by its own module doc "deliberately does **not**
continuously reposition on scroll/resize" because the observer-bridge
capability that would need (a separate, still-blocked capability) is
unimplemented. menubar's current plain CSS `absolute` placement was verified
live (open a menu, force scroll on its containing region, re-measure both
the trigger's and the content's `getBoundingClientRect()`) to stay glued to
its trigger through a scroll the one-shot `Positioner` would not track.
Migrating would trade that working scroll-follow for collision-awareness a
top-of-chrome dropdown rarely needs. Both exceptions are re-evaluated only if
a future change lands the observer-bridge capability this requirement's
"No component SHALL implement its own placement math" line assumes exists.

**Correction (2026-09-07):** the "observer-bridge capability" this
requirement's 2026-09-01 Correction described as fully unimplemented is now
partially landed, and menubar's exception needs re-stating precisely rather
than treated as either fully closed or fully open. `Positioner` gained
`use_reposition_bridge`, wired into its own recompute path, and that
primitive's `MutationObserver`-driven path was live-verified end-to-end in a
real browser (an anchor-relative CSS mutation produced an exact,
pixel-matching reposition, reproduced twice). Its
`IntersectionObserver`/`ResizeObserver`/native-`scroll` dispatch paths —
the specific mechanism menubar's own exception was verified against — were
confirmed only to *register* (spied via instrumented `addEventListener`),
never fire-tested end-to-end: the automated verification environment runs
with `document.hidden` permanently `true`, a state Chrome throttles those
callbacks under, indistinguishable from an inactive real user tab from the
API's point of view but not from a foregrounded one. context-menu's
exception is unaffected either way — its reason (no anchor element for a
click-point) does not depend on the observer-bridge capability at all.
menubar's exception therefore remains in force until its specific dependency
— a live, foregrounded-browser reproduction of the same scroll-follow
measurement that originally earned the exception — is actually run and
passes. Implementing menubar's migration without that verification, on the
strength of the (unrelated) `MutationObserver` path's success alone, is not
sufficient and SHALL NOT be treated as closing this exception.

**Correction (2026-09-08): menubar's exception is closed.** `MenubarContent`
now composes `Positioner` (`menubar.rs`'s own module doc records the full
migration), anchored to a newly-added per-`MenubarMenu` trigger id.
`Positioner` renders inline — a `position: fixed`-styled element still in its
normal place in the component tree, not a DOM portal — so `MenubarMenu`'s
pre-existing `onkeydown` (wrapping both trigger and content) needed no
changes; this was a placement-only migration.

The dependency the 2026-09-07 Correction left open — a live reproduction of
the exact scroll-follow measurement that originally earned the exception —
was run and passed: with a menu open, scrolling the page 100px moved both the
trigger and the positioned content by exactly 100px (`335.05px → 235.05px`
and `367.05px → 267.05px`), preserving the same 32.0px offset before and
after. This is a strictly better result than the pre-migration exception
(which only ever claimed to stay glued through scroll, with no
collision-awareness); the migration adds real collision-aware placement on
top of the same scroll-follow behavior.

One assumption in the 2026-09-07 Correction turned out to be wrong, and is
corrected here rather than left standing: it treated `document.hidden`
staying permanently `true` in the automated verification environment as
blocking *any* live reproduction of this measurement, on the reasoning that
Chrome throttles `IntersectionObserver`/`ResizeObserver` callbacks under that
condition. That reasoning is correct for those two observers specifically,
but `use_reposition_bridge`'s actual scroll-tracking path
(`positioner.rs`) is a plain `document.addEventListener('scroll', ..., true)`
listener — not gated on either observer — and empirically it fired and
recomputed correctly even with `document.hidden: true` confirmed on the same
tab immediately before and after the measurement. The measurement was
therefore run and passed in that same automated environment, not a
foregrounded tab as the prior Correction insisted was required; the user
reviewed this exact evidence (the precise before/after numbers above) and
confirmed it satisfies the exception's own dependency rather than requiring a
separate foregrounded-tab repetition. `IntersectionObserver`/`ResizeObserver`
firing end-to-end in this environment remains unverified, but menubar's own
exception was never earned by those two specifically — only by the
scroll-follow behavior now proven.

context-menu's exception is unaffected — its reason (no anchor element for a
click-point) never depended on the observer-bridge capability.
