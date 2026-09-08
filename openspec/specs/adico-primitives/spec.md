# adico-primitives Specification

## Purpose

Define `adico-primitives` as a documented, public, shared behavior layer that
registry components compose, instead of each component reimplementing
positioning, menu behavior, focus management, or dismissal independently. This
capability formalizes the inventory and boundary decided in design.md §8a,
prompted by the 2026-08-30 shadcn props parity audit finding real API gaps in
29 of 38 tracked components traceable to missing or non-reusable shared
behavior.

## Requirements

### Requirement: Shared behavior is a public, documented primitive surface
Cross-cutting behavior used by more than one registry component (controllable
state, unique id generation, presence/animated-open, focus scope and trap,
dismissable layers, collection and roving-focus management, selection,
portals, and pointer/gesture tracking) SHALL be exposed as a public,
documented API of `adico-primitives`, not as a private module or
crate-private (`pub(crate)`) function reachable only from within the crate.

#### Scenario: A new registry component needs existing shared behavior
- **WHEN** a registry component requires behavior another component already
  implements (for example, outside-click dismissal or roving focus)
- **THEN** it composes the existing public primitive API instead of
  duplicating the behavior internally

#### Scenario: A shared primitive is still crate-private
- **WHEN** a primitive's behavior is only reachable through a private module
  or a `pub(crate)`/private `fn`
- **THEN** it does not satisfy this requirement even if the behavior itself is
  correct, and is tracked as a promotion gap

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

### Requirement: The menu primitive supports arbitrarily nested submenus
`adico-primitives` SHALL provide a single `Menu` primitive supporting
`SubmenuRoot`/`SubmenuTrigger` nesting to arbitrary depth, `CheckboxItem`,
`RadioGroup`/`RadioItem`, `Group`/`GroupLabel`, and `Separator`. The
dropdown-menu registry item SHALL compose this primitive directly, rather than
maintaining an independent, flat menu implementation. context-menu and
menubar SHALL each implement the same `role="menu"`/`role="menuitem"` contract
this primitive establishes, but MAY remain independent primitives rather than
composing `Menu` directly, when their anchoring/placement or multi-sibling
coordination model differs enough that adding the equivalent capability to
`Menu` would serve no other consumer — matching Base UI's own architecture,
where `Menubar`/`ContextMenu` share only role and keyboard conventions with
`Menu`, not its `Content`/`Item` implementation.

**Correction (2026-09-01):** originally required all three (context-menu,
dropdown-menu, menubar) to compose `Menu`. Implementation found this doesn't
hold for two of the three: context-menu owns click-point placement, Safari
viewport correction, and scroll suppression with no home in `MenuContent`
without new props serving no other consumer; menubar's per-sibling
open-state coordination has no `MenuContext` counterpart, mirroring Base
UI's own decision not to build `Menubar` on `Menu`. Only dropdown-menu — a
straight re-export, per Base UI's own "`Menu` *is* the dropdown menu" — was a
genuine unification. The requirement now describes that as the actual target
shape, not a temporary gap.

#### Scenario: A menu composes the shared Menu primitive
- **WHEN** dropdown-menu needs a nested submenu, checkbox item, or radio item
- **THEN** it is available directly, since dropdown-menu is a re-export of the
  shared `Menu` primitive

#### Scenario: A menu's placement or coordination model has no home on Menu
- **WHEN** context-menu's click-point placement or menubar's per-sibling
  open-state coordination has no equivalent on `Menu`, and adding one would
  serve no other consumer
- **THEN** the registry item implements the same `role="menu"`/
  `role="menuitem"` contract independently rather than being force-fit onto
  `Menu`, and this is not tracked as a parity gap

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

### Requirement: Primitives are independently validated
Each primitive in `adico-primitives` SHALL have tests that exercise its
behavior without requiring a specific registry/ui consumer component to exist,
so primitive correctness is not only verified indirectly through a downstream
styled component.

#### Scenario: A primitive is added before its consuming component
- **WHEN** a shared primitive (for example, the unified `Menu` or the
  `Positioner`) is implemented ahead of every registry component that will use
  it
- **THEN** it has its own passing tests demonstrating correct behavior in
  isolation

### Requirement: Persisted app-wide settings share one primitive
A setting that is shared app-wide and SHALL survive a reload SHALL be
expressed as a module-level `GlobalSignal` driven through the crate's single
persisted-global primitive, rather than each consumer hand-rolling its own
storage read/write. The primitive SHALL support multiple independent keys,
persisting to `localStorage` on `web` and to a per-key preferences file on
`native`, and SHALL behave as a plain in-memory shared signal (no
persistence) when neither client feature is enabled. It SHALL NOT require
the consuming application to mount a provider component.

#### Scenario: A second persisted setting is added
- **WHEN** a new shared, reload-surviving setting is introduced (for example
  a second registry component's own preference)
- **THEN** it composes the existing persisted-global primitive with its own
  `GlobalSignal` and storage key, rather than a new hand-written storage
  read/write implementation

#### Scenario: A persisted setting is read on a build with neither client feature
- **WHEN** a consumer builds with neither the `web` nor the `native` feature
  enabled
- **THEN** the setting behaves as a plain in-memory shared signal, defaulted
  to its declared default, with no panic and no attempted storage access

#### Scenario: Two components read the same persisted setting simultaneously
- **WHEN** two components are mounted at the same time and both read the
  same persisted setting
- **THEN** both observe one live, shared value, and a change from either one
  is immediately observed by the other with no reload needed

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

### Requirement: Clipboard copy is a target-gated shared primitive
`adico-primitives` SHALL expose a hook that copies a given text value to the
system clipboard, resolving to a status (idle, copied, or failed) rather than
a bare boolean, so a consumer can render a confirmation without maintaining
its own timer. On the `web` target it SHALL use the browser clipboard API. On
every other target it SHALL resolve to a failed status rather than silently
appearing to succeed, since no native clipboard integration exists in this
crate. Browser-interop details (the actual clipboard API call) SHALL stay
inside this primitive, never called directly from registry UI source.

#### Scenario: A web consumer copies text
- **WHEN** a `web`-target consumer calls the clipboard hook's copy function
  with a text value
- **THEN** the value is written to the system clipboard and the hook's status
  resolves to "copied"

#### Scenario: Clipboard access is denied or unsupported
- **WHEN** the browser denies clipboard permission, or the running target has
  no clipboard integration
- **THEN** the hook's status resolves to "failed", not "copied" and not a
  silent no-op

#### Scenario: Status is transient
- **WHEN** a copy attempt resolves to "copied" or "failed"
- **THEN** the status returns to "idle" after a short, fixed delay without
  the consumer needing to manage that timing itself

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

### Requirement: The scroll-area primitive exposes a mergeable, reusable scrollbar-styling contract
`ScrollArea` SHALL accept a caller-supplied `class` that is merged with, not replacing,
its internal visibility class, in both server-rendered and client-rendered output.
Scrollable regions across `adico-primitives` and `registry/ui` SHALL adopt this shared
contract instead of setting `overflow` directly on their own element, except for elements
that clip rather than scroll, native editing surfaces (`<textarea>`, contenteditable),
page/document-level scroll, and consumer-supplied virtualization boxes.

#### Scenario: A caller passes a custom class to ScrollArea
- **WHEN** a consumer renders `ScrollArea` with a `class` prop
- **THEN** the rendered element's `class` attribute contains both adico's internal
  visibility class and the caller's class, identically in server-rendered and
  client-rendered output

#### Scenario: A primitive-owned scroll element adopts the contract in place
- **WHEN** a component's own element already carries an ARIA role or existing scroll
  instrumentation (for example `SelectList`, `CommandList`, or `MessageScrollerViewport`)
- **THEN** the scroll-area styling contract is applied to that existing element rather
  than introducing a wrapping element that would change its structural position

#### Scenario: A non-scrolling clipping container is excluded
- **WHEN** an element uses `overflow: hidden` purely to clip content (for example
  rounded-corner or animation masking) rather than to scroll
- **THEN** it is not required to adopt the scroll-area contract

### Requirement: ScrollArea renders overlay scrollbars over a real native-overflow viewport
`ScrollArea` SHALL render its overlay scrollbar parts (viewport, scrollbar, thumb, corner)
such that the viewport remains a genuine native-overflow element receiving real browser
scroll events, and SHALL NOT use a `transform` to implement scrolling.

#### Scenario: An anchored popover continues tracking scroll
- **WHEN** a scroll-area viewport that has adopted the shared contract is scrolled while
  an anchored popover elsewhere on the page is open
- **THEN** the popover's position-tracking mechanism continues to receive native scroll
  events exactly as it did before adoption

#### Scenario: Server-rendered output has no measurement-dependent flash
- **WHEN** a page containing a `ScrollArea` is server-rendered and then hydrated in the
  browser
- **THEN** the overlay scrollbar thumb renders hidden until its first client-side
  measurement, producing no visible flash or jump between server output and hydrated
  output

### Requirement: ScrollArea's scrollbar-visibility props have observable, distinct effect
`ScrollType::Hidden` SHALL suppress the native scrollbar via an actual CSS declaration,
and `always_show_scrollbars` SHALL produce visually distinct styling from the default
auto-hide behavior.

#### Scenario: ScrollType::Hidden hides the native scrollbar
- **WHEN** a `ScrollArea` is rendered with `scroll_type: ScrollType::Hidden`
- **THEN** its markup contains a CSS style declaration suppressing the native scrollbar,
  not merely an inert HTML attribute of the same name

#### Scenario: always_show_scrollbars differs from the default
- **WHEN** a `ScrollArea` is rendered with `always_show_scrollbars: true` versus its
  default
- **THEN** the two renders apply distinguishably different CSS rather than identical
  rules under different class names

### Requirement: Keyboard navigation scrolls the active item into view inside a scrollable listbox or menu
Primitives that track an active or highlighted item inside a scrollable region
(listbox, select, combobox, command, menu, typeahead) SHALL scroll that item into view
when it changes via keyboard navigation and falls outside the currently visible area.

#### Scenario: Arrow-key navigation past the visible bottom of a long list
- **WHEN** a user presses the down-arrow key repeatedly in an open Select, Combobox, or
  Command list until the active option would fall below the visible viewport
- **THEN** the scroll container scrolls so the active option remains fully visible

#### Scenario: A short list needs no scrolling
- **WHEN** every item already fits within the scroll container's visible area
- **THEN** keyboard navigation does not trigger any scroll adjustment
