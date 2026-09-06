## 1. Clipboard primitive

- [x] 1.1 Added `packages/adico-primitives/src/clipboard.rs`: `ClipboardStatus`
  enum (`Idle | Copied | Failed`), `use_clipboard() -> (Memo<ClipboardStatus>, Callback<String>)`,
  target-gated `copy_to_clipboard` (`web`: `document::eval` calling
  `navigator.clipboard.writeText`, reporting success/failure back via
  `dioxus.send`/`.recv::<bool>()`; every other target: always `false`), using
  `crate::time::sleep` for the auto-revert delay. Declared in `lib.rs`.
  Verify: `cargo test -p adico-primitives` covers `ClipboardStatus::default()`
  and the non-`web` `copy_to_clipboard` branch (manually polled with
  `Waker::noop()`, since it has no `.await` point) — the full Idle → Copied/
  Failed → Idle transition and the real `document::eval` path need a live
  Dioxus runtime + browser this crate has no headless harness for; verified
  live instead (see task 3.2/4.4 below).
- [x] 1.2 `cargo check --target wasm32-unknown-unknown -p adico-primitives`
  — the `web`-gated `document::eval` path compiles.

## 2. Registry: `copy-button` item

- [x] 2.1 Authored `registry/ui/copy_button.rs`: `CopyButton` composing
  `use_clipboard`, `adico_primitives::icons::{Check, Copy}`, the installed
  `button` item (`ButtonVariant::Ghost`, `ButtonSize::IconSm`), showing the
  checkmark while `ClipboardStatus::Copied`, an accessible label reflecting
  idle/copied/failed state.
- [x] 2.2 Added the `copy-button` entry to `registry/registry.json` (real
  checksum, no `provenance`, classified `ADICO_ONLY_EXTRA`). Also added the
  new `ui/copy_button.rs` match arm to `packages/adico-cli/src/main.rs`'s
  hand-maintained embedded-file whitelist — **a load-bearing detail not
  originally called out in this change's design/tasks**: the CLI's
  `RegistryFileReader` matches embedded sources by literal file-path string,
  not derived from `registry.json`, so `adico add copy-button` silently fails
  with "this adico binary does not embed the requested registry source"
  without this addition. Also had to add `"@adico/copy-button"` to a
  hand-written expected-item-list assertion in
  `packages/adico-cli/src/main.rs`'s
  `discovery_uses_default_and_explicit_configured_sources_without_mutation`
  test — another hardcoded list that breaks on any new registry item.
  `registry validate` passes.

## 3. ThemeBuilder gets a working copy button

- [x] 3.1 Added `copy-button` to `theme-builder`'s `registryDependencies`.
  Added `CopyButton { value: css_export.clone() }` next to the CSS-export
  `<textarea>` in `registry/ui/theme_builder.rs`, replacing the `<label>`
  wrapper with a `div` (the textarea keeps its own `aria-label`, so the
  `<label>` association was redundant once a second, non-field control needed
  to sit beside it). Updated checksum, `registryDependencies`, and the
  compositionNote's now-inaccurate "no browser-interop, no new cargo
  dependencies" claim.
- [x] 3.2 Verified live: `dx serve`, `/theme-builder` → opened the dialog,
  clicked the copy control, then read the **real macOS system clipboard**
  via `pbpaste` (not just in-page `navigator.clipboard.readText()`, which
  hung on a permission prompt in the automated browser) — it contained the
  exact CSS export text.

## 4. ColorPicker: `ColorPickerFields`

- [x] 4.1 Added `ColorPickerFields` to `registry/ui/color_picker.rs`: local
  `format: Signal<ColorFormat>` (`Hex | Rgb | Hsl`, defaulting to `Hex`), a
  compact `NativeSelect`/`NativeSelectSize::Sm` to switch format, matching
  field(s) per format built on `palette`'s `Hsl`/`Srgb`/`FromColor`/
  `IntoColor` (HSL/RGB) and a hand-rolled hex parse/format mirroring
  `ColorPickerSwatch`'s existing `UpperHex` formatting, plus a `CopyButton`
  for the current formatted text. Fields commit on `onchange` (blur/Enter),
  not `oninput` — a refinement over the design's originally-stated `oninput`:
  a controlled input re-synced from context every render fights per-keystroke
  `oninput` edits, while `onchange` gives the user room to finish typing
  before it's parsed and pushed. Added `copy-button` and `native-select` to
  `color-picker`'s `registryDependencies`; updated the checksum.
- [x] 4.2 Verified live end-to-end in a real browser rather than only via
  unit tests (SSR-render assertions can't exercise live two-way sync): typed
  `0` into the HSL hue field and confirmed the swatch, hue-slider handle, and
  color area all updated together; switched HEX → RGB → HSL and confirmed
  each showed the same color correctly converted (`#9B80FF` ↔ `155 128 255`
  ↔ `253 100% 75%`); confirmed an existing `ColorPicker` composed without
  `ColorPickerFields` (`wave5-color-picker-consumer`'s own flat composition,
  untouched) is unaffected — its Playwright spec still passes 2/2.
- [x] 4.3 `registry validate`, `registry build` — passed (69 item payloads).

## 5. Propagate and wire into the playground

- [x] 5.1 Reinstalled `copy-button`, `color-picker`, and `theme-builder`
  through the `adico` CLI into `apps/playground`, `examples/basic-spa`,
  `examples/basic-ssr`, and `wave5-color-picker-consumer`.
- [x] 5.2 Added `ColorPickerFields {}` to
  `apps/playground/src/pages/color_picker.rs`'s popup content.
- [x] 5.3 `component-compat sync`, `primitive-usage sync` (both new items
  auto-classified `delegated` — correct, since both have real `use_signal`/
  `spawn` logic), `styling-usage sync`, `prop-parity sync`. `primitive-compat
  sync` — hand-added `("Clipboard", "clipboard.rs")` to `ADICO_ONLY_EXTRAS`
  in `packages/adico-xtask/src/primitive_compat.rs` first, as required.
- [x] 5.4 `playground-controls check` passed with no new generated files —
  neither `CopyButton` (`value: ReadSignal<String>` only) nor
  `ColorPickerFields` (`class: Option<String>` only) has a qualifying prop
  for the control-panel generator.

## 6. Full validation

- [x] 6.1 `cargo fmt --all --check` — clean.
- [x] 6.2 `cargo check --locked --workspace` — clean.
- [x] 6.3 `cargo clippy --locked -p adico-cli -p adico-primitives -p
  adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D
  warnings` — clean.
- [x] 6.4 `cargo test --locked -p adico-cli -p adico-primitives -p
  adico-registry-core -p adico-test-utils -p adico-xtask` — all pass
  (including a hardcoded item-list test in `adico-cli` that needed updating,
  see task 2.2). Also ran `cargo check -p adico-playground` and
  `cargo check -p adico-playground --features server` — clean.
- [x] 6.5 `registry validate`, `provenance check`, `primitive-usage check`,
  `styling-usage check`, `component-compat check`, `primitive-compat check`,
  `prop-parity check`, `playground-controls check` — all pass.
- [x] 6.6 `openspec validate add-color-format-fields-and-copy-button --strict`
  — valid.
- [x] 6.7 Verified live in a real browser (see tasks 3.2 and 4.2 for the
  specific interactions and clipboard confirmations).
- [x] 6.8 `wave5-color-picker.spec.ts` against the reinstalled
  `wave5-color-picker-consumer` — 2/2 passed, no regression from
  `ColorPickerFields`/`CopyButton`.

## 7. Close out

- [ ] 7.1 Sync delta specs into `openspec/specs/{adico-primitives,
  adico-existing-components}` and archive this change — left for the user to
  trigger explicitly, per this repo's OpenSpec workflow.
