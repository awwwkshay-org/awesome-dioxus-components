## Why

`apps/playground` currently hand-rolls three things the registry already
ships as real, installable components: a `ThemeMode` enum
(`theme.rs`, `Light`/`Dark` only) that duplicates and is strictly weaker
than the already-installed `ModeToggle` (`Light`/`Dark`/`System`,
persisted, OS-aware); a primary-palette HSL table byte-identical to
`ThemeSwitcher`'s; and a hand-rolled `<nav><ul><li>` sidebar in `Layout`
that reimplements the registry `Sidebar` family's structure and classes.
`ModeToggle` and `ThemeSwitcher` are already installed in
`apps/playground/src/components/ui/` (from the prior full-catalog install)
but referenced nowhere in playground source — dead code, and the clearest
signal this drifted. Playground also applies its theme through an inline
`style`/class on a shell `<div>`, a mechanism that actively conflicts with
`ThemeSwitcher`'s `:root`-level custom properties (an inline style on a
descendant always wins), so simply dropping the installed components in
today would render broken. Per this session's standing rule — the registry
only grows for genuine, generic component needs, and playground exists to
demonstrate real components, not to maintain parallel implementations of
them — this change rewires playground onto the real components instead.

This change depends on `build-adico-component-ecosystem` task 4.8k
(`theme-builder`, a new registry component productizing `theme.rs`'s one
genuinely novel piece — the full 28-token editor, independent light/dark
values, CSS export, and "generate theme") having landed first, and assumes
`2026-08-30-refactor-playground-structure`'s `routes.rs`/`pages/`/
`components/demo.rs`,`controls.rs` layout is already in place.

## What Changes

- Delete `apps/playground/src/theme.rs` and `ThemeSelection` entirely.
  `App` stops applying any inline shell class/style; the document root is
  styled purely by whatever `ModeToggle`/`ThemeSwitcher`/`ThemeBuilder`
  write to `document.documentElement`, matching how `examples/basic-ssr`
  already composes `ModeToggle` + `ThemeSwitcher` with no shell wrapper.
- Install `theme-builder` (via `adico add theme-builder`) into
  `apps/playground`, alongside the already-installed but currently unused
  `mode-toggle`/`theme-switcher`.
- Rewrite `Layout` (in `routes.rs`) to compose the real `Sidebar` family
  (`SidebarProvider`, `Sidebar`, `SidebarHeader`, `SidebarContent`,
  `SidebarGroup`, `SidebarMenu`, `SidebarMenuItem`, `SidebarMenuButton`,
  `SidebarFooter`, `SidebarRail`, `SidebarInset`, `SidebarTrigger`)
  instead of the hand-rolled `<nav>`, matching the composition already
  proven in `examples/basic-spa`'s `SidebarDemo`. Navigation uses
  `SidebarMenuButton`'s `onclick` + `use_navigator()` (not a nested
  router `Link`, since `SidebarMenuButton` always renders a native
  `<button>` with no `as_child` escape hatch) plus `is_active` for the
  current route — a real usability improvement (active-link highlighting)
  that falls out of using the real component, not scope creep.
- `SidebarFooter` composes `ModeToggle` + `ThemeSwitcher` + a small,
  playground-specific launcher (a `Dialog` wrapping the installed
  `ThemeBuilder`) for `apps/playground/src/components/` — the launcher
  itself is playground UI wiring, not a registry component.
- No change to `registry/ui/*.rs` in this change beyond what task 4.8k
  already added. If composing the real `Sidebar` at full navigation
  length surfaces a genuine rendering defect, that gets fixed in the
  registry source directly (matching this session's established
  5.3b/5.3c pattern), never worked around in playground.

## Capabilities

### Modified Capabilities

- `adico-playground-structure`: adds a requirement that the playground
  shell composes real registry components (`Sidebar` family,
  `ModeToggle`, `ThemeSwitcher`, `theme-builder`) for navigation and theme
  controls instead of maintaining app-specific reimplementations, and
  removes the now-inaccurate implication that `theme.rs` is a permanent
  fixture.

## Impact

- Affected code: `apps/playground/src/theme.rs` (deleted),
  `apps/playground/src/routes.rs`'s `Layout` (rewritten),
  `apps/playground/src/main.rs`'s `App` (drops the inline shell
  class/style), `apps/playground/components.json`/`adico.lock`
  (`theme-builder` added), new
  `apps/playground/src/components/theme_builder_launcher.rs` (or similar
  small playground-only wiring file — exact name decided during
  implementation).
- Depends on `build-adico-component-ecosystem` task 4.8k landing first
  (adds the `theme-builder` registry item this change installs).
- Assumes `2026-08-30-refactor-playground-structure` has landed
  (`routes.rs`/`pages/`/`components/demo.rs`,`controls.rs`); this change
  edits `routes.rs`'s `Layout`, which only exists once that change lands.
- No change to `parity.json` — playground is a development/demo surface,
  not a `parity.json` evidence source; `theme-builder` itself is
  `parity.json`-exempt per task 4.8k (Dioxus-only extra).
- No change to any other `examples/*`/`tests/installation/*` fixture.
