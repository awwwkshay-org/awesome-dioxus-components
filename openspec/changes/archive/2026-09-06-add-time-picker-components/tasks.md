## 1. Confirm local-time resolution before building on it

- [x] 1.1 **Already answered, by `demo-popup-components-behind-triggers`**:
  `packages/adico-primitives/src/lib.rs`'s
  `local_time_tests::now_local_resolves_on_this_native_target_rather_than_falling_back`
  probes `OffsetDateTime::now_local()` directly under `cargo test`'s default
  multithreaded harness and asserts `Ok` — confirmed empirically on this
  workspace's native macOS target: the `unsound_local_offset` gate that would
  force a UTC fallback does not apply here. No need to re-derive; read that
  test's doc comment for the full finding before writing `now_local_time()`/
  `now_local_datetime()` in task 2.1.
- [x] 1.2 **Answered by the same test.** Native macOS (and, by the same
  `time`-internal target gating, Windows) resolves true local time; the UTC
  fallback exists for Linux/BSD native builds and is not exercised on the
  platform this was tested on. Document this in `now_local_time()`/
  `now_local_datetime()`'s own doc comments in task 2.1, same as the existing
  `now_local_date()` comment does, rather than re-probing.

## 2. Public local-time primitives

- [x] 2.1 Added `now_local_time()` and `now_local_datetime()` as public
  trait methods alongside `LocalDateExt::now_local_date()` in
  `packages/adico-primitives/src/lib.rs`, reusing its
  `.unwrap_or_else(|_| ...now_utc()...)` fallback shape (`now_local_datetime`
  resolves the offset once via a single `now_local()` call, per design.md's
  "resolve the offset once" decision — not two independent calls). Verify:
  `cargo check --locked --workspace` passes.
- [x] 2.2 Added `now_local_time_and_datetime_agree_with_now_local_date`,
  asserting all three resolve to a mutually consistent date/hour on this
  target (the existing `now_local_resolves_on_this_native_target...` test
  already covers the fallback-vs-real-offset finding for all three, since
  they share one code path). `cargo test -p adico-primitives` passes (2
  tests in `local_time_tests`).

## 3. Extract the shared segment primitive

- [x] 3.1 Extracted `packages/adico-primitives/src/segment.rs`:
  `SegmentFieldContext` (`focus`, `disabled`, `read_only` only — no
  date-specific fields), `use_segment_field_provider`, and `NumericSegment`/
  `NumericSegmentProps` (renamed from the private `DateSegment`/
  `DateSegmentProps` it was extracted from). `date_picker.rs`'s
  `BaseDatePickerContext` had `focus`/`disabled`/`read_only` removed
  entirely (dead after the extraction — confirmed via grep before removing)
  and now composes `SegmentFieldContext` alongside it via
  `use_segment_field_provider`. The date-specific "focusing a segment closes
  the enclosing popover" behavior moved to a new `on_focus` hook each of
  `DatePickerYearSegment`/`MonthSegment`/`DaySegment` wires itself (closing
  over their own already-fetched `BaseDatePickerContext`), rather than the
  shared primitive knowing about popovers. Verify:
  `cargo test -p adico-primitives --test test_date_picker` — all 8
  pre-existing tests pass unmodified.
- [x] 3.2 Added `on_format_value: Option<Callback<T, String>>` to
  `NumericSegmentProps` (not a `#[props(default = ...)]` closure — that
  can't see the struct's own `max_length` field to close over it; `None`
  falls back to the original `format!("{:0>width$}", ...)` inline). Verify:
  all `test_date_picker` tests (which never set this field) pass
  byte-for-byte unmodified.
- [x] 3.3 Added `MeridiemSegment`/`MeridiemSegmentProps` to `segment.rs`:
  same roving-focus sequence and `onmounted`/`onfocus` shape as
  `NumericSegment`, `aria-valuetext` ("AM"/"PM"/"--") instead of numeric
  ARIA, ArrowUp/ArrowDown and `A`/`P` keys toggle the value. Verify: covered
  end-to-end by `test_time_picker.rs`'s
  `a_12_hour_time_input_adds_a_meridiem_segment_and_displays_the_12_hour_value`
  (asserts the rendered "PM" text) once `time_picker.rs` consumed it — no
  separate isolated unit test, since `MeridiemSegment` has no real caller
  until `TimePickerMeridiemSegment` exists.
- [x] 3.4 Centralized index computation: `DateElementContext` now stores
  precomputed `year_index`/`month_index`/`day_index` fields (computed once,
  in `DateElement`'s body, from `start_index`) instead of each segment
  wrapper repeating `ctx.start_index + <literal>`. Added
  `const DATE_SEGMENT_COUNT: usize = 3` and changed
  `DateRangePickerEndValue`'s `start_index: 3` to
  `start_index: DATE_SEGMENT_COUNT`. Verify: added
  `date_range_picker_composes_six_spinbutton_segments_with_derived_indices`
  to `test_date_picker.rs` (asserts 6 spinbuttons, 2 of each
  year/month/day label, 5 hidden separators) — passes, alongside all 8
  pre-existing tests. **Correction from the original task wording:** a
  black-box SSR test harness has no way to observe which segment currently
  holds simulated keyboard focus (nothing in the rendered markup reflects
  it), so "unchanged focus order" is verified structurally (correct segment
  *count* and labels) rather than by simulating an actual ArrowRight
  keypress across the start/end boundary — a real focus-order regression
  would need a live-browser Playwright check.
- [x] 3.5 `cargo clippy --locked -p adico-primitives --all-targets -- -D
  warnings` — clean, both with and without the `web` feature.

## 4. TimePicker primitive and columns/dial state

- [x] 4.1 Added `packages/adico-primitives/src/time_picker.rs`: `TimePicker`
  root (mirrors `DatePicker`'s shape — `use_segment_field_provider` +
  `TimePickerContext`), `TimeElement` (mirrors `DateElement`'s
  signals-in/effect-reconciles-out pattern, substituting `Time::from_hms`
  for `Date::from_calendar_date`), `TimePickerHourSegment`/`MinuteSegment`/
  `SecondSegment`/`MeridiemSegment` wrappers built on `segment::NumericSegment`/
  `MeridiemSegment`, `TimePickerInput`/`InputValue`. Segment counts/offsets
  (`second_index`/`meridiem_index`) are `Memo`-backed (not plain `usize`,
  unlike date's fixed 3-segment shape) since whether seconds/meridiem exist
  is reactive to the `show_seconds`/`is_12h` props, which a consumer can
  toggle live. Also added `ClockDial`/`use_clock_dial`/`angle_to_value` for
  the dial view, and a small, justified addition to
  `move_interaction.rs::MoveInteraction`: a public `pointer_position()`
  accessor (wraps the already-tracked `active_pointer_id` +
  `pointer::pointer_position`) — needed because a dial requires the
  pointer's *absolute* position relative to the element's center to compute
  an angle, unlike `pointer_move()`'s delta-since-last-frame that
  slider/color-picker use; this is "what's required" per design.md's
  Non-Goal on `use_move_interaction`, not a new pointer primitive.
  **Real bug found and fixed by this task's own verification, before it
  ever reached the registry layer:** the first draft applied
  `to_display_hour`'s 12-hour conversion unconditionally regardless of
  `is_12h`, so a 24-hour `TimePicker` showing 14:30 rendered hour segment
  "2" instead of "14" — caught by
  `test_time_picker.rs::a_24_hour_time_input_renders_the_canonical_hour`,
  fixed by gating the conversion on `is_12h()` (see that file/module's
  comments for detail). Verify: `cargo test -p adico-primitives` — 4 inline
  unit tests (`to_display_hour`/`from_display_hour` round-trip,
  `angle_to_value` cardinal-point and near-boundary snapping) plus 5
  `tests/test_time_picker.rs` SSR tests (root role/label, 2/3-segment
  composition, 24h vs 12h hour display, seconds granularity) all pass.
- [x] 4.2 `cargo check --target wasm32-unknown-unknown -p adico-primitives
  --features web` — clean.

## 5. Registry: `time-picker` item

- [x] 5.1 Authored `registry/ui/time_picker.rs`: `TimePicker`,
  `TimePickerInput`/`InputValue` + re-exported segments
  (hour/minute/second/meridiem via task 3's primitives), `TimePickerTrigger`
  (renders the selected time via `TimePickerContext`), `TimePickerContent`,
  `TimePickerClock` (click/tap-to-set dial, deliberately not continuous-drag —
  see the file's own module doc for why), `TimePickerColumns`. `rustfmt
  --edition 2024` parses clean.
  **Follow-up fixes from two post-implementation user review passes** (found
  via live browser testing on `/time-picker` and `/date-time-picker`, not
  caught by any compile-time check):
  - `TimePickerClock`'s hour dial rendered a literal `"0"` label at the top
    (12 o'clock) position in 12-hour mode instead of `"12"` -- the internal
    value domain for the 12h hour dial is `0..12` (matching
    `to_display_hour`'s own `0 == 12 o'clock` convention used by
    `current_value`/`apply_value`), but the label text rendered the raw
    `value` instead of translating `0` to `"12"`. Fixed with a
    `display_label` closure. Confirmed live: toggling 12-hour now correctly
    highlights "12" at the top of the dial.
  - Added an opt-in `fill_height: bool` prop (default `false`) to
    `TimePickerColumnsProps` so a consumer composing it next to a taller
    sibling (`date-time-picker`'s calendar) can stretch its columns to match,
    instead of the fixed `max-h-48`. Two real bugs were found and fixed while
    wiring this up on the `date-time-picker` playground page (documented in
    task 6's follow-up note below) before this was confirmed working.
  - **Reversal of the original task-5.1 deviation** (a second, later review
    pass explicitly asked for a `view: TimePickerView` prop): added the
    `TimePickerView` enum (`Digital` (default) | `Analog`), a registry-only
    `TimePickerViewContext` provided by `TimePicker` from a new `view` prop,
    and a `TimePickerBody` component that reads that context and renders
    `TimePickerColumns` or `TimePickerClock` accordingly.
    `TimePickerColumns`/`TimePickerClock` remain independently composable
    directly (unchanged, still used that way on `time-picker`'s own demo
    page originally) for a consumer who wants to hardcode one specific view
    without a prop switch. Also found and fixed a real, unrelated gap while
    implementing this: `TimePickerClock`'s dial face has no AM/PM affordance
    at all (clicking it only ever sets the 1-12 position) -- with only the
    clock composed and no `TimePickerColumns`, there was previously no way
    to choose AM vs. PM. Added an AM/PM button pair next to the Hour/Minute
    select, always rendered when `is_12h` is set, sharing a new
    `apply_meridiem`/`time_option_class` pair of free functions with
    `TimePickerColumns`'s own (pre-existing) AM/PM buttons rather than
    duplicating the logic. Confirmed live: switching `/time-picker`'s new
    "View" demo control to Analog and clicking "PM" correctly set the
    trigger to "12:00 PM".
  - **Missing popup surface (third review pass — "time picker feels off").**
    `TimePickerColumns`/`TimePickerClock` rendered with no card at all: bare
    text floating on the page background, with the columns' own scroll
    clipping reading as a rendering glitch rather than a scrollable list.
    Root cause: `TimePickerContent` strips the popover's frame (`border-0
    bg-transparent p-0 shadow-none`) copied verbatim from
    `DatePickerContent` -- but that convention only works because the *view
    part* supplies the surface instead (`calendar`'s `CalendarView` carries
    `border bg-popover p-3 text-popover-foreground shadow-sm`), and the time
    views carried nothing. Fixed by giving both time views their own card
    surface matching `CalendarView`'s, restoring the established
    content-strips-frame / view-owns-surface split rather than special-casing
    `TimePickerContent`. The `date-time-picker` playground page's hand-drawn
    `border-t`/`border-l` divider became redundant and was dropped (the two
    panels now read as two cards). Confirmed live on both pages; DOM-verified
    that the calendar/time height match still holds exactly (both 320px, same
    top and bottom) after the added border and padding.
  - **Duplicated value in the trigger field (fourth review pass — "fix layout
    issues").** Composing `TimePickerTrigger` inside a `TimePickerInput` next
    to `TimePickerInputValue` rendered the same time twice in one field —
    `05 : 00` from the segments, then a full nested bordered button reading
    `5:00 AM ⌄`. Root cause: `TimePickerTrigger`'s default is a standalone
    labeled button (correct on its own, and what the earlier "trigger should
    show the selected time" request asked for), whereas `date-picker`'s
    `DatePickerTrigger` is always the compact icon-only shape precisely
    because it lives inside the field. Added a `compact: bool` prop (default
    `false`) that swaps to that icon-only shape, and set it on the
    `time-picker` demo page. The prop swaps the *whole* base class rather
    than layering overrides, because `cn` (`registry/lib/cn.rs`) is a plain
    concatenator with no Tailwind conflict resolution — verified live that a
    caller's `p-0` does **not** beat the base `p-4` (the popover's computed
    padding stayed 16px), so override-style styling is unreliable here.
    Confirmed live: the field now reads `03 : 04 ⌄` (24h) and
    `03 : 04 AM ⌄` (12h), matching `DatePickerPage`'s single-field shape,
    and `date-time-picker` is unaffected (its trigger is standalone and
    stays labeled).
  - **Clock hand + drag, and a stale-rect defect it exposed (fifth review
    pass).** Added a clock hand from the dial centre to the selected value,
    and made the face draggable, not just click-to-set. Drag listens on the
    dial element itself (`onpointerdown`/`onpointermove`/`onpointerup`/
    `onpointerleave`), so it still avoids `adico-primitives`'s global
    document-level pointer registry that the original design flagged as
    unverified — no new primitive plumbing was needed after all.
    **Two real bugs found by testing this in the browser:**
    1. Dragging selected the hour labels as text. Fixed with
       `select-none` (plus `touch-none`, so a touch drag turns the hand
       instead of scrolling the page).
    2. **The dial resolved pointer positions against a stale rect.** The
       rect was captured once in `onmounted`, but `Positioner` places the
       popup *after* its content mounts, so the stored centre described
       where the dial sat *before* being positioned — measured live at
       ~169px above the true centre, which made a drag released at 3
       o'clock resolve to 5. This affected the original click-to-set too and
       is almost certainly what "not able to select all the dots" was.
       Fixed by working entirely in the dial's own coordinate space
       (`element_coordinates`, i.e. offsetX/offsetY) and measuring only the
       dial's *size* at mount — size, unlike position, does not change when
       the popup is placed. Size uses `get_scroll_size` rather than
       `get_client_rect` for the same zoom-in-transform reason documented in
       task 8.1. Verified live with real browser input (synthetic
       `PointerEvent`s carry no `offsetX`, so they cannot test this): the 0,
       6, 12 and 18 dots each select their own value, and a click between
       two labelled dots correctly yields the odd hour 7 — every value on
       the face is reachable.
  - **Jerky hand movement (sixth review pass — "clock movement is not
    smooth").** The hand was rendered from the *snapped* value, so it lurched
    a whole step at a time -- 15 degrees per hour on a 24-hour face, 30 on a
    12-hour one. Fixed by separating the two concerns the way an analog
    picker should: a `drag_degrees` signal holds the raw pointer angle while
    a drag is in flight, so the hand sweeps continuously, while the
    *committed value* still snaps. On release `drag_degrees` clears and the
    hand eases onto the snapped angle; the `transition-transform` is applied
    only when not dragging, so it never lags behind the pointer mid-drag.
    Also stopped writing the value on every `pointermove` -- it now writes
    only when the snapped value actually changes, since each write re-renders
    the whole picker (the comparison reads the value fresh via
    `current_dial_value()` rather than the render-time capture, so it cannot
    skip a needed update). Verified live by driving a real drag through 5
    degree steps and sampling the rendered rotation: the hand reported 0.7,
    5.1, 10.3, 15.6 ... 60.2 degrees -- every intermediate angle distinct
    rather than clustering on multiples of 15 -- with
    `transition-duration: 0s` during the drag, `0.15s` when idle, settling
    exactly on 60 degrees and committing `04:00`.
  - **Noted, not changed:** only even hours are *labelled* on the 24-hour
    dial (and minutes only in fives) because 24 labels of `size-7` overlap
    on one ring at this radius. All 24 hours and all 60 minutes remain
    selectable between the labels (verified above); giving every value its
    own visible target would mean a two-ring Material-style dial, which is a
    design change rather than a fix.
  - **Noted, not changed:** that same `cn` limitation means the dead `p-0` in
    `TimePickerContent`/`DateTimePickerContent` (copied from the pre-existing
    `DatePickerContent`) leaves a 16px transparent gutter, so the popup sits
    ~20px below the trigger instead of the `Positioner`'s intended 4px. It is
    invisible (the gutter is transparent), identical to `date-picker`'s
    shipped behavior, and fixing it properly means giving `cn` real
    tailwind-merge semantics — a repo-wide behavior change that warrants its
    own OpenSpec change rather than being folded in here.
- [x] 5.2 Added the `time-picker` entry to `registry/registry.json` with the
  real lowercase SHA-256 checksum, `registryDependencies: ["cn",
  "native-select", "popover", "variants"]`, no `provenance` field.
- [x] 5.3 `registry validate` / `registry build` both pass (71 item
  payloads).
- [x] 5.4 Added the `"ui/time_picker.rs"` embed arm and
  `"@adico/time-picker"` expected-item-list entry in
  `packages/adico-cli/src/main.rs`; `discovery_uses_default_and_explicit_configured_sources_without_mutation`
  passes.

## 6. Registry: `date-time-picker` item

- [x] 6.1 Authored `registry/ui/date_time_picker.rs`: `DateTimePicker`
  nests a `TimePicker` inside a `DatePicker` and combines the two halves into
  one `time::PrimitiveDateTime`. **Real bug found and fixed by this task's
  own verification, before it reached the playground:** the first draft
  derived *both* halves from the external `selected_datetime` prop on every
  change, so completing the date first computed against a still-unset time
  and reported `None` — meaning the prop never became `Some` from that call,
  so finishing the time afterward combined against a still-`None` date too.
  Neither half could ever combine. Fixed by buffering each half in its own
  `last_date`/`last_time` signal (set immediately when its own picker
  reports a value, independent of whether the combined value is complete
  yet), with a `use_effect` still resyncing both from
  `props.selected_datetime` to support a fully controlled external reset.
  Caught and fixed via the new regression test in 6.3 before ever installing
  into the playground. No shared roving-focus context was needed across the
  date and time segments — every segment renders a plain, unmanaged
  `tabindex="0"`, so native Tab order already flows from one `CollectionState`
  into the next. **A second, design-level issue found in review** (an
  advisor pass flagged it before task 9): `DateTimePickerPopover` originally
  took its own `value`/`is_12h`/`show_seconds` props duplicating what the
  consumer already passed to `DateTimePicker`, directly contradicting this
  file's own doc comment ("without a second copy... being threaded through
  by the consumer"). Fixed by moving `DateTimePickerDisplayContext`
  provision into `DateTimePicker` itself (which already computes
  `last_date`/`last_time` and holds `is_12h`/`show_seconds`), as a `Memo`
  combining the two signals reactively; `DateTimePickerPopover` now carries
  none of those three props. Also hardened the `use_effect` resync to only
  overwrite `last_date`/`last_time` when `props.selected_datetime` is
  `Some`, rather than on every change including `None` — resyncing
  unconditionally would risk wiping a buffered half if a controlled
  consumer's `on_value_change` ever re-set the prop to the same `None` it
  already held, since Dioxus's signal-change notification on an equal value
  isn't a contract this file should rely on. Documented trade-off: an
  external caller can no longer clear a fully-set value back to `None` by
  resetting the prop alone (remounting via a `key` is the escape hatch).
- [x] 6.2 Added the `date-time-picker` entry to `registry/registry.json`
  (real checksum, `registryDependencies: ["cn", "date-picker", "popover",
  "time-picker"]`). `registry validate` / `registry build` both pass.
- [x] 6.3 **Correction from the original task wording:** a live keyboard
  Tab/arrow simulation across the combined segment sequence needs a real
  browser (deferred to task 9.2's Playwright spec, same SSR-harness
  limitation already noted in task 3.4). Added
  `sequential_partial_updates_still_combine_once_both_halves_are_known`
  instead — a regression test for the buffering bug found in 6.1, proving
  the date buffered while typing survives until the time half completes too.
- [x] 6.4 **Added after a second post-implementation user review pass**
  (not part of the original task list): `DateTimePickerProps` now takes and
  forwards `time-picker`'s `view: TimePickerView` prop to its nested
  `TimePicker`, so `TimePickerBody` inside `DateTimePickerContent` renders
  the matching body without the consumer threading the choice separately.
  Confirmed live on `/date-time-picker`: switching the new "View" demo
  control to Analog renders the dial (with its AM/PM buttons) alongside the
  calendar, unchanged from the standalone `/time-picker` behavior.

## 7. Install into playground and propagate registry tooling (strict order)

- [x] 7.1 In `apps/playground`, ran `adico add time-picker` then
  `adico add date-time-picker` (through the real CLI, re-run after the 6.1
  bugfix so the installed copy matches the fixed registry source) — wrote
  `src/components/ui/{time_picker,date_time_picker}.rs`, the `// adico:start`
  region of `src/components/ui/mod.rs`, and `adico.lock`.
- [x] 7.2 `cargo run -p adico-xtask -- playground-controls sync` — generated
  `TimePickerPopoverControls`/`TimePickerPopoverDemoState` and
  `DateTimePickerPopoverControls`/`DateTimePickerPopoverDemoState` (open/
  default-open only; the `Signal`/`ReadSignal`-typed props like `disabled`,
  `is_12h`, `show_seconds` have no generated control, matching
  `DatePickerPage`'s existing pattern of hand-written `BoolControl`s for
  those).
- [x] 7.3 `primitive-usage sync` classified both new items `"delegated"`
  automatically (source contains `use_context`/`use_signal`), no hand-edit
  needed. `styling-usage sync` needed a hand-written `styleException` (the
  dial's runtime `left/top` label-position style) and four `radiusException`
  entries (trigger, input-value, columns, clock) in
  `statics/styling_usage/time-picker.json` — `date-time-picker` needed none.
  `prop-parity sync` needed no hand-edit. `primitive-usage check`,
  `styling-usage check`, `prop-parity check` all pass (69 items).
- [x] 7.4 Hand-added `("TimePicker", "time_picker.rs")` to
  `ADICO_ONLY_EXTRAS` in `packages/adico-xtask/src/primitive_compat.rs`
  before running `primitive-compat sync` — `date-time-picker` introduces no
  new primitive module (it's a pure registry-level composition of `date-picker`
  and `time-picker`), so it needed no entry. `primitive-compat check` passes.
- [x] 7.5 `component-compat sync` regenerated cleanly; `component-compat
  check` passes — no manual entry needed for either item.

## 8. Playground pages and routes

- [x] 8.1 Wrote `apps/playground/src/pages/time_picker.rs` and
  `date_time_picker.rs`, modeled on `pages/date_picker.rs` and
  `pages/color_picker.rs`: hand-written `BoolControl`s for
  disabled/read-only/12-hour/show-seconds plus the generated
  `<X>PopoverControls` panel from task 7.2. Both start at `None` ("Pick a
  time" / "Pick date & time") rather than a current-time-derived default,
  matching `DatePickerPage`'s and `ColorPickerPage`'s existing precedent —
  sidesteps the SSR-hydration concern in 8.4 entirely rather than needing an
  after-mount resolution. **Updated after task 5.1's `view` prop reversal**
  (originally demoed `TimePickerColumns` and `TimePickerClock` side by side
  inside `TimePickerContent`): both pages now add a `SelectControl { label:
  "View", ... }` bound to a `TimePickerView` signal, passed as the `view`
  prop, with the body composed via a single `TimePickerBody {}` (`{ class:
  "flex-1 min-h-0", fill_height: true }` on the DateTimePicker page, to
  preserve the height-matching fix below) instead of hardcoding one specific
  part — demoing the prop-driven switch a real consumer would use, rather
  than showing both views permanently at once.
  **Follow-up from a post-implementation user review** (`DateTimePickerPage`'s
  time-side panel was visibly shorter than the calendar, per a user
  screenshot): tried `TimePickerColumns { fill_height: true, class:
  "flex-1 min-h-0" }` alone first, relying on CSS flex stretch through the
  popup's `position: fixed` (otherwise auto-height) surface -- confirmed live
  in the browser that this does **not** resolve a definite height in that
  context; the columns rendered fully unclipped (measured 1789.5px tall),
  not matching the calendar at all. Replaced with a JS-measured explicit
  height: wrap `DatePickerCalendar` in a `div` with `onmounted`, read
  `event.data().get_scroll_size()` (not `get_client_rect()` -- the popover's
  own `data-[state=open]:zoom-in-95` entrance animation is a CSS
  `transform: scale()`, which `get_client_rect()`/`getBoundingClientRect()`
  reports mid-transition; confirmed live it returned 95% of the settled
  height, while `scrollHeight` is transform-independent), store it in a
  signal, and apply it as an explicit inline `style` on the time-side
  wrapper. A second bug surfaced while wiring this up: the measurement
  wrapper around the calendar is itself a flex item subject to the row's
  default `align-items: stretch`, so without `self-start` on it, it got
  stretched to match its (at-that-point-unclipped) sibling's huge height,
  poisoning the very measurement being taken (confirmed live: it measured
  1757px instead of the calendar's real ~330px). Fixed by adding
  `class: "self-start"` to the measurement wrapper. Final result confirmed
  live via DOM inspection: both panels measure exactly 320px. Measured once
  on mount, not re-measured on month navigation -- a 5-week vs. 6-week month
  can leave a small height mismatch, a documented trade-off, not a silent
  one.
- [x] 8.2 Updated `apps/playground/src/pages/mod.rs` in both alphabetical
  blocks (`mod`/`pub use`) for `date_time_picker` and `time_picker`.
- [x] 8.3 Updated `apps/playground/src/routes.rs` in all three
  independently-ordered spots: the rustfmt-sorted `use crate::pages::{...}`
  list, the `Route` enum (inserted next to `DatePickerPage`, thematic order),
  and `nav_items()` (alphabetical by label). `cargo fmt --all --check`
  passes.
- [x] 8.4 No current-time-derived default exists on either page: both
  initialize their signal as `None` (`use_signal(|| None::<Time>)` /
  `None::<PrimitiveDateTime>`, `pages/time_picker.rs`/`date_time_picker.rs`),
  and grepping `registry/ui/time_picker.rs` and
  `packages/adico-primitives/src/time_picker.rs` confirms `default_local_time()`
  is never called from any component body — it exists only as an exported
  helper a consumer may opt into. With no time-derived value computed during
  render, SSR and the client render the same initial markup, sidestepping
  the hydration concern rather than needing an after-mount resolution.
  `cargo check`/`cargo test -p adico-playground --features server` and
  `cargo check --target wasm32-unknown-unknown -p adico-playground --features
  web` only confirm both feature builds compile clean — they cannot and do
  not confirm the absence of a hydration mismatch, which is a runtime
  property; the source-level absence of a time-derived render is the actual
  evidence here.

## 9. Consumer fixture

- [x] 9.1 Added `tests/installation/time-picker-consumer/`, installed with the
  real CLI (`adico init` then `adico add time-picker date-time-picker`) — no
  workspace-path import. It binds a digital (columns) and an analog (dial)
  picker to **one shared signal** on purpose, so a test can drive one input
  path and assert the others converge, plus a `DateTimePicker`, with
  `#time-value`/`#datetime-value` readouts for exact assertions. `cargo check`
  passes.
  **Two fixture-only defects had to be fixed to make it usable, both worth
  recording because they will bite the next fixture that renders a popover:**
  1. The popup opened logically (`data-state=open`, correct ARIA) but stayed
     `visibility: hidden` forever. `Positioner` places an anchored popup
     through `dioxus_document::eval`, and that eval's *runtime provider* is
     only registered when `dioxus-web`'s own `document` feature is on — which
     no existing fixture enables. Fixed by adding `document` to the fixture's
     dioxus features, which in turn required pinning `dioxus-web = "=0.7.9"`
     (left unpinned it resolves to 0.7.10, which fails to compile against the
     rest of the 0.7.9 dioxus crates once `document` is enabled).
  2. Its Tailwind build emitted **no utilities from the installed components
     at all** (9KB output, no `pointer-events-none`, no `sr-only`).
     `tests/installation/.gitignore` ignores `*/src/components/`, and
     Tailwind v4 skips gitignored paths during source detection — an explicit
     `@source "./src/**/*.rs"` glob does *not* override this (tried, no
     effect). Not fixed: it is a property of how these fixtures deliberately
     avoid committing generated source. Its consequence is recorded in 9.2.
- [x] 9.2 Added `tests/playwright/time-picker.spec.ts` (5 tests, all passing
  against the fixture): typing into the segmented trigger, keyboard-only
  selection from the columns, the two pickers converging on one value,
  `DateTimePicker` combining a date and a time, and an axe check scoped to the
  segmented input and columns (not the dial, per the accepted a11y
  trade-off).
  **The clock dial's pointer coverage lives in a second spec,
  `playground-time-picker.spec.ts` (3 tests, all passing against the
  playground), not in the fixture** — because of 9.1's Tailwind finding: the
  dial relies on `pointer-events-none` to stop its hour labels and hand
  intercepting the pointer, and without that utility a real pointer lands on
  a label, so `element_coordinates` are relative to the wrong element and the
  angle is meaningless. Verified this is a fixture-stylesheet limitation and
  not a component defect by driving the same drag with correctly-offset
  synthetic events in the fixture (0 -> 3 -> 6 -> 9 -> 12, exactly right).
  That second spec covers pointer drag setting the hour, the hand tracking
  continuously rather than in snapped steps, and the hand easing onto the
  snapped value on release.
  **A third real finding came out of writing it:** committing a value changes
  the trigger field's text (`HH` becomes a number), which changes its width,
  which moves the popup's anchor — so `Positioner` shifts the popup *mid-drag*
  (~20px horizontally, measured). A test that caches the dial's box before the
  drag is therefore off by a whole snap step; the spec re-measures the dial on
  every pointer step. This is correct product behaviour (the popup follows its
  anchor), documented so the next test author does not rediscover it.
  Both specs are registered as `npm run test:time-picker` /
  `test:playground-time-picker` and documented in `tests/playwright/README.md`.

## 10. Full validation

- [x] 10.1 `cargo fmt --all --check` — passes.
- [x] 10.2 `cargo check --locked --workspace` — passes, no warnings.
- [x] 10.3 `cargo clippy --locked -p adico-cli -p adico-primitives -p
  adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D
  warnings` — passes.
- [x] 10.4 `cargo test --locked -p adico-cli -p adico-primitives -p
  adico-registry-core -p adico-test-utils -p adico-xtask` — all suites pass
  (0 failures across every crate, including the 93 `adico-primitives`
  doctests and the new `time_picker`/`date_time_picker` tests). Also ran
  `cargo test -p adico-playground --features server` (91 passed, 0 failed,
  0 warnings) even though it's outside this task's named command set, since
  the new pages/components live there.
- [x] 10.5 `registry validate`, `provenance check`, `primitive-usage check`,
  `styling-usage check`, `component-compat check`, `primitive-compat check`,
  `playground-controls check`, `prop-parity check`, `component-props check`
  — all pass.
- [x] 10.6 `cargo check --target wasm32-unknown-unknown -p adico-primitives`
  — passes. Also ran `cargo check --target wasm32-unknown-unknown -p
  adico-playground --features web` (outside this task's named command, but
  the actual consumer of the `web`-gated dial/clipboard-style DOM interop) —
  passes.
- [x] 10.7 `openspec validate add-time-picker-components --strict` — valid.
- [x] 10.8 `dx serve` in `apps/playground`; both `/time-picker` and
  `/date-time-picker` screenshotted across digital/analog and 12h/24h, and
  confirmed the popup paints over the "Component controls" card. Anchoring
  measured directly rather than eyeballed: the popup sits exactly the
  `Positioner`'s 4px offset below its trigger, and scrolling with it open
  leaves that gap unchanged (it recomputes and follows).
  **Correction to this task's own wording:** "stays anchored while panning"
  cannot be observed as written — panning starts with a pointerdown on the
  canvas background, which is an outside click, so the popover dismisses
  before any panning happens (verified live: the canvas panned, the popup was
  already closed). The underlying concern is covered by the scroll
  measurement above.
- [x] 10.9 Verified `now_local_time()` genuinely resolves the process
  timezone rather than silently falling back to UTC, by measuring the offset
  it reports under several zones (throwaway test, removed afterwards):
  `TZ=UTC` -> 0 min, `TZ=Asia/Kolkata` -> +330 min (+5:30),
  `TZ=America/Los_Angeles` -> -420 min (-7:00) — each exactly the real zone
  offset. `cargo test -p adico-primitives local_time` also passes under all
  of those zones. Note this exercises the *native* path; the browser path
  resolves through `time/wasm-bindgen`. Nothing time-derived renders in the
  playground pages themselves (see 8.4), so there is no UI default to check.
- [x] 10.10 Ran both new specs against their own apps — 5 fixture tests and
  3 playground dial tests, all passing.
  **Correction to this task's own wording:** there is no single `npm test`
  that runs this suite. Each spec targets a *different* served application
  and is run with `ADICO_PLAYWRIGHT_BASE_URL` pointed at it (the pattern
  `tests/playwright/README.md` already documents for `fullstack.spec.ts`);
  a bare `npm test` would run every spec against whichever single app
  happened to be served and fail the rest. The two new specs are documented
  in that README with their exact commands.

## 11. Close out

- [x] 11.1 Documented both new Playwright specs in
  `tests/playwright/README.md`, including why dial coverage runs against the
  playground rather than the fixture. No other doc needed changing: nothing
  under `docs/` or `README.md` enumerates the registry's component list or
  describes timezone handling (checked by grep) — the component inventory
  lives in the machine-generated, CI-gated `statics/*.json`, which are
  synced. **Deliberately not updated:** `docs/adico/m10-parity-status.md`
  carries counts that are now stale (66 items / 11 + 21 extras, vs 69 / 13 +
  25 today), but it opens by declaring itself a point-in-time snapshot dated
  2026-09-04 and tells the reader to read `statics/*.json` instead once it is
  stale. Rewriting a dated historical snapshot under a new date is an
  editorial decision beyond this task.
- [ ] 11.2 Sync delta specs into `openspec/specs/{adico-primitives,
  adico-existing-components}` and archive this change per the
  `openspec-archive-change` workflow, once all tasks above are checked and
  validation passes.
