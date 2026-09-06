## Why

The custom `ColorPicker` (the `ColorArea`/`HueSlider` composition, distinct
from a native `<input type="color">`) lets a user pick a color visually but
gives no way to read or type its value as HEX/RGB/HSL text, and no way to
copy that value out. Every real color-picker UI (including the browser's own
native picker, which is what inspired this request) pairs the visual picker
with format-aware numeric/text fields. Separately, nothing in this registry
can copy text to the clipboard at all: `ThemeBuilder`'s generated CSS export
is a read-only `<textarea>` a user must manually select and copy, a gap noted
but deliberately deferred when `ThemeBuilder` was built (see design.md).

## What Changes

- **New `adico-primitives` module `clipboard`**: `use_clipboard()`, a
  target-gated hook following the same `document::eval` pattern already
  established by `persisted_state.rs` for browser interop. On the `web`
  target it calls `navigator.clipboard.writeText` and reports success or
  failure; on every other target (native desktop/mobile, SSR) it reports
  failure rather than silently pretending to work, since no native clipboard
  integration exists in this workspace yet. Browser-interop detail stays
  behind this primitive adapter, per this repo's architecture rule.
- **New registry item `copy-button`**: a small, generic, reusable
  `CopyButton` — pass it a text value, it copies it on click and shows a
  brief checkmark confirmation. No shadcn/upstream equivalent to mirror;
  classified `ADICO_ONLY_EXTRA` like `theme-builder`.
- **`ThemeBuilder` gains a `CopyButton` next to its CSS-export `<textarea>`**
  — a real, cited fix for the "no copy button" gap noted above, composing the
  new registry item rather than reimplementing clipboard access inline.
- **`ColorPicker` gains a new composable part, `ColorPickerFields`**: a
  compact format-select (HEX / RGB / HSL) plus the matching numeric/text
  input(s) for the selected format, staying in sync with the same
  `ColorPickerContext` `ColorArea`/`HueSlider`/`ColorPickerSwatch` already
  read and write. Includes a `CopyButton` for the current formatted value.
  Composable, not automatic: existing `ColorPicker` usages (including the
  ones added by `demo-popup-components-behind-triggers`) are unaffected
  unless a consumer adds `ColorPickerFields`.
- Playground: `pages/color_picker.rs`'s popup gains `ColorPickerFields`;
  `ThemeBuilderLauncher`'s CSS export gains the copy button.

## Capabilities

### New Capabilities
(none — additive parts on existing capabilities)

### Modified Capabilities
- `adico-primitives`: adds the target-gated clipboard-copy primitive.
- `adico-existing-components`: adds the `copy-button` registry item and the
  `ColorPickerFields` part on `color-picker`; fixes `theme-builder`'s missing
  copy affordance on its CSS export.

## Impact

- `packages/adico-primitives/src/clipboard.rs` (new)
- `registry/ui/copy_button.rs` (new), `registry/ui/color_picker.rs`,
  `registry/ui/theme_builder.rs`, `registry/registry.json`,
  `registry/generated/**`
- `apps/playground/src/pages/color_picker.rs`,
  `apps/playground/src/components/theme_builder_launcher.rs` (or
  `theme_builder.rs` itself, wherever the CSS export renders — confirmed in
  design.md)
- Installed copies refreshed through the `adico` CLI in `apps/playground`,
  `examples/basic-{spa,ssr}`, and `wave5-color-picker-consumer`
- `statics/{component_compatibility.json,primitive_compatibility.json,
  primitive_usage,styling_usage,prop_parity}` entries for `copy-button` and
  the changed `color-picker`/`theme-builder` items
- No new Cargo dependency (uses `document::eval`, not a clipboard crate); no
  native-target clipboard support in this change (see design.md's Non-Goals).
