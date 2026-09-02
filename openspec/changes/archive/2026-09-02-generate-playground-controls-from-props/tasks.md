## 1. Enum introspection in adico-xtask

- [x] 1.1 Add an `Item::Enum` arm to `walk_items` in
      `packages/adico-xtask/src/rust_introspect.rs`, extracting each public
      enum's name, its variants in declaration order, and which variant
      carries `#[default]`. Doc comments are not extracted (see design.md -
      labels are derived from the identifier, not doc comments). Verify with
      a new unit test alongside the existing
      `extracts_declared_and_bare_defaults` test, covering an enum with a
      `#[default]` variant and one without.
- [x] 1.2 Extend `FileIntrospection`'s public surface (new field, e.g.
      `enums: BTreeMap<String, EnumIntrospection>`) without changing
      existing `props`/`components`/`hooks_*` behavior. Verify
      `cargo test -p adico-xtask` still passes, including
      `primitive_compat.rs`/`component_compat.rs`'s existing tests that
      depend on `FileIntrospection`.

## 2. `playground-controls` xtask command

- [x] 2.1 Add `packages/adico-xtask/src/playground_controls.rs` implementing
      the prop-shape allowlist from design.md (`bool`/`Option<bool>` ->
      `BoolControl`, `String` -> `TextControl`, enum-with-`#[default]` ->
      generated option constant; everything else skipped with a recorded
      reason), plus the identifier-to-label humanizer (`IconXs` ->
      `Icon Xs`). Verify with unit tests against fixture source (a struct
      with one of each supported shape, plus one of each skipped shape)
      asserting the correct classification and reason text, and unit tests
      for the humanizer covering a single word, a two-word PascalCase
      identifier, and an identifier with a multi-letter acronym-like run
      (e.g. `Xs`).
- [x] 2.2 Implement `sync`: for every `apps/playground/src/components/ui/*.rs`
      file with at least one enum-typed prop, write
      `apps/playground/src/generated/controls/<item>.rs` containing a `pub
      const <ENUM>_OPTIONS: &[(&str, <Enum>)]` per enum plus its
      exhaustiveness guard (design.md's `const _: () = { fn _exhaustive... }`
      pattern), and a generated `mod.rs` aggregating them. Verify by running
      `cargo run -p adico-xtask -- playground-controls sync` and inspecting
      that `apps/playground/src/generated/controls/button.rs` contains both
      `BUTTON_VARIANT_OPTIONS` and `BUTTON_SIZE_OPTIONS` with all declared
      variants.
- [x] 2.3 Implement `check`: regenerate in memory and diff against committed
      output without writing, exit non-zero on any difference, run fully
      offline. Verify by hand-editing a generated file and confirming `cargo
      run -p adico-xtask -- playground-controls check` fails and identifies
      the file; then reverting and confirming it passes.
- [x] 2.4 Implement `diff`: print the would-be change without writing.
      Verify by editing a source enum's doc comment and confirming `diff`
      shows the label change.
- [x] 2.5 Wire the subcommand into `packages/adico-xtask/src/main.rs`
      alongside `primitive-usage`/`styling-usage`. Verify
      `cargo run -p adico-xtask -- playground-controls sync --help` (or
      equivalent) lists it.

## 3. Generate and commit output

- [x] 3.1 Run `cargo run -p adico-xtask -- playground-controls sync` once
      against the current tree and commit
      `apps/playground/src/generated/controls/`. Verify
      `cargo run -p adico-xtask -- playground-controls check` then passes
      with a clean diff.
- [x] 3.2 Add `apps/playground/src/generated/controls/mod.rs` (or the
      existing `main.rs`/`lib.rs`) wiring so the new module is part of the
      compiled crate. Verify `cargo check --locked --workspace` succeeds.
- [x] 3.3 Confirm the exhaustiveness guard actually fails the build on
      drift: locally add a throwaway variant to `ButtonVariant` in
      `apps/playground/src/components/ui/button.rs` without regenerating,
      run `cargo check --locked --workspace`, verify it fails, then revert
      the throwaway change (do not commit it).

## 4. Wire pages to consume generated constants

Confirmed target set (from running `playground-controls sync` against the
current tree): 15 components got a generated file — `alert`, `avatar`,
`badge`, `button`, `button_group`, `empty`, `input_group`, `item`,
`native_select`, `sidebar`, `skeleton`, `switch`, `tabs`, `toggle`,
`toggle_group`. **CI-driving discovery**: an unused generated constant is
not just a lint nit here — CI runs with `RUSTFLAGS: -D warnings` and
`cargo clippy --locked --workspace --all-targets -- -D warnings` (the
whole workspace, not the narrower package list in this repo's local
baseline commands), so every generated constant must have a real consumer
or the build fails. Of the 15, 8 already have a page control to migrate
(`alert`, `badge`, `button`, `empty`, `item`, `sidebar`, `skeleton`,
`tabs`); 7 have no existing control at all (`avatar`, `button_group`,
`input_group`, `native_select`, `switch`, `toggle`, `toggle_group`) and
need one newly wired, per user confirmation — this expands the change's
page-touching scope beyond the original 28-page estimate, deliberately: it
resolves the CI failure honestly (real usage, not suppression) and gives
these 7 pages live variant/size switching for the first time.

- [x] 4.1 For `pages/badge.rs`: replace the hand-typed `BadgeVariant` option
      `vec![...]` with `generated::controls::badge::BADGE_VARIANT_OPTIONS`.
      Verify by running the playground and confirming the Badge page's
      Variant control now offers all 7 variants, including `Ghost` and
      `Link`.
- [x] 4.2 For `pages/item.rs`: replace the hand-typed `ItemVariant` option
      list the same way. Verify the Item page's Variant control offers all
      4 variants, including `Outline`.
- [x] 4.3 For `pages/button.rs` and `pages/sidebar.rs`: replace their
      hand-typed option lists (`ButtonVariant`, `ButtonSize`,
      `SidebarCollapsible`, `SidebarSide`, `SidebarVariant`) with the
      generated constants. Verify each control still renders and behaves
      identically (same variant set, same default), and that `button.rs`'s
      line count drops roughly in line with the removed option-list
      boilerplate.
- [x] 4.4 For `pages/alert.rs`, `pages/empty.rs`, `pages/skeleton.rs`, and
      `pages/tabs.rs`: replace each page's hand-typed option list for its
      generated enum with the matching generated constant. Verify each
      page's control still renders and its option set matches the
      generated constant exactly.
- [x] 4.5 For `pages/avatar.rs`, `pages/button_group.rs`,
      `pages/input_group.rs`, `pages/native_select.rs`, `pages/switch.rs`,
      `pages/toggle.rs`, and `pages/toggle_group.rs` (none of which have an
      existing `controls:` block): add a `use_signal` and a `SelectControl`
      sourced from that component's generated constant, wired into the
      component's call site, following the same pattern as `pages/badge.rs`.
      Verify each page now renders a working control that changes the live
      component, and that the generated constant is no longer reported
      unused by `cargo check`/`cargo clippy`.
- [x] 4.6 Confirm no generated constant is unused: run `cargo clippy
      --locked --workspace --all-targets -- -D warnings` (matching CI's
      invocation, not just this repo's narrower local package list) and
      verify it reports no `never used` warnings under
      `apps/playground/src/generated/`.

## 5. Documentation and validation wiring

- [x] 5.1 Document the new `playground-controls sync|check|diff` command in
      `docs/development.md`'s xtask command list and add
      `playground-controls check` to `docs/validation.md`'s validation
      matrix, alongside `primitive-usage check`/`styling-usage check`.
      Verify the documented command text matches the actual CLI output.
- [x] 5.2 Run the full baseline validation from `CLAUDE.md`: `cargo fmt
      --all --check`, `cargo check --locked --workspace`, `cargo clippy
      --locked -p adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask --all-targets -- -D warnings`, `cargo
      test --locked -p adico-cli -p adico-primitives -p
      adico-registry-core -p adico-test-utils -p adico-xtask`, `openspec
      validate generate-playground-controls-from-props --strict`, plus the
      CI-matching `cargo clippy --locked --workspace --all-targets -- -D
      warnings` (task 4.6 already covers this but confirm again after all
      of section 4 lands). Verify all commands succeed and report any that
      could not be run (e.g. wasm32 target checks are out of scope for this
      change since no wasm-only code path changes).
