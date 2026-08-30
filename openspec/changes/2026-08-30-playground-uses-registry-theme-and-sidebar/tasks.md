## 1. Preconditions

- [ ] 1.1 Confirm `build-adico-component-ecosystem` task 4.8k is complete
  and `adico list` (run from `apps/playground`) shows `theme-builder` as
  an installable item; do not proceed past this task until it does.
- [ ] 1.2 Confirm `2026-08-30-refactor-playground-structure` has landed:
  `apps/playground/src/routes.rs` and `apps/playground/src/pages/` exist,
  `demo.rs`/`controls.rs` live under `apps/playground/src/components/`.

## 2. Install theme-builder and remove theme.rs

- [ ] 2.1 Run `adico add theme-builder` from `apps/playground`; verify
  `components.json`/`adico.lock` record it and
  `apps/playground/src/components/ui/theme_builder.rs` exists.
- [ ] 2.2 Delete `apps/playground/src/theme.rs`; remove `mod theme;` and
  all `ThemeSelection`/theme-context code from `main.rs`'s `App` (the
  `use_signal(ThemeSelection::default)`, `use_context_provider`, and the
  wrapping `div { class: "{selection.shell_class()}", style:
  "{selection.variables()}" }` around `Router::<Route> {}`); verify
  `cargo check -p adico-playground` fails loudly on any remaining
  reference to `theme::`/`ThemeSelection` (confirming nothing was missed)
  before fixing each one.

## 3. Add the playground-only ThemeBuilder launcher

- [ ] 3.1 Create `apps/playground/src/components/theme_builder_launcher.rs`:
  a small component rendering a trigger button and the installed `Dialog`/
  `DialogContent` wrapping `ui::ThemeBuilder {}` — no theme logic of its
  own, pure composition of two already-installed components; register it
  in `components/mod.rs` outside the managed block; verify
  `cargo check -p adico-playground` succeeds.

## 4. Rewrite Layout onto the real Sidebar family

- [ ] 4.1 Rewrite `Layout` in `routes.rs`: `SidebarProvider` wrapping
  `Sidebar` (containing `SidebarHeader` with the existing brand
  link/logo, `SidebarContent > SidebarGroup > SidebarMenu` with one
  `SidebarMenuItem > SidebarMenuButton` per `nav_items()` entry using
  `onclick: move |_| navigator().push(route)` and
  `is_active: current_route() == route` per design.md Decision 1,
  `SidebarFooter` composing `ui::ModeToggle {}` + `ui::ThemeSwitcher {}` +
  `ThemeBuilderLauncher {}`, `SidebarRail`) and `SidebarInset` (containing
  `SidebarTrigger` + `Outlet::<Route> {}`); delete the old hand-rolled
  `<nav><ul><li>` markup entirely; verify `cargo check -p adico-playground`
  succeeds.
- [ ] 4.2 If composing `Sidebar` at the full 21-item nav length surfaces a
  genuine rendering defect in `registry/ui/sidebar.rs` (not a playground
  composition mistake), fix it in the registry source directly as its own
  cited fix — do not work around it in playground; if no defect is found,
  explicitly record that in this task's completion note rather than
  leaving the question unanswered.

## 5. Verify

- [ ] 5.1 Run `cargo check --workspace`, `cargo fmt --all --check`,
  `cargo test -p adico-playground`; verify all pass.
- [ ] 5.2 Run a live `dx serve --platform web` instance of
  `apps/playground`: click every sidebar nav link and confirm the correct
  page loads with active-state highlighting; toggle `SidebarTrigger` and
  confirm the sidebar collapses/expands; use `ModeToggle` to switch
  Light/Dark/System and confirm the whole page (not just the sidebar)
  responds; click `ThemeSwitcher` swatches and confirm the primary color
  updates live; open `ThemeBuilderLauncher`, edit a token, and confirm it
  applies live and "Copy CSS" still produces a paste-ready block; verify
  zero console errors throughout.
- [ ] 5.3 Grep the crate for `theme.rs`, `ThemeSelection`, `ThemeMode`
  (playground's own, not the primitive's), and `Palette` (playground's
  own enum) to confirm none remain.
- [ ] 5.4 Run `openspec validate playground-uses-registry-theme-and-sidebar
  --strict`; verify it reports no errors.
