## 1. Extract the router into `routes.rs`

- [ ] 1.1 Create `apps/playground/src/routes.rs`; move the `Route` enum,
  `nav_items()`, and the `Layout` component out of `main.rs` verbatim
  (adjust only the imports each needs, e.g. `crate::pages::Home` for
  `Route::Home {}` and the `Route::*Page {}` targets, `crate::theme::
  {ThemeLauncher, ThemeModal, ThemeSelection}`, asset consts `Layout`
  references); verify `routes.rs` compiles on its own module boundary (no
  leftover reference to something still in `main.rs` that isn't
  imported).
- [ ] 1.2 Update `main.rs` to declare `mod routes;` and use
  `routes::{Route, Layout}` in `App`'s `Router::<Route> {}` call; remove
  the now-moved `Route`, `nav_items()`, and `Layout` definitions from
  `main.rs`; verify `main.rs` retains only asset consts, `App`, and the
  `adico:start`/`adico:end` block.

## 2. Move `demo.rs`/`controls.rs` under `components/`

- [ ] 2.1 Move `apps/playground/src/demo.rs` and
  `apps/playground/src/controls.rs` to
  `apps/playground/src/components/demo.rs` and
  `apps/playground/src/components/controls.rs` unchanged; add
  `pub mod demo;` and `pub mod controls;` to
  `apps/playground/src/components/mod.rs` **outside** the
  `// adico:start`/`// adico:end` managed block; verify
  `cargo check -p adico-playground` succeeds with the new paths.
- [ ] 2.2 Run `adico add <any already-installed item> --replace` (or
  `adico css build`) from `apps/playground` and confirm the two new
  `pub mod demo;`/`pub mod controls;` lines in `components/mod.rs`
  survive untouched, proving the managed-region writer preserves
  hand-authored content outside its markers in this file the same way it
  already does in `main.rs`/`adico_lib/mod.rs`; if the lines are lost or
  the file is fully overwritten, stop and re-plan rather than proceeding
  on an unverified assumption (see design.md Risks).
- [ ] 2.3 Do **not** move `theme.rs` — leave it exactly where and as it is
  (see design.md Decision 5); `main.rs`/`routes.rs`'s `use crate::theme::
  {...}` stays unchanged in this change.

## 3. Split `pages.rs` into `pages/`

- [ ] 3.1 Create `apps/playground/src/pages/index.rs` (TanStack Start's
  convention for a directory's index route, used here for `/`) with the
  `Home` component moved from `main.rs` unchanged; verify its rendered
  output (`dioxus::ssr::render` of a fresh `VirtualDom`) is byte-identical
  to the pre-move `Home`'s output.
- [ ] 3.2 Create one file per remaining page under `apps/playground/src/pages/`
  (`button.rs`, `badge.rs`, `card.rs`, `input.rs`, `textarea.rs`,
  `skeleton.rs`, `item.rs`, `pagination.rs`, `dialog.rs`, `sheet.rs`,
  `select.rs`, `combobox.rs`, `tooltip.rs`, `popover.rs`, `hover_card.rs`,
  `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `calendar.rs`,
  `date_picker.rs`, `sidebar.rs`), each containing exactly the
  corresponding `#[component] fn XPage()` block (plus any private
  helper type used only by that page, e.g. `ButtonContent` moves into
  `pages/button.rs` alongside `ButtonPage`) moved verbatim from the old
  `pages.rs`, with `use crate::demo::Demo`/`use crate::controls::{...}`
  updated to `use crate::components::demo::Demo`/
  `use crate::components::controls::{...}`; verify each new file's
  component body diffs identically (whitespace/import-order aside)
  against the corresponding block in the pre-split `pages.rs`.
- [ ] 3.3 Move the two `#[cfg(all(test, feature = "server"))]`
  `DatePickerPage` SSR-render tests from the end of the old `pages.rs`
  into `pages/date_picker.rs`'s own `#[cfg(test)]` module; verify
  `cargo test -p adico-playground --features server` runs and passes both.
- [ ] 3.4 Create `apps/playground/src/pages/mod.rs` declaring `pub mod`
  for all 22 files (`index` plus the 21 above) and re-exporting each page
  component (matching the old `use pages::{...}` import list in
  `main.rs`); delete `apps/playground/src/pages.rs`; verify no remaining
  reference to a `pages.rs` path anywhere in the crate.
- [ ] 3.5 Update `main.rs` and `routes.rs` imports to `mod pages;` /
  `use crate::pages::{...}` as needed instead of the old flat `pages`
  module import; verify `cargo check -p adico-playground` succeeds.

## 4. Verify the refactor preserves behavior exactly

- [ ] 4.1 Run `cargo check --workspace`, `cargo fmt --all --check`, and
  `cargo test -p adico-playground` (default features) and
  `cargo test -p adico-playground --features server` (the moved
  DatePicker tests); verify all pass with zero warnings introduced.
- [ ] 4.2 Run a live `dx serve --platform web` instance of
  `apps/playground` and navigate to `/` and all 21 page routes; verify
  every page renders with no console errors and the sidebar nav lists
  the same 21 links in the same order as before the refactor.
- [ ] 4.3 Run `openspec validate refactor-playground-structure --strict`;
  verify it reports no errors.

## 5. Record follow-up scope

- [ ] 5.1 Confirm no task in this change added a page or route for any of
  the 24 components installed without a page (`accordion`, `alert-dialog`,
  `aspect-ratio`, `avatar`, `checkbox`, `collapsible`, `color-picker`,
  `drag-and-drop-list`, `label`, `mode-toggle`, `progress`, `radio-group`,
  `scroll-area`, `slider`, `switch`, `tabs`, `tag-group`, `theme-switcher`,
  `toast`, `toggle`, `toggle-group`, `toolbar`, `virtual-list`); leave this
  named explicitly as follow-up scope for a future change rather than
  silently expanding this one.
- [ ] 5.2 Confirm `theme.rs` was left untouched by this change (see
  design.md Decision 5) — it is replaced outright, not migrated, by the
  follow-up change that adds a registry `ThemeBuilder` component and
  rewires `apps/playground` onto `ModeToggle`/`ThemeSwitcher`/
  `ThemeBuilder`/`Sidebar`.
