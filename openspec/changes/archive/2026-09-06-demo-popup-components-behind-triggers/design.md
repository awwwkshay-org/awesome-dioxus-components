## Context

See `proposal.md` - Why. Relevant existing mechanics:

- `packages/adico-primitives/src/positioner.rs`'s `Positioner` renders every
  anchored popup (`popover`, `tooltip`, `hover-card`, `select`, `combobox`,
  `dropdown-menu`, `context-menu`, `menubar`, `navigation-menu`) with an inline
  `position: fixed; left; top; visibility` style. `apps/playground/tailwind.css`'s
  `.adico-date-picker-popover` rule uses `!important` to override this for
  DatePicker only, which is why it is the sole overlay clippable by ancestor
  scroll and detachable from its trigger.
- `apps/playground/src/components/demo.rs` already establishes correct
  stacking for every other popup: the preview canvas is `relative z-20`, the
  "Component controls" card is `z-10`. Any `z-50` popup rendered inside the
  canvas already paints over the controls. No change to `demo.rs` is needed.
- `registry/ui/date_picker.rs` already has the pattern this change reuses for
  ColorPicker: a `#[props(default = PopoverRoot)] pub popover_root: fn(PopoverRootProps) -> Element`
  injection point so the styled popover root is swappable, consumed by the
  primitive at `packages/adico-primitives/src/date_picker.rs:421-433`.
- `packages/adico-primitives/src/lib.rs`'s `LocalDateExt::now_local_date()`
  already implements local-time resolution with a UTC fallback
  (`OffsetDateTime::now_local().map(...).unwrap_or_else(|_| ...now_utc()...)`),
  and `packages/adico-primitives/src/date_picker.rs` already calls it
  correctly for its own "today" defaults. It is `pub(crate)`, so
  `pages/calendar.rs` — playground code in a separate crate that merely
  *depends on* `adico-primitives` — cannot call it across the crate boundary.
  See Decisions.

## Goals / Non-Goals

**Goals:**
- Every popup-family playground route opens via a real trigger, with the
  trigger showing the current value where one exists.
- DatePicker (and every other anchored popup) is positioned exclusively by
  `Positioner`, with no app-level CSS overriding it.
- Three stale/missing z-index class strings in the registry are corrected.
- `pages/calendar.rs`'s "today" is timezone-correct.
- Every change is additive to registry component APIs (new exported parts),
  never a breaking prop change.

**Non-Goals:**
- No changes to `adico-primitives::positioner` itself — the mechanism is
  already correct; only the one CSS override fighting it is removed.
- No new registry item (TimePicker/DateTimePicker) — that is a separate,
  larger change (`add-time-picker-components`) sequenced after this one, since
  it touches the same `date_picker.rs` surface and its own segment-extraction
  work should build on this change's corrected baseline rather than race it.
- No change to Calendar's or ThemeBuilder's own prop surface — only how the
  playground demos them.

## Decisions

**ColorPicker trigger shape mirrors DatePicker's, not a new pattern.**
`ColorPickerPopover` takes the same `popover_root` injection prop as
`DatePickerPopover`, so a consumer (or the playground) can compose a styled
popover root without `adico-primitives` needing to know about `PopoverRoot` at
all. Alternative considered: giving `ColorPicker` its own bespoke
open/close signal wired directly to `Popover`'s primitive context, bypassing
the injection prop. Rejected — it would diverge from the one established
precedent in this codebase for exactly this composition, for no benefit.

**The swatch fill is an inline style, not a Tailwind class.** The color is a
runtime `Hsv` value; Tailwind only emits classes it can see statically in each
consuming project's own `src/`, so a class-based approach would require every
consumer's Tailwind config to special-case dynamic colors. `ColorPickerSwatch`
renders `style: format!("background-color: {}", ...)` from
`ColorPickerContext::color()`, converted through the crate's existing color
conversion path (the same one `color_name` already uses), with the swatch's
non-color chrome (`border`, `rounded`, sizing) staying ordinary Tailwind
classes.

**`now_local_date()` visibility: make it `pub` in `adico-primitives`, rather
than reimplementing the fallback expression in playground code.**
`packages/adico-primitives/src/date_picker.rs` already calls
`OffsetDateTime::now_local_date()` correctly (`date_picker.rs:458,462,942,967,1002`)
via the crate-internal `LocalDateExt` trait — it has no bug to fix. The gap is
that `pages/calendar.rs` lives in a *different* crate (the playground
consumes `adico-primitives` as a dependency) and so cannot reach a
`pub(crate)` item at all. Making the extension trait (or a plain free
function wrapping it) `pub` lets playground code call the same
correctness-sensitive local-then-UTC-fallback expression instead of
maintaining its own copy. Alternative considered: give `pages/calendar.rs`
its own local copy of `OffsetDateTime::now_local().unwrap_or_else(...)` —
rejected, since this crate is exactly the place such a helper belongs (see
`packages/adico-primitives/src/time.rs`'s own doc comment: "target-aware
timing support for owned primitives") and duplication risks the two copies
drifting.

**Registry class-string fixes are corrected in `registry/ui/*.rs` directly,
not worked around in the playground.** Per the existing, unmodified
requirement "Registry components are never modified solely for playground's
convenience" (`adico-playground-structure`), these are genuine rendering
defects independent of playground — `NavigationMenuContent` has no z-index at
all, `DatePickerContent`'s `z-[1000]`/`adico-date-picker-popover` do nothing
now that `Positioner` owns positioning, and the `dropdown_menu`/`menubar`
`absolute` classes are dead since both route through `MenuContent` →
`Positioner`. Each is a one-line class-string edit with an existing
`#[cfg(test)]` assertion (`dropdown_menu.rs:410-412`) updated in the same
commit.

**`menubar.rs` does not get an open-state control (found during
implementation, not anticipated at proposal time).**
`packages/adico-primitives/src/menubar.rs`'s `MenubarProps` has no `open`/
`default_open`/`on_open_change` prop — which menu, if any, is open is private
context state (`open_menu: Signal<Option<usize>>` on a menubar-scoped
context) shared across sibling `MenubarMenu`s, not surfaced as a controllable
prop at all. This is unlike every other popup-family item in scope (Popover,
Dialog, Sheet, Drawer, AlertDialog, DropdownMenu, ContextMenu, Select,
Combobox, HoverCard, Tooltip), all of which expose controllable open state.
Adding one to `Menubar` purely so its playground page can have a matching
checkbox would be exactly the playground-convenience-only registry change
the existing, unmodified "Registry components are never modified solely for
playground's convenience" requirement forbids — it is not a confirmed
rendering or behavioral defect, `Menubar` already works correctly via its
own trigger clicks. `pages/menubar.rs` is left unchanged; the spec
requirement is scoped to components with controllable open state
accordingly. If a future need for controlled Menubar state arises
independent of this playground checkbox, it should be proposed as its own
change against `adico-primitives`, not folded in here.

**Theme-builder page reuses `ThemeBuilderLauncher`, not a new trigger.**
`apps/playground/src/components/theme_builder_launcher.rs` already
Dialog-wraps `ui::ThemeBuilder {}` and is used in the sidebar footer. Writing
a second Dialog-trigger wrapper on the page would duplicate it for no reason;
rendering the existing launcher on the page is a one-line change.

## Risks / Trade-offs

- **[Risk] Removing `.adico-date-picker-popover`'s CSS regresses DatePicker's
  positioning if `Positioner` has an edge case the CSS was silently
  papering over** (e.g. the previously-documented visibility-stuck-hidden bug,
  now fixed, per `positioner.rs`'s own comment) → Mitigation: this is exactly
  why `dx serve` + screenshot verification of `/date-picker` under panning and
  ancestor-scroll is called out explicitly in the plan's verification section,
  not left to automated checks alone.
- **[Risk] Reinstalling ColorPicker through the CLI into
  `wave5-color-picker-consumer` could desync from `tests/playwright/wave5-color-picker.spec.ts`
  if that spec asserts on the current DOM shape** → Mitigation: run that spec
  as part of verification; it is named explicitly in the proposal's Impact
  section rather than left to a generic "run playwright" step.
- **[Trade-off] Keeping Calendar's flat tree *and* adding a popover instance
  means the page renders the calendar twice** → Accepted: the alternative
  (converting fully) would hide Calendar's own month/year-navigation props
  behind a click on every page load, regressing the existing "Playground
  exposes chosen component controls" requirement for exactly the props this
  page exists to demonstrate.

## Migration Plan

No consumer-facing migration: every registry change is additive (new exported
parts, corrected class strings, no prop removed or renamed). Existing
consumers who installed `color-picker`, `navigation-menu`, `date-picker`,
`dropdown-menu`, or `menubar` before this change are unaffected until they
re-run `adico add` to pick up the fix; nothing breaks if they don't.
Rollback, if needed, is reverting the registry source and re-running
`registry build` — there is no data migration.
