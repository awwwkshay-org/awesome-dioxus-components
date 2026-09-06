## Context

See `proposal.md` - Why. Load-bearing facts from exploration (two independent
reads of `packages/adico-primitives/src/date_picker.rs` agree):

- `DateSegment<T: Clone + Copy + Integer + FromStr + Display>` (`date_picker.rs:688`)
  is already generic over its numeric type, parameterized on `min`/`max`/
  `max_length`/`default`/`index`. Hour/minute/second (`u8`, width 2) would work
  against its body with zero changes, **except** it is a private item.
- It pulls `BaseDatePickerContext` (`:716`), private, whose `onfocus` calls
  `ctx.set_open(false)` (`:878-880`) and which carries date-only fields
  (`enabled_date_range`, `available_ranges`) meaningless to a time field.
- `display_value` (`:703`) hardcodes `format!("{:0>width$}", ...)` — no hook
  exists to render a segment's value differently from its raw number, which
  blocks 12-hour display over a 0–23 backing value and blocks AM/PM entirely.
- The `Integer + FromStr` bound and digit-only typing (`:779`) structurally
  exclude AM/PM — it needs a new, non-numeric sibling segment, not a
  parameterization of `DateSegment`.
- ARIA is numeric-only today (`aria_valuemin/valuemax/valuenow`, `:864-866`);
  a non-numeric segment needs `aria-valuetext` instead, which does not exist
  yet.
- Segment index arithmetic is hardcoded: `ctx.start_index + 1usize` (`:984`),
  `+ 2usize` (`:1030`), `DateRangePickerEndValue { start_index: 3 }` (`:1312`).
  These bake in "exactly 3 segments per date" and must become derived.
- `DatePickerSeparator` (`:1045`) is already fully generic (`symbol: char`,
  `aria_hidden`, `tabindex="-1"`) and needs no change to serve as a time
  separator (`:`).
- `DateElement` (`:1153-1206`) — one signal per segment, an effect pushing
  external value changes down, a second effect reassembling
  (`Date::from_calendar_date`) and firing `on_*_change` — is the structural
  template a `TimeElement` mirrors, substituting `Time::from_hms`.
- `LocalDateExt::now_local_date()` (`lib.rs:511-522`) already implements
  resolve-with-UTC-fallback and is called correctly by `date_picker.rs`
  itself; it is `pub(crate)`, so nothing outside the crate can reach it or a
  time equivalent yet.
- `time`'s `Time`/`PrimitiveDateTime` types are already usable under the
  current feature set (`std, macros, parsing, local-offset`, plus `web` →
  `time/wasm-bindgen`); `formatting` is not enabled and is not needed —
  existing code hand-formats with `format!("{:0>width$}", ...)`.
- `use_move_interaction` (`move_interaction.rs:99`) already supplies element
  rect, continuous drag position, and arrow-key movement for track-style
  controls (`slider`, `color_picker`); the clock dial reuses it rather than
  needing new pointer plumbing.
- No upstream (`statics/catalogs/{shadcn,base-ui,dioxus-components,
  dioxus-primitives}.json`) has a time-picker entry — both new items classify
  as `ADICO_ONLY_EXTRA` in `packages/adico-xtask/src/primitive_compat.rs` and
  need no `SHADCN_EXCEPTIONS`/`DIOXUS_COMPONENT_EXCEPTIONS` entry (those are
  keyed by upstream slug) and no provenance record (nothing was imported).
- Registry tooling has real ordering constraints, established by tracing
  `adico-xtask`: `playground-controls sync` globs the *installed* playground
  copy and never reads `registry.json`, so it is a no-op before `adico add`
  runs; `primitive-usage check` rejects `presentational` classification for
  any source containing `use_signal`/`use_effect`/`use_context`, so the new
  items must be `delegated`, which requires the real primitive module to
  exist first; `component-compat`/`primitive-usage`/`styling-usage`/
  `prop-parity` are all regenerate-then-check (no manual per-item entry), but
  `primitive-compat`'s `ADICO_ONLY_EXTRAS` list is hand-maintained and must be
  edited before `primitive-compat sync`.

## Goals / Non-Goals

**Goals:**
- Ship `TimePicker` and `DateTimePicker` as genuinely new registry items,
  reusing existing primitives (segment/roving-focus, `Positioner`,
  `use_move_interaction`) rather than parallel reimplementations.
- Keep the accessible path (columns + segmented typing) fully capable on its
  own; the dial is additive.
- Time is always local, with the fallback behavior explicit and tested rather
  than assumed.
- Land after `demo-popup-components-behind-triggers`, whose registry fixes to
  `date_picker.rs`/`navigation_menu.rs`/etc. this change's own
  `date_picker.rs` segment-extraction work builds on top of.

**Non-Goals:**
- No change to `Positioner`, `use_move_interaction`, or any other shared
  primitive's own behavior beyond what the new segment/local-time surfaces
  require.
- No timezone *selection* UI — the requirement is "always local", not
  "configurable timezone". A consumer who needs a fixed or selectable
  timezone composes their own logic on top of the exposed `PrimitiveDateTime`/
  `OffsetDateTime` value; this registry item does not attempt it.
- No change to `DateRangePicker`'s public behavior — its segment-count
  derivation must produce the same layout it has today; this is a regression
  constraint, not a feature.

## Decisions

**Extract a minimal segment-field context rather than making
`BaseDatePickerContext` itself generic or growing it with time-specific
fields.** The date context's `enabled_date_range`/`available_ranges` and its
open-on-focus-loss behavior are genuinely date/popover-specific. Alternative
considered: add optional time-related fields to `BaseDatePickerContext` and
have time segments consume the same context — rejected, because it would
make every date consumer's context carry dead fields for a concern it doesn't
have, and would couple a time-only field to a popover-open callback it may
not want (a bare `TimePicker`, without an enclosing popover, still needs
segments to work). The extracted context carries only what both need:
`focus: CollectionState`, `disabled`, `read_only`.

**Add an `on_format_value` hook to the segment primitive rather than
special-casing 12-hour display.** `display_value`'s hardcoded
`format!("{:0>width$}", ...)` cannot express "render 13 as 1" (12-hour) or
"render 0/1 as AM/PM" (which isn't numeric at all, hence the separate segment
type below). A hook mirroring the existing `on_format_placeholder` callback
shape keeps the primitive itself timezone/format-agnostic; `TimePicker`
supplies the hour-mod-12 formatting, date segments keep their current
identity formatting by not overriding the hook.

**AM/PM is a new sibling segment type, not a `DateSegment`
parameterization.** The `Integer + FromStr` bound is load-bearing for every
existing numeric segment's digit-typing and arrow-key math; forcing a
two-value enum through it would either weaken the bound for everyone or
require a fake `Integer` impl for a type that isn't one. A dedicated
`MeridiemSegment` participates in the same `CollectionState` roving-focus
sequence and exposes `aria-valuetext` ("AM"/"PM") instead of numeric
`aria-valuenow`.

**Segment-count derivation replaces the hardcoded `+1usize`/`+2usize`/
`start_index: 3` literals with a value computed from the actual composed
segment sequence.** Alternative considered: leave `DateElement`'s literals
alone and give `TimeElement`/`DateTimeElement` their own independent literals
— rejected, because `DateTimePicker` is exactly the case where a fixed count
is wrong (seconds and meridiem are optional), and duplicating the arithmetic
in three places is how this class of bug (silently wrong focus order) hides.
`DateRangePicker`'s existing behavior is the regression test for this
change: it must still compute 6 segments (two dates) with the derived logic.

**Clock dial view built on `use_move_interaction`, not a new pointer
primitive.** It already gives element rect, continuous drag position, and
arrow-key movement — the same foundation `slider` and `color_picker` use. The
dial converts pointer-position-relative-to-center into an angle, then snaps to
the nearest valid hour/minute. No ARIA pattern exists for an analog clock
(same situation `move_interaction.rs`'s own header comment records about
itself), which is precisely why columns, not the dial, are the default and
the accessible path — this is a spec requirement (see specs/adico-existing-components),
not left as an implementation choice.

**`now_local_time()`/`now_local_datetime()` are added as public siblings to
`LocalDateExt::now_local_date()`, not a new module.** Same fallback shape,
same crate. Whether the underlying `local-offset` resolution actually
succeeds outside wasm (native/SSR) depends on `time`'s `unsound_local_offset`
gating in multithreaded processes — this is confirmed empirically as the
first implementation task (a `#[test]` probing `OffsetDateTime::now_local()`
on this workspace's actual native target), not assumed either way, and the
result is documented at the call site regardless of outcome. The **SSR/
hydration constraint this creates**: a value rendered from "now" during
server-side rendering and recomputed on the client after hydration will
disagree if either side's resolution differs — so any current-time-derived
default (e.g. `TimePicker`'s value when unset) SHALL resolve after mount
(client-side) or be seeded from an explicit prop, never computed during
initial/SSR render. `examples/basic-ssr` is the fixture that exercises this.

## Risks / Trade-offs

- **[Risk] Segment-count derivation could silently regress `DateRangePicker`
  if the new derivation doesn't reduce to the same 6-segment layout** →
  Mitigation: task list requires an explicit `DateRangePicker` regression
  test asserting unchanged segment count/focus order before this is
  considered done, not just "existing tests still pass" (existing tests may
  not have covered this specifically).
- **[Risk] `now_local_time()` may always fall back to UTC on native/SSR if
  `local-offset` needs `unsound_local_offset` to succeed** → Mitigation: this
  is exactly why task 1 (the empirical probe) runs before any spec language
  or UI work locks in an assumption; if native/SSR is UTC-only in practice,
  that is documented plainly rather than silently promised as "always local".
- **[Risk] An analog dial with no ARIA pattern could become a second,
  divergent way to set a value that assistive tech can't reach** →
  Mitigation: the columns default and the requirement that the dial never be
  the only path are spec-level (not just design intent), and
  `tests/playwright` axe coverage for the columns view and segmented input is
  a named verification step.
- **[Trade-off] Extracting the segment context is an internal breaking change
  to `date_picker.rs`'s private structure** → Accepted: no public
  `date-picker` API changes, and `DateRangePicker`'s regression test is the
  safety net; the alternative (leaving `DateSegment` private and duplicating
  its ~200 lines of roving-focus/auto-advance/ARIA logic for time) is a
  larger, more error-prone surface to maintain long-term.

## Migration Plan

Purely additive at the registry-item level — no existing consumer is
affected until they choose to `adico add time-picker` /
`date-time-picker`. The internal `date_picker.rs` segment refactor has no
public API surface change, verified by the `DateRangePicker` regression test
and the existing date-picker test suite passing unmodified. Rollback, if
needed: revert the registry source and `packages/adico-primitives` changes
and re-run `registry build`; no data migration exists.
