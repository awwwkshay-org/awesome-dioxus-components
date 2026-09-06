## 1. Generic persisted-global primitive

- [x] 1.1 Add `packages/adico-primitives/src/persisted_state.rs` with
      `use_persisted_global<T: Copy + PartialEq + 'static>(global, storage_key,
      to_token, from_token) -> (Memo<T>, Callback<T>)`, target-gated
      `web`/`native`/neither load+persist bodies adapted from `theme_mode.rs`'s
      proven implementation, and register `pub mod persisted_state;` in
      `lib.rs`. Verify: `cargo check --locked -p adico-primitives` passes
      under `--features web`, `--features native`, and default (no features).

      **Done.** Module created; registered in `lib.rs`'s shared-machinery
      group (alongside `pointer`/`portal`). All three feature configurations
      compile clean (`cargo check` ×3), and `cargo clippy --all-targets -D
      warnings` is clean under all three too.

- [x] 1.2 Add unit tests for the pure logic (token round-trip via a
      throwaway test enum, preferences-file-name/contents round-trip,
      storage-key validation) and verify `cargo test -p adico-primitives
      persisted_state` passes.

      **Done.** 7 tests added, all passing:
      `a_token_pair_coerces_to_the_hooks_fn_pointer_parameters`,
      `from_token_rejects_an_unrecognized_or_empty_token`,
      `every_token_is_distinct`,
      `preferences_file_name_reproduces_the_shipped_theme_mode_file`,
      `preferences_contents_round_trip`,
      `parse_preferences_contents_rejects_garbage`,
      `storage_key_validation_rejects_path_and_whitespace_characters`.

## 2. Refactor ThemeMode onto the new primitive

- [x] 2.1 Rewrite `theme_mode.rs`'s `use_persisted_theme_mode` to delegate
      to `persisted_state::use_persisted_global`, deleting the now-dead
      `load_persisted_mode`/`persist_mode`/`desktop_preferences_path`/
      `read_desktop_preferences`/`write_desktop_preferences` and un-gating
      `mode_token`/`mode_from_token` (now used unconditionally as fn
      pointers). Verify: public signature unchanged, `cargo test -p
      adico-primitives theme_mode` still passes with the same 5
      pre-existing tests plus one new one.

      **Done.** `mode_tokens_round_trip_and_are_distinct` added (now
      possible to write unconditionally since the token fns are no longer
      `cfg`-gated). All 6 `theme_mode` tests pass.

## 3. Persist ThemeSwitcher's palette

- [x] 3.1 Delete `theme_switcher.rs`'s DOM-read-back hydration mechanism
      (`ALL_PROPERTY_NAMES`, `read_value`, `matching_combined_palette`,
      `ThemePalette::matching_primary`/`matching_surface`, the `hydrated`
      signal and its gated `use_effect`+`spawn`, the `read_root_properties`
      import) and its 6 associated tests. Verify: `grep -c matching_combined
      registry/ui/theme_switcher.rs` returns 0.

      **Done.**

- [x] 3.2 Add `PALETTE_STORAGE_KEY`, `ThemePalette::token()`/`from_token()`
      (lowercase tokens, matching `ThemeMode`'s convention), and rewire
      `ThemeSwitcher`'s component body onto
      `use_persisted_global(&PALETTE, PALETTE_STORAGE_KEY, ThemePalette::token,
      ThemePalette::from_token)`. Verify: `grep -n 'PALETTE.write()'
      registry/ui/theme_switcher.rs` matches only the doc-comment warning
      example, no real code line.

      **Done.** Confirmed via the grep above.

- [x] 3.3 Add tests: `palette_tokens_round_trip_every_preset`,
      `palette_tokens_are_distinct_lowercase_ascii`,
      `from_token_rejects_an_unrecognized_token`,
      `the_three_role_groups_apply_fourteen_distinct_properties`. Verify:
      `cargo test -p adico-playground theme_switcher` passes (8 tests).

      **Done.** All 19 `theme_builder`/`theme_switcher` tests pass together
      in `adico-playground` (11 + 8).

- [x] 3.4 Rewrite the module doc comment's hydration narrative to describe
      the persisted-store design and explicitly name the accepted
      sibling-overwrite tradeoff (see design.md's Risks section). Verify:
      re-read the doc comment, confirm no reference to the deleted
      `hydrated`/DOM-read-back mechanism remains.

      **Done.**

## 4. Mechanical propagation

- [x] 4.1 `rustfmt --edition 2024 registry/ui/theme_switcher.rs`, `cargo fmt
      --all`, recompute its SHA-256, update `registry/registry.json`'s
      `theme-switcher.files[0].checksum` and `documentation.compositionNote`.
      Verify: `cargo run -p adico-xtask -- registry build && registry
      validate` both pass.

      **Done.** Checksum `2f3648304be6f9f054bdbb87def3b11b4b11c3e73d1ff47dc59d8e6f3ae90e01`.

- [x] 4.2 Copy `registry/ui/theme_switcher.rs` verbatim to the three
      maintained consumers (`apps/playground`, `examples/basic-spa`,
      `examples/basic-ssr`). Verify: all three `shasum -a 256` match the
      registry source.

      **Done.**

- [x] 4.3 `cargo run -p adico-xtask -- primitive-usage sync` (the new
      `use adico_primitives::persisted_state::...` import changes
      `theme-switcher`'s recorded `primitiveModules`). Verify: `git diff
      --stat statics/primitive_usage/` shows only `theme-switcher.json`
      changed, and `primitive-usage check` passes afterward.

      **Done.** `primitiveModules` is now `["persisted_state", "theme_mode"]`.

- [x] 4.4 Run and confirm passing, with no `sync` needed: `styling-usage
      check`, `component-props check`, `playground-controls check`,
      `prop-parity check`, `provenance check`.

      **Done.** All pass. (`component-compat check` and `primitive-compat
      check` report pre-existing staleness unrelated to this change,
      confirmed present before this change's first commit too -- not
      addressed here.)

- [x] 4.5 Full validation pass: `cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings`, `cargo test --locked` for
      those same 5 packages, `cargo check --target wasm32-unknown-unknown -p
      adico-primitives --features web`. Verify: all commands exit 0.

      **Done.** All six commands passed with exit code 0.

## 5. Live verification (no unit test reaches localStorage/reload behavior)

- [x] 5.1 In `dx serve` (from `apps/playground`): select a non-default
      preset, hard-reload, confirm the Select still shows it, `--primary`
      matches that preset, and `localStorage.getItem('adico-theme-palette')`
      matches its token.

      **Done.** Selected Rose, hard-reloaded: Select still showed "Rose" in
      both mounted instances, `localStorage.getItem('adico-theme-palette')
      === "rose"`, `--primary` matched Rose's value for the active
      appearance.

- [x] 5.2 With that preset selected, toggle dark/light and confirm
      `--primary` recomputes to that preset's *other*-appearance value (not
      stuck light, not reverted to Slate) — the exact regression shape
      `apply_resolved_class`'s own doc comment warns a silent, compiles-fine
      effect-ordering bug can cause.

      **Done.** Dark → Light with Rose selected: `--primary` went from
      `349.7 89.2% 60.2%` (Rose dark) to `346.8 77.2% 49.8%` (Rose light) --
      correctly recomputed, not stuck, not reverted to Slate.

- [x] 5.3 Confirm `adico-theme-mode` and `adico-theme-palette` are
      independent `localStorage` keys — changing one doesn't clear the
      other.

      **Done.** Both keys present simultaneously (`"light"` /`"rose"`)
      after the mode toggle in 5.2; changing mode didn't clear the palette
      key or vice versa.

- [x] 5.4 Confirm multi-instance sync still holds: the playground's
      persistent sidebar `ThemeSwitcher` and the `/theme-switcher` demo-page
      instance both update immediately when either changes.

      **Done.** Both instances showed "Rose" simultaneously throughout,
      with no reload needed between selecting it and seeing both update.

- [x] 5.5 Confirm the cross-consistency direction that must not regress:
      pick a non-Slate preset via `ThemeSwitcher`, open `theme-builder`
      ("Customize theme") — its Primary/Secondary/Accent swatches open
      already highlighting that preset.

      **Done.** With Rose selected via `ThemeSwitcher`, opening
      `theme-builder` showed "Primary · Rose", "Secondary · Rose", "Accent ·
      Rose" and Appearance "Light" -- all correctly reflecting the
      persisted selection.

- [x] 5.6 Confirm the accepted tradeoff from design.md is scoped as
      expected: apply per-token edits in `theme-builder`, then mount/remount
      a `ThemeSwitcher` — the primary/secondary/accent subset gets
      overwritten (expected), nothing else breaks, and 5.5's direction still
      works afterward.

      **Done.** Changed Primary to Violet in `theme-builder` (confirmed
      `--primary` became Violet's `262.1 83.3% 57.8%`); navigating away and
      back to `/theme-switcher` (remounting its demo-page instance) reapplied
      the persisted Rose preset, overwriting the Violet edit back to `346.8
      77.2% 49.8%` -- exactly the accepted, documented tradeoff, and 5.5's
      direction (re-opening `theme-builder`) still showed the correct,
      consistent Rose state afterward.

## 6. Explicit non-goal (report, don't silently skip)

- [x] 6.1 Do not refresh `tests/installation/theme-consumer`'s fixture or
      `tests/playwright/theme-switcher.spec.ts` — both are already stale
      relative to registry as of the prior three-radio-picker-to-Select
      redesign (a separate, earlier commit this change does not touch), and
      are consistent with each other. Refreshing them is real, separate
      follow-up work with its own Playwright verification.

      **Done (recorded, not run).** `cd tests/playwright && npm test` was
      not run for this change.
