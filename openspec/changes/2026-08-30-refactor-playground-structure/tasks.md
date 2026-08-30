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

## 2. Split `pages.rs` into `pages/`

- [ ] 2.1 Create `apps/playground/src/pages/index.rs` (TanStack Start's
  convention for a directory's index route, used here for `/`) with the
  `Home` component moved from `main.rs` unchanged; verify its rendered
  output (`dioxus::ssr::render` of a fresh `VirtualDom`) is byte-identical
  to the pre-move `Home`'s output.
- [ ] 2.2 Create one file per remaining page under `apps/playground/src/pages/`
  (`button.rs`, `badge.rs`, `card.rs`, `input.rs`, `textarea.rs`,
  `skeleton.rs`, `item.rs`, `pagination.rs`, `dialog.rs`, `sheet.rs`,
  `select.rs`, `combobox.rs`, `tooltip.rs`, `popover.rs`, `hover_card.rs`,
  `dropdown_menu.rs`, `context_menu.rs`, `menubar.rs`, `calendar.rs`,
  `date_picker.rs`, `sidebar.rs`), each containing exactly the
  corresponding `#[component] fn XPage()` block (plus any private
  helper type used only by that page, e.g. `ButtonContent` moves into
  `pages/button.rs` alongside `ButtonPage`) moved verbatim from the old
  `pages.rs`; verify each new file's component body diffs identically
  (whitespace/import-order aside) against the corresponding block in the
  pre-split `pages.rs`.
- [ ] 2.3 Move the two `#[cfg(all(test, feature = "server"))]`
  `DatePickerPage` SSR-render tests from the end of the old `pages.rs`
  into `pages/date_picker.rs`'s own `#[cfg(test)]` module; verify
  `cargo test -p adico-playground --features server` runs and passes both.
- [ ] 2.4 Create `apps/playground/src/pages/mod.rs` declaring `pub mod`
  for all 22 files (`index` plus the 21 above) and re-exporting each page
  component (matching the old `use pages::{...}` import list in
  `main.rs`); delete `apps/playground/src/pages.rs`; verify no remaining
  reference to a `pages.rs` path anywhere in the crate.
- [ ] 2.5 Update `main.rs` and `routes.rs` imports to `mod pages;` /
  `use crate::pages::{...}` as needed instead of the old flat `pages`
  module import; verify `cargo check -p adico-playground` succeeds.

## 3. Verify the refactor preserves behavior exactly

- [ ] 3.1 Run `cargo check --workspace`, `cargo fmt --all --check`, and
  `cargo test -p adico-playground` (default features) and
  `cargo test -p adico-playground --features server` (the moved
  DatePicker tests); verify all pass with zero warnings introduced.
- [ ] 3.2 Run a live `dx serve --platform web` instance of
  `apps/playground` and navigate to `/` and all 21 page routes; verify
  every page renders with no console errors and the sidebar nav lists
  the same 21 links in the same order as before the refactor.
- [ ] 3.3 Run `openspec validate refactor-playground-structure --strict`;
  verify it reports no errors.

## 4. Record follow-up scope

- [ ] 4.1 Confirm no task in this change added a page or route for any of
  the 24 components installed without a page (`accordion`, `alert-dialog`,
  `aspect-ratio`, `avatar`, `checkbox`, `collapsible`, `color-picker`,
  `drag-and-drop-list`, `label`, `mode-toggle`, `progress`, `radio-group`,
  `scroll-area`, `slider`, `switch`, `tabs`, `tag-group`, `theme-switcher`,
  `toast`, `toggle`, `toggle-group`, `toolbar`, `virtual-list`); leave this
  named explicitly as follow-up scope for a future change rather than
  silently expanding this one.
