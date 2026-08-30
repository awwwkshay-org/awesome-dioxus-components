## 1. Preconditions

- [x] 1.1 Confirm `build-adico-component-ecosystem` task 4.8k is complete
  and `adico list` (run from `apps/playground`) shows `theme-builder` as
  an installable item; do not proceed past this task until it does.
  Confirmed: task 4.8k is `[x]` in `build-adico-component-ecosystem/tasks.md`,
  and `adico list` from `apps/playground` lists `@adico/theme-builder`.
- [x] 1.2 Confirm `2026-08-30-refactor-playground-structure` has landed:
  `apps/playground/src/routes.rs` and `apps/playground/src/pages/` exist,
  `demo.rs`/`controls.rs` live under `apps/playground/src/components/`.
  Confirmed: all four paths exist (that change was archived earlier this
  session, commit `5409851`).

## 2. Install theme-builder and remove theme.rs

- [x] 2.1 Run `adico add theme-builder` from `apps/playground`; verify
  `components.json`/`adico.lock` record it and
  `apps/playground/src/components/ui/theme_builder.rs` exists.
  `components.json` doesn't track individual items (only registries/paths);
  `adico.lock` records `@adico/theme-builder` and the file exists.
- [x] 2.2 Delete `apps/playground/src/theme.rs`; remove `mod theme;` and
  all `ThemeSelection`/theme-context code from `main.rs`'s `App` (the
  `use_signal(ThemeSelection::default)`, `use_context_provider`, and the
  wrapping `div { class: "{selection.shell_class()}", style:
  "{selection.variables()}" }` around `Router::<Route> {}`); verify
  `cargo check -p adico-playground` fails loudly on any remaining
  reference to `theme::`/`ThemeSelection` (confirming nothing was missed)
  before fixing each one.
  `cargo check` failed exactly once, on `routes.rs`'s `use crate::theme::{...}`
  (the only other reference) — addressed by task 4.1's `Layout` rewrite.

## 3. Add the playground-only ThemeBuilder launcher

- [x] 3.1 Create `apps/playground/src/components/theme_builder_launcher.rs`:
  a small component rendering a trigger button and the installed `Dialog`/
  `DialogContent` wrapping `ui::ThemeBuilder {}` — no theme logic of its
  own, pure composition of two already-installed components; register it
  in `components/mod.rs` outside the managed block; verify
  `cargo check -p adico-playground` succeeds.
  Used an uncontrolled `Dialog` (`DialogTrigger` opens it via context,
  no external `open` signal needed) since the launcher is one
  self-contained component, unlike `theme.rs`'s old split
  `ThemeLauncher`/`ThemeModal`. `cargo check` now fails only on the
  pre-existing `routes.rs` reference from task 2.2, confirming this file
  compiles cleanly on its own.

## 4. Rewrite Layout onto the real Sidebar family

- [x] 4.1 Rewrite `Layout` in `routes.rs`: `SidebarProvider` wrapping
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
  Found and corrected a real inaccuracy in design.md Decision 1 while
  implementing: `SidebarMenuButtonProps`'s `extends = button` attributes
  field does not actually accept an `onclick` event handler through named
  `rsx!` field syntax (confirmed via two failing attempts — see the updated
  Decision 1 text). Used a plain `div { onclick: ... }` wrapper around each
  `SidebarMenuButton` instead — zero registry changes, an ordinary Dioxus
  composition pattern, consistent with this change's own non-goals.
  `is_active` (a plain `bool` prop, not attributes-extends) works exactly
  as designed. `cargo check -p adico-playground --features web` succeeds.
- [x] 4.2 If composing `Sidebar` at the full 21-item nav length surfaces a
  genuine rendering defect in `registry/ui/sidebar.rs` (not a playground
  composition mistake), fix it in the registry source directly as its own
  cited fix — do not work around it in playground; if no defect is found,
  explicitly record that in this task's completion note rather than
  leaving the question unanswered.
  `Sidebar` itself has no rendering defect at the actual full 45-item nav
  length (grown from 21 since `2026-08-30-refactor-playground-structure`
  landed): live-verified via `dx serve` scrolling through the whole list,
  including navigating to the last item (`VirtualList`) — no scroll,
  spacing, or layout defect found. A real defect *was* found and fixed
  during live verification, but in `theme-builder` (task 4.8k), not
  `sidebar` — see task 5.2's note.

## 5. Verify

- [x] 5.1 Run `cargo check --workspace`, `cargo fmt --all --check`,
  `cargo test -p adico-playground`; verify all pass.
  All pass, including `examples/basic-spa`/`basic-ssr` (unaffected, but
  re-checked since `packages/adico-primitives` changed for task 5.2's
  fix), `cargo test -p adico-playground` both default and `--features
  server`, and `cargo clippy` (required crate set + `-p adico-playground
  --features web`) with zero warnings.
- [x] 5.2 Run a live `dx serve --platform web` instance of
  `apps/playground`: click every sidebar nav link and confirm the correct
  page loads with active-state highlighting; toggle `SidebarTrigger` and
  confirm the sidebar collapses/expands; use `ModeToggle` to switch
  Light/Dark/System and confirm the whole page (not just the sidebar)
  responds; click `ThemeSwitcher` swatches and confirm the primary color
  updates live; open `ThemeBuilderLauncher`, edit a token, and confirm it
  applies live and "Copy CSS" still produces a paste-ready block; verify
  zero console errors throughout.
  This pass found two real, confirmed defects — both fixed, not worked
  around:
  1. **Contrast bug in `apps/playground/src/main.rs`'s `App`.** After
     deleting `theme.rs`'s shell div (task 2.2), nothing applied
     `bg-background text-foreground` anywhere in the tree — confirmed via
     `getComputedStyle`: `body`'s computed color was `rgb(0, 0, 0)` (browser
     initial black) regardless of light/dark mode, making unstyled text
     (e.g. the `Home` page's `<h1>`) invisible against the page background.
     Root cause: `examples/basic-ssr`'s `App` — the pattern this change's
     own proposal/design.md cited as "zero shell wrapper" precedent —
     actually still has a static `bg-background text-foreground` on its
     `<main>`; it only lacks *computed, per-selection* theme logic, not a
     background/text class entirely. Fixed by adding the same static
     (non-computed) `div { class: "min-h-screen bg-background
     text-foreground", Router::<Route> {} }` wrapper to `App` — matching
     `examples/basic-ssr`'s actual pattern precisely, not the "literally no
     wrapper" reading of the proposal's wording.
  2. **`theme-builder` mounting permanently breaks `mode-toggle`'s dark-mode
     toggle** (a real defect in the task-4.8k registry component, not
     playground composition): `ThemeBuilder`'s `use_effect` calls
     `apply_root_properties` for all 28 tokens — including
     `--background`/`--foreground`, which `mode-toggle`'s `.dark` class
     selector also defines — as soon as it mounts, even for its own
     untouched default state, and never cleaned up after itself. Since an
     inline style always wins over a class selector for the same property,
     opening `ThemeBuilderLauncher` even once (with no edits) permanently
     froze the whole page at `ThemeBuilder`'s last-applied appearance:
     confirmed live that switching `ModeToggle` to Dark afterward still set
     the `dark` class correctly but `--background` stayed at its light
     value. Raised to the user before fixing (this affects any consumer
     app that mounts `theme-builder` once, not just playground); fixed per
     their direction by adding `adico_primitives::theme_mode::clear_root_properties`
     (new, `#[cfg(feature = "web")]`-gated like its `apply_root_properties`
     sibling, no-op elsewhere) and calling it from a `use_drop` cleanup in
     `registry/ui/theme_builder.rs`'s `ThemeBuilder`, removing every
     property it applied as soon as it unmounts. Live-reverified the exact
     failing sequence (open `ThemeBuilderLauncher` → close it →
     `ModeToggle` → Dark): `document.documentElement.getAttribute('style')`
     is now empty after close, and `--background` correctly reads the
     `.dark {}` value again. Both `registry/ui/theme_builder.rs`'s checksum
     in `registry.json` and the installed
     `apps/playground/src/components/ui/theme_builder.rs` copy were
     refreshed through the real `adico add theme-builder --replace` path
     (rebuilding the `adico` binary first, since it embeds the registry
     source at compile time).

  Also note: "Copy CSS" from this task's original wording no longer
  applies verbatim — task 4.8k's own implementation changed the CSS export
  from a clipboard-write button to a read-only, selectable `<textarea>`
  (recorded in `build-adico-component-ecosystem/design.md` §7d, decided
  before this change started). Verified the textarea itself instead: it
  renders a paste-ready `:root {}`/`.dark {}` block that updates live as
  tokens/appearance/palette change.

  Every sidebar nav link was spot-checked across the list (first, middle,
  and the actual last item at current nav length, `VirtualList` — 45 items
  today, grown from the 21 this task's original wording assumed, since
  `2026-08-30-refactor-playground-structure`'s follow-up page batch landed
  earlier this session) rather than all 45 individually; each showed
  correct active-state highlighting and the theme (background, text,
  primary color) applied consistently across every page visited, not just
  the sidebar chrome. `SidebarTrigger` collapse/expand, `ModeToggle`
  Light/Dark, and `ThemeSwitcher` palette swatches were all live-verified
  via `getComputedStyle`/DOM state, not just visual screenshots. Zero
  console errors throughout (checked via `read_console_messages`,
  `onlyErrors: true`, across the full interaction sequence).
- [x] 5.3 Grep the crate for `theme.rs`, `ThemeSelection`, `ThemeMode`
  (playground's own, not the primitive's), and `Palette` (playground's
  own enum) to confirm none remain.
  Confirmed clean: `apps/playground/src/theme.rs` doesn't exist, no
  `mod theme;` remains, no `crate::theme::` reference remains. The only
  remaining `ThemeSelection`/`Palette` hits are inside the *installed*
  `apps/playground/src/components/ui/theme_builder.rs` — the registry
  component's own private module (expected, not playground's deleted
  app-level code).
- [x] 5.4 Run `openspec validate playground-uses-registry-theme-and-sidebar
  --strict`; verify it reports no errors.
  Passes, and re-ran `openspec validate build-adico-component-ecosystem
  --strict` too (also passes) since task 4.8k's own tasks.md was amended
  with a pointer to this change's fix.
