## Why

No component in this registry lets a consumer pick a time, and no upstream
this project mirrors (shadcn, Base UI, dioxus-components, dioxus-primitives)
has one either — confirmed against all four `statics/catalogs/` snapshots.
Consumers who need one today have nothing to install. The user asked for a
`TimePicker` that shows the selected time in its trigger and lets a user set
it either by typing (a segmented field, matching how `DatePicker` already
works) or through a popup offering both an analog clock and scrollable
hour/minute columns, plus a `DateTimePicker` combining it with the existing
`DatePicker` — with time always shown and interpreted in the local timezone of
the device running the UI, never a fixed zone.

## What Changes

- **New registry item `time-picker`**: a styled `TimePicker` composing a new
  `adico-primitives` time-picker primitive. Its trigger renders the selected
  time as segmented text (matching `DatePicker`'s composition pattern); its
  popup offers a columns view (scrollable hour/minute/second/meridiem lists)
  and a clock-dial view, switchable via a `view: TimePickerView` prop
  (`Columns | Clock`) that **defaults to `Columns`**, because an analog dial
  has no ARIA pattern to key keyboard/AT interaction off of — columns are the
  accessible path, the dial is pointer enhancement over the same value, never
  the only way to set it.
- **New registry item `date-time-picker`**: composes registry `date-picker` +
  `time-picker` behind one trigger showing the full formatted value.
- **`adico-primitives` gains a public local-time-with-fallback surface**:
  extending the existing (currently `pub(crate)`) `LocalDateExt` pattern with
  time/datetime equivalents, so every consumer of "now" in this ecosystem —
  the new time-picker's default value and `apps/playground`'s own demo
  defaults — computes it identically. On wasm with the `web` feature, local
  time resolves through the browser (`time/wasm-bindgen`); resolution on
  native/SSR targets is confirmed empirically as part of this change (see
  design.md) and falls back to UTC if the platform cannot supply a sound local
  offset — this fallback is explicit and documented, not silent.
- **A previously private, date-specific numeric segment field
  (`DateSegment`/`DateSegmentProps` in `packages/adico-primitives/src/date_picker.rs`)
  is extracted into a shared, reusable segment primitive** so hour/minute/
  second segments can compose the same roving-focus, auto-advance, and
  spinbutton-ARIA behavior `DatePicker`'s day/month/year segments already
  have, plus a new non-numeric AM/PM segment for 12-hour display. Per-field
  segment-count arithmetic (currently hardcoded literals) becomes derived, so
  both the existing `DateRangePicker` and the new `DateTimePicker`'s combined
  date+time field compute their segment layout correctly.
- Playground gains `time-picker` and `date-time-picker` pages/routes, and a
  consumer fixture under `tests/installation/` installed through the real
  CLI.
- **BREAKING (internal only)**: `DateSegment`/`DateSegmentProps` change from
  private, picker-context-coupled items to a smaller, generalized context.
  No public `date-picker` registry API changes; `DateRangePicker` behavior is
  unchanged and covered by regression tests.

## Capabilities

### New Capabilities
(none — both new registry items are additive instances of the existing
"registry item" and "primitive surface" capabilities, not a new kind of
capability)

### Modified Capabilities
- `adico-primitives`: adds a shared segmented numeric/enum field primitive
  (generalizing `date_picker`'s existing one) and a public local-time
  resolution surface with an explicit UTC fallback.
- `adico-existing-components`: adds the `time-picker` and `date-time-picker`
  registry items, their trigger/value-display and accessible-default-view
  behavior, and their local-timezone-only time interpretation.

## Impact

- `packages/adico-primitives/src/{date_picker.rs,lib.rs}` (segment extraction,
  local-time helpers), new `packages/adico-primitives/src/time_picker.rs`
- `registry/ui/time_picker.rs`, `registry/ui/date_time_picker.rs` (new),
  `registry/registry.json`, `registry/generated/**`
- `statics/{component_compatibility.json,primitive_compatibility.json,
  primitive_usage,styling_usage,prop_parity}` entries for both new items
  (both classify as `ADICO_ONLY_EXTRA` — no upstream counterpart, no
  provenance record required)
- `apps/playground/src/{pages/time_picker.rs,pages/date_time_picker.rs,
  pages/mod.rs,routes.rs}` and generated control panels
- A new `tests/installation/*` consumer fixture, installed via the CLI
- No CLI or registry schema changes. No new Cargo dependency — `time`'s
  `Time`/`PrimitiveDateTime` types are already available under the existing
  feature set.
- **Sequencing**: this change touches `registry/ui/date_picker.rs`, which
  `demo-popup-components-behind-triggers` also edits (registry defect fixes).
  This change is authored to apply after that one lands, so the segment
  extraction starts from its corrected baseline rather than racing it.
