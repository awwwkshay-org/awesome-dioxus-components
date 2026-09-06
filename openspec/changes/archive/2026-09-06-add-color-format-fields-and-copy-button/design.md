## Context

See `proposal.md` - Why. The reference screenshots that prompted this change
turned out to be Chrome's own native OS color-picker popup (opened from
`ThemeBuilder`'s `<input type="color">`), not app code — there is no existing
format-select implementation anywhere in this repo to extract; this is new
work built on top of already-existing pieces:

- `ColorPickerContext` (`packages/adico-primitives/src/color_picker.rs:205-248`)
  already exposes `color() -> Hsv<encoding::Srgb, f64>` and `set_color`,
  which `ColorArea`, `HueSlider`, and (from `demo-popup-components-behind-triggers`)
  `ColorPickerSwatch` already read/write. `ColorPickerFields` is another
  consumer of the same context — no context changes needed.
- The `palette` crate (already a dependency, per `color-picker`'s
  `cargoDependencies`) provides `Hsl`/`Srgb`/`FromColor`/`IntoColor`
  directly — HSL math does not need hand-rolling the way
  `theme_builder.rs`'s private `hsl_to_hex`/`hex_to_hsl` do for their own
  `"H S% L%"` CSS-string token representation (a different, incompatible
  representation this change does not touch or reuse).
- `packages/adico-primitives/src/persisted_state.rs:104-157` is the estab
  lished pattern for target-gated `document::eval` browser interop
  (`#[cfg(feature = "web")]` real implementation via `dioxus_document::eval`
  with `dioxus.send`/`dioxus.recv`, other targets get an explicit
  lesser-capability path, never a silent pretend-success). The removed
  `apps/playground/src/theme.rs::copy_theme_css` (git history only, commit
  `c4682d8`) used raw `web_sys`/`wasm_bindgen_futures` directly from
  application code — exactly the shape `openspec/changes/archive/2026-09-05-build-adico-component-ecosystem/design.md`
  §7d says this architecture forbids; this change's `document::eval` approach
  is the correct replacement, not a repeat of the same mistake.
- Lucide's `Check` and `Copy` icons already exist in the `dioxus_icons::lucide`
  re-export this crate uses (`adico_primitives::icons::{Check, Copy}`) — no
  new icon dependency.
- `packages/adico-primitives/src/time.rs`'s target-gated `sleep()` (already
  `pub(crate)`, usable within the crate) is reused for the copy-confirmation
  auto-revert delay, rather than a second timer mechanism.

## Goals / Non-Goals

**Goals:**
- A generic, reusable `CopyButton` other components (not just ColorPicker)
  can compose — proven by using it to fix `ThemeBuilder`'s real, pre-existing
  gap in the same change.
- `ColorPickerFields` supports all three formats the user asked for (HEX,
  RGB, HSL), matching the interaction shape of the native browser color
  picker that inspired the request (a compact format-select plus fields,
  values editable in either direction).
- Composable, not automatic — no existing `ColorPicker` composition changes
  behavior.

**Non-Goals:**
- No native-target (desktop/mobile) clipboard support. This workspace has no
  clipboard crate dependency today (confirmed: no `arboard` or equivalent),
  and adding one is a real dependency decision this change does not make
  unilaterally. Native/SSR targets get an honest "failed" status, matching
  the removed `theme.rs` precedent's own non-wasm branch, which reported
  unavailability rather than faking success.
- No format beyond HEX/RGB/HSL (no HSV display, no CSS `oklch()` display,
  etc.) — the three requested formats only.
- Does not touch `theme_builder.rs`'s own separate `hsl_to_hex`/`hex_to_hsl`
  per-token color editor — that is a different, CSS-token-string
  representation serving a different purpose (semantic theme tokens vs. a
  single picked color), and reconciling the two is out of scope here.

## Decisions

**`use_clipboard()` returns a status enum (`Idle | Copied | Failed`) with a
built-in auto-revert timer, not a bare `Result` the caller must debounce
itself.** Every real caller (a copy button) needs the same "show confirmation
for ~1.5s then revert" behavior; putting the timer in the hook means
`CopyButton` and any future caller don't duplicate it. Alternative
considered: return `Callback<String> -> impl Future<Output = bool>` and let
each call site manage its own timeout — rejected, duplicative for no benefit
given there is exactly one interaction shape every caller wants.

**`ColorPickerFields`' format state (`Hex | Rgb | Hsl`) is local `use_signal`
state inside the component, not part of `ColorPickerContext`.** It is a
display preference of this one composed part (which format is currently
shown), not shared color state — nothing else needs to know or react to it.
Alternative considered: thread it through the shared context so multiple
`ColorPickerFields` instances could stay in sync — rejected as unnecessary
generality; no scenario in this proposal needs two format-selectors on the
same picker to agree.

**HEX parsing/formatting is hand-rolled (strip `#`, parse 3 byte-pairs via
`u8::from_str_radix`), not a new crate dependency.** `ColorPickerSwatch`
(from `demo-popup-components-behind-triggers`) already formats hex this way
via `Srgb<u8>`'s `UpperHex` impl; parsing the same shape back is a few lines
and avoids a new dependency for a solved problem.

**The format-select control is a real `<select>` (this registry's existing
`NativeSelect`/`NativeSelectOption`), styled compact, not a custom
div-based cycling toggle.** The user's "like native" placement answer is
read as an interaction/layout preference (compact, chevron-style, sitting
right below the area/slider) — the accessibility posture of building it on a
real native `<select>` (already this codebase's established pattern for
every other format/option toggle, e.g. `SelectControl` in the playground
controls) is not something this change trades away for a closer visual
mimicry of the browser's own picker chrome, which is OS-drawn and not
something a web page can literally reproduce anyway.

**Numeric/text fields update the color on `oninput`, ignoring unparseable
intermediate states rather than clamping mid-keystroke.** Matches the
lightweight, low-machinery bar this component sets for itself (no new
roving-focus/segment primitive, unlike `DatePicker`'s segments) — a full
segmented-input treatment is out of scope for a picker's format display,
which is closer to `theme_builder.rs`'s existing plain-`<input>` per-token
editors than to `DatePicker`'s spinbutton segments.

## Risks / Trade-offs

- **[Risk] `navigator.clipboard.writeText` requires a secure context (HTTPS
  or localhost) and can be denied without a page-visible permission prompt in
  some browser configurations** → Mitigation: the hook's "Failed" status
  exists precisely for this; `CopyButton`'s spec requirement explicitly
  covers "the copy fails" as a first-class scenario, not an afterthought.
- **[Trade-off] No native clipboard support** → Accepted per Non-Goals;
  documented plainly in the primitive's own doc comment so a future change
  adding one knows exactly what gap it's filling.
- **[Risk] Two incompatible color-format machineries now exist in this file
  family** (`theme_builder.rs`'s HSL-string/hex token editor vs.
  `color_picker.rs`'s new palette-crate-based HEX/RGB/HSL fields) → Accepted:
  reconciling them would mean migrating `ThemeBuilder`'s token storage
  representation, a much larger and riskier change with no requested benefit
  right now; noted here so a future change doesn't attempt to unify them
  without weighing that cost first.

## Migration Plan

Purely additive: `copy-button` is a new registry item; `ColorPickerFields` is
a new, non-default composable part; `ThemeBuilder`'s only change is one new
element next to its existing export textarea. No existing consumer's
rendered output changes unless they explicitly add the new parts. Rollback:
revert the registry/primitives source and re-run `registry build`; no data
migration.
