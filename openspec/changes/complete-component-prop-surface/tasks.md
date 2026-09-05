## 1. Shared prop vocabulary

- [x] 1.1 Add `registry/lib/variants.rs` (new `registry:lib` item,
      alongside `cn`): `pub enum Radius { None, Sm, Default, Lg, Xl, Full }`
      with `#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]`,
      `#[default]` on `Default`, and `impl Radius { pub fn class(self) ->
      &'static str }` mapping to the corresponding `rounded-*` Tailwind
      utility (`Default` maps to the bare `rounded-lg` class tracking the
      `--radius` token, matching `theme_builder.rs`'s existing default).
      Register it in `registry/registry.json`, with its `files[0].checksum`
      set to the file's real `sha256` hex digest (e.g. `shasum -a 256
      registry/lib/variants.rs`) — checksums in `registry.json` are
      hand-maintained, not computed by `registry build`, and a stale one
      fails install/`adico add` even though `registry validate` stays
      silent about it (no `registryDependencies` of its own, matching
      `cn`'s entry shape). Add a matching `("lib/variants.rs",
      RegistryLocation::Embedded)` arm to `ConfiguredRegistryReader::read`
      in `packages/adico-cli/src/main.rs` (the embedded-registry file
      reader is an explicit per-file `include_bytes!` allowlist, not a
      directory scan — every other `registry:lib`/`registry:ui` item
      already has its own arm there) so `adico add`/`--replace` can
      actually fetch it. Verify with a unit test asserting each variant's
      `class()` output and that `Default::default()` is `Radius::Default`,
      and by installing `variants` into a scratch consumer via the real
      `adico add` CLI path (not a direct file copy) to confirm the new
      reader arm and checksum are both correct.
- [ ] 1.2 Add `variants` to the `registryDependencies` of every item
      identified in task 3.1's target list (do not add it speculatively to
      items that don't get a `radius` prop). Verify
      `cargo run -p adico-xtask -- registry build` and `registry validate`
      pass.

**Checksum discipline for every remaining task in this change:** any task
below that edits a `registry/ui/*.rs` or `registry/lib/*.rs` file's
content must also recompute and update that file's `checksum` in
`registry/registry.json` (`shasum -a 256 <path>`) in the same task —
`registry build`/`registry validate` do not do this for you, and a stale
checksum only surfaces later, at install time.

## 2. Convention normalization

- [x] 2.1 **`disabled` representation.** Categorize every styled
      component as native-leaf (`Option<bool>`) or primitive-backed
      (`ReadSignal<bool>`); fix every component whose current
      representation doesn't match its category, documenting `Button`'s
      native-only routing as the one intentional exception in its own doc
      comment. Verify by grepping every `disabled` field's type across
      `registry/ui/*.rs` and confirming each matches its category or is
      `Button` with its exception comment present.
- [x] 2.2 **`class: Option<String>` everywhere.** Add to `Slider`,
      `RangeSlider`, `Toast` (fix in
      `packages/adico-primitives/src/{slider,toast}.rs`, since their
      registry facades re-export the primitive's props type verbatim —
      keep the facade a bare re-export, do not introduce a
      registry-owned wrapper struct), and `AreaThumb`/`HueSlider` (fix in
      `registry/ui/color_picker.rs`, since these two take no props struct
      at all today). **Correction found during implementation:**
      `styling-usage check` does not actually assert a `class` field
      exists on every styled item (verified by reading
      `packages/adico-xtask/src/styling_usage.rs` — its conditions (a)–(f)
      cover Tailwind-only/token-compliance, not prop-surface shape); verify
      instead by grepping each of the four files for a real `class:
      Option<String>` field/parameter reaching the rendered root element,
      plus a compile check that
      existing `with_class()`-style workarounds in `slider.rs` can be
      simplified now that a real `class` field exists (simplify them in
      this task, don't leave the workaround dangling unused).
- [x] 2.3 **`attributes: Vec<Attribute>` extends coverage.** Using
      `statics/prop_parity/*.json`'s `dioxus-components`/
      `dioxus-primitives` axes (the ones that enumerate native events
      individually) as the evidence source, identify every styled
      component that renders a real element but has no `#[props(extends =
      GlobalAttributes)]` `attributes` field, and add one.
      **Correction found during implementation:** Change A's earlier
      `looks_like_native_event_name`/`has_attributes_extend` fix already
      closed nearly all of this gap — the actual evidence-driven residue
      was exactly 3 items, not the large remainder design.md's Context
      implied: `toggle` (missing `onmounted`/`onfocus`/`onkeydown` — the
      registry facade wasn't forwarding these three individually-named
      callback fields the primitive already exposes, a documented
      Dioxus-issue workaround distinct from the generic `attributes`
      mechanism — plus a plain missing `attributes` field), `sidebar`'s
      `SidebarTrigger` (missing `attributes`; added after the trigger's
      own fixed `onclick` since Dioxus requires an attribute spread to be
      an element's last attribute, so a caller's own `onclick` replaces
      rather than composes with the toggle, matching every other
      `attributes`-accepting component's own precedent), and
      `pagination`'s `PaginationPrevious`/`PaginationNext` (missing
      `attributes`, added and forwarded to the underlying
      `PaginationLink`, which already had it). Verified
      `cargo run -p adico-xtask -- prop-parity sync` then reports zero
      native-event-shaped `missing` entries for the fixed items (per
      `prop_parity.rs`'s `looks_like_native_event_name`/
      `has_attributes_extend` coverage rule).
- [x] 2.4 **Restore dropped props.** `Checkbox`: restore the primitive's
      form `value` and `attributes` fields the registry facade currently
      narrows away. `ToastProvider`: un-narrow `default_duration`/
      `max_toasts` back to `ReadSignal<...>`, matching the primitive's own
      type. Verify with a unit test per component asserting the facade's
      prop type matches the primitive's, and update
      `docs/adico/component-hardening-audit.md` to record both as closed
      (matching its existing `Menubar`-dropped-`disabled` entry's format).
- [ ] 2.5 **Handler naming.** Rename `AlertDialogAction`/
      `AlertDialogCancel`'s and `ToolbarButton`'s `on_click` to a
      component-appropriate semantic name (e.g. `on_select`), matching the
      primitive-facing convention every other primitive-backed component
      in the registry already uses. Verify by grepping for `on_click` in
      `registry/ui/*.rs` and confirming no remaining primitive-backed
      component uses it (a native-leaf component's plain `onclick`
      passthrough via `attributes` is unaffected — this only renames
      hand-declared `on_click` fields).
- [ ] 2.6 **`**BREAKING**` controlled-trio.** Migrate `Select`'s,
      `Combobox`'s, and `TagGroup`'s singular-select function `value`
      parameter from `Option<ReadSignal<Option<T>>>` to
      `ReadSignal<Option<T>>`, matching their own `SelectMulti`/
      `ComboboxMulti`/`TagGroupMulti` functions' already-compliant
      `values` shape. If a specific case needs the double-`Option` for a
      real reason found during implementation, keep it and record
      `intentional_difference` with that reason instead of forcing the
      migration. Verify `cargo check --locked --workspace`, update every
      call site in `examples/*`/`tests/installation/*` that constructs a
      `Select`/`Combobox`/`TagGroup` value signal, and re-run their
      Playwright suites if any exist for these components.

## 3. `radius` rollout

- [ ] 3.1 Using `grep -lE "rounded-(none|sm|md|lg|xl|2xl|3xl|full)\b"
      registry/ui/*.rs` (50 files as of this proposal) as the starting
      candidate list, produce the final target list of components with a
      genuine visible bounded surface (excluding any candidate whose
      `rounded-*` usage is on a sub-element that isn't the component's own
      semantic surface, with a recorded reason for each exclusion). Add
      `radius: Radius` (`#[props(default)]`) to each.
- [ ] 3.2 Strip every `rounded-*` literal from each target component's
      base class string(s) **and** every `impl FooVariant/FooSize { fn
      class(self) }` match arm that currently hard-codes one (confirmed
      present in at least `button.rs`'s `ButtonSize::class()` — check all
      14 files with this inherent-impl pattern, not just `Button`).
      Compose `radius.class()` into the final `cn()` call instead. Verify
      per component that its Tailwind classes still produce the same
      visual rounding at the `Default` variant (no visual regression at
      the default).
- [ ] 3.2b Add one `(item, part, "radius")` entry per task 3.1 target item
      to `packages/adico-xtask/src/prop_parity.rs`'s
      `ADICO_EXTENSION_REASONS` table (design.md's D3 addendum), naming
      the part `radius` actually lands on for that item, with reason
      "adico extension: consistent corner-radius control not present
      upstream" (or the component-appropriate equivalent). Verify `cargo
      run -p adico-xtask -- prop-parity sync` classifies `radius` as
      `adico_extension` with that reason on every target item, and
      `prop-parity check` passes.
- [ ] 3.3 Add condition (g) to `packages/adico-xtask/src/styling_usage.rs`:
      for every item in the task 3.1 target list, assert no `rounded-`
      literal appears anywhere in its `registry/ui/*.rs` file outside
      `Radius::class()`'s own match arms. Verify the new condition fails
      against a deliberately reintroduced `rounded-md` literal in a test
      fixture, then passes against the real, fixed source.

## 4. `loading` rollout

- [ ] 4.1 Add `loading: bool` (`#[props(default)]`) and `loading_text:
      Option<String>` to `Button`, `InputGroupButton`, `PaginationLink`,
      `AlertDialogAction`, `ToolbarButton`, `SidebarMenuButton`, and
      `DataTable`. Each composes the existing `Spinner` registry item
      (declared in `registryDependencies`, not duplicated) when `loading`
      is true, sets `aria-busy="true"` and native `disabled`, and renders
      `loading_text` in place of/alongside the normal content per that
      component's own composition shape.
- [ ] 4.2 Add each of the seven `(item, part, prop)` entries to
      `packages/adico-xtask/src/prop_parity.rs`'s `ADICO_EXTENSION_REASONS`
      table with the reason "adico extension: shadcn composes `<Button
      disabled><Spinner /></Button>` by hand; adico exposes it as a first-class
      prop instead" (or the component-appropriate equivalent). Verify
      `cargo run -p adico-xtask -- prop-parity sync` classifies `loading`/
      `loading_text` as `adico_extension` with that reason on all seven
      items, and `prop-parity check` passes.

## 5. Parity-prop waves

Each wave: for every item listed, resolve every `missing` entry
`statics/prop_parity/<item>.json` currently records against its
best-matching axis — implement it, or record it `intentional_difference`
with a written reason (design.md's D6). Consult the generated JSON file
directly for the exact prop list; it is not duplicated here.

- [ ] 5.1 **Wave 1 — composite/collection controls** (96 missing entries):
      `combobox` (29), `slider` (26), `select` (16), `calendar` (14),
      `navigation-menu` (11). Verify `cargo run -p adico-xtask --
      prop-parity diff` reports no drift for these five items against
      their updated source, and that re-running `prop-parity sync`
      produces records with zero remaining `missing` status for them.
- [ ] 5.2 **Wave 2 — menu/overlay family** (34 missing entries):
      `dropdown-menu` (5), `hover-card` (5), `popover` (6), `tooltip` (6),
      `toggle-group` (6), `context-menu` (3), `menubar` (3). Verify same
      as 5.1 for these seven items.
- [ ] 5.3 **Wave 3 — native leaf/form controls** (32 missing entries):
      `checkbox` (7), `switch` (6), `progress` (6), `input` (2),
      `textarea` (2), `toolbar` (2), `avatar` (2), `toggle` (1),
      `accordion` (1), `button` (1), `label` (1), `radio-group` (1).
      Verify same as 5.1 for these twelve items.
- [ ] 5.4 **Wave 4 — dialog/overlay content** (18 missing entries):
      `sheet` (6), `dialog` (3), `alert-dialog` (3), `drawer` (3),
      `command` (3). Verify same as 5.1 for these five items.
- [ ] 5.5 **Wave 5 — Dioxus-only/composite misc** (48 missing entries):
      `pagination` (12), `tag-group` (8), `color-picker` (4),
      `drag-and-drop-list` (4), `tabs` (4), `attachment` (3), `item` (2),
      `sidebar` (2), `bubble` (1), `input-otp` (1), `marker` (1), `toast`
      (6). Verify same as 5.1 for these twelve items.

**Per-wave fixture refresh:** end each of sections 1–5 by refreshing
`apps/playground`'s installed copy via `adico add --replace` (real CLI
path, never a direct file edit) for every item touched in that section,
then `cargo check --locked --workspace`. Waiting until task 6.2 to refresh
the playground copy means every intermediate check in sections 1–5
validates a stale installed copy — a compile break would be attributed to
the wrong section. `examples/*`/`tests/installation/*` still refresh once,
at task 6.2, since they aren't touched incrementally during a wave.

## 6. Propagate and validate

- [ ] 6.1 Re-run `cargo run -p adico-xtask -- registry build`,
      `primitive-usage sync`, `styling-usage sync`, `component-compat
      sync`, and `prop-parity sync` in that order, and commit the
      regenerated `statics/**`/`registry/generated/**` output. Verify
      every corresponding `check` command passes with zero drift.
- [ ] 6.2 Refresh every installed consumer fixture
      (`examples/basic-spa`, `examples/basic-ssr`,
      `tests/installation/*`) through `adico add --replace`, never by
      hand-editing copied source. Verify each fixture's `cargo check`/
      `cargo test` passes, and `cargo check --target wasm32-unknown-unknown`
      for the web-targeting fixtures.
- [ ] 6.3 Run `cd tests/playwright && npm test` for keyboard/axe coverage
      on every component whose interactive props changed in Waves 1–5 or
      the `**BREAKING**` controlled-trio migration (task 2.6). Verify all
      pass; report any surface with no existing fixture (per
      `docs/validation.md`'s recorded desktop/mobile gap) rather than
      claiming it passed.
- [ ] 6.4 Run the full baseline: `cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings` (the documented baseline;
      also run the wider `--workspace` form and report any failure
      outside this change's own files separately, per the precedent set
      in `extend-upstream-prop-evidence`), `cargo test --locked -p
      adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask`, and `openspec validate
      complete-component-prop-surface --strict`. Verify all pass; report
      any that don't and why.
