## Context

Direct exploration of `apps/playground` (this session) confirmed three
concrete drifts from "playground demonstrates real registry components":

1. `theme.rs`'s `ThemeMode { Light, Dark }` is a separate enum from
   `adico_primitives::theme_mode::ThemeMode { Light, Dark, System }`, and
   is strictly weaker (no `System`, no persistence, no OS-detection via
   `dark-light`). `theme.rs`'s `ThemeModeControl` (a `<select>`) is a
   worse `ModeToggle`.
2. `theme.rs`'s `Palette::primary_tokens` HSL table (12 pairs) is
   byte-identical to `ThemeSwitcher`'s `ThemePalette::primary_hsl` —
   pure duplication, already acknowledged in `theme_switcher.rs`'s own
   doc comment.
3. `main.rs`'s `Layout` hand-rolls `<nav><ul><li>` with classes
   (`hover:bg-sidebar-accent hover:text-sidebar-accent-foreground`, etc.)
   that mirror the registry `Sidebar` family's own classes, instead of
   using `Sidebar`/`SidebarMenu`/`SidebarMenuButton` directly.
4. `ModeToggle` and `ThemeSwitcher` are already installed in
   `apps/playground/src/components/ui/` and exported from `mod.rs`, but
   used nowhere in playground source.
5. The application mechanisms are incompatible: playground's `App`
   applies `ThemeSelection::variables()` as an inline `style` on a shell
   `<div>` and `shell_class()` for the `dark` class, while the registry's
   `apply_root_properties`/`apply_resolved_class` write to
   `document.documentElement`. Dropping `ThemeSwitcher` into the current
   shell as-is would render broken (inline style wins over a `:root`
   custom property on a descendant).

`build-adico-component-ecosystem` task 4.8k (design.md §7d) adds
`theme-builder`, a real registry component productizing `theme.rs`'s one
genuinely novel piece (the full 28-token editor, independent light/dark
values, CSS export, "generate theme"), applying its tokens through the
same `apply_root_properties` mechanism `theme-switcher` already uses. This
change is the playground-side half: once `theme-builder` exists, nothing
in `theme.rs` needs to keep existing.

## Goals / Non-Goals

**Goals:**

- Delete `theme.rs` and its duplicated `ThemeMode`/`Palette` entirely —
  not migrate, not shrink, remove.
- Make playground's shell theme-agnostic: no inline style/class applied
  by playground code anywhere; the document root is styled purely by the
  installed `mode-toggle`/`theme-switcher`/`theme-builder` components,
  the same as a real consumer app gets.
- Replace the hand-rolled nav with the real `Sidebar` family, composed
  the same way `examples/basic-spa`'s `SidebarDemo` already proves works.
- Keep any genuinely playground-specific wiring (e.g. a small dialog
  launcher for `ThemeBuilder`) under `apps/playground/src/components/`,
  never inlined into `routes.rs` or leaked into the registry.

**Non-Goals:**

- Modifying `registry/ui/sidebar.rs`, `mode_toggle.rs`, `theme_switcher.rs`,
  or `theme_builder.rs` preemptively for playground's convenience. If
  composing them at full navigation length surfaces a genuine rendering
  defect, fix it in the registry directly (own task/commit), but this
  change does not budget for a registry fix it hasn't found yet.
  matching the 5.3b/5.3c precedent already established this session.
- Adding pages for the 24 components still missing one (unrelated,
  already tracked as follow-up scope in
  `2026-08-30-refactor-playground-structure`).
- Persisting `ThemeBuilder`'s edited tokens across reloads. `mode-toggle`
  already persists mode; a full custom-theme's persistence is
  `theme-builder`'s own concern (task 4.8k), not something this change's
  playground wiring adds on top.

## Decisions

### 1. `SidebarMenuButton` navigation uses `onclick` + `use_navigator()`, not a nested `Link`

`SidebarMenuButton` (confirmed via direct source read) always renders a
native `<button type="button">` with no `as_child`/polymorphic escape
hatch — there is no way to make it render an `<a>` instead. Nesting a
router `Link` (itself an `<a>`) inside it would be invalid HTML
(interactive element inside interactive element) and is not how any
existing `Sidebar` usage in this repo composes navigation.

Confirmed during implementation that `SidebarMenuButtonProps`'s
`#[props(extends = GlobalAttributes)] #[props(extends = button)]`
`attributes: Vec<Attribute>` field does not actually accept an `onclick`
event handler through named-field `rsx!` syntax — unlike `registry/ui/button.rs`'s
`Button`, which declares its own explicit `onclick: EventHandler<MouseEvent>`
field precisely because the generic `extends` mechanism doesn't
auto-generate event-handler setters, only plain-attribute ones. Two direct
attempts confirmed this: a bare `onclick: ...` field fails with "no method
named `onclick` found ... in `SidebarMenuButtonPropsBuilder`", and a quoted
`"onclick": ...` fails because it's then treated as a plain string
attribute, not a listener. So each nav item instead wraps its
`SidebarMenuButton` in a plain `div` carrying
`onclick: move |_| navigator.push(route.clone())`
(`dioxus_router::hooks::use_navigator`) — an ordinary Dioxus composition
pattern (a click-catching wrapper around a native button; clicks on the
inner button bubble to the wrapper) needing zero registry changes, matching
this change's own non-goals and the sibling spec's "compose around the
existing API using ordinary Dioxus patterns" fallback. `SidebarMenuButton`
still gets `is_active: current_route == route` directly (that prop *is*
plain `bool`, not part of the attributes-extends set, and works as
expected) — which the current hand-rolled `<nav>` never had, since `Link`
alone doesn't compute active state. This is a direct, real usability
improvement from adopting the real component, not something added for its
own sake.

Alternative considered: modify `SidebarMenuButton` to accept an
`as_child`-style render-prop so a `Link` could be nested/substituted.
Rejected outright per this change's own non-goals — that's exactly the
"modify the registry because playground needs it" pattern this whole
change exists to stop. `onclick` + `use_navigator()` needs zero registry
changes and is a completely ordinary way to make a button navigate.

### 2. Theme application is 100% delegated to the installed components; playground's shell carries no theme logic

`App` (in `main.rs`) currently does
`let theme = use_signal(ThemeSelection::default); use_context_provider(|| theme);`
and wraps `Router::<Route> {}` in
`div { class: "{selection.shell_class()}", style: "{selection.variables()}" }`.
All of that is deleted. `App` becomes a plain shell: asset links,
`document::Title`, and `Router::<Route> {}` — no theme context, no signal,
no *computed, per-selection* wrapper. Every page keeps working unmodified
because none of them ever read the deleted `ThemeSelection` context
directly (confirmed: only `Layout`/`App` touched it).

Corrected during implementation: `examples/basic-ssr` does not actually
mount `ModeToggle`/`ThemeSwitcher` with *zero* shell wrapper, as first
written above — its own `App` has a static `main { class: "min-h-screen
space-y-8 bg-background p-8 text-foreground", ... }`. Live-verified this
distinction matters: without any `bg-background`/`text-foreground`
anywhere in the tree, `body`'s computed text color is the browser's
initial black regardless of light/dark mode, making unstyled text (e.g.
`Home`'s `<h1>`) invisible. `App` therefore keeps a static (not
computed, not signal-driven) `div { class: "min-h-screen bg-background
text-foreground", Router::<Route> {} }` — the same class string on every
render regardless of theme state, matching `examples/basic-ssr`'s actual
pattern exactly, as distinct from `theme.rs`'s deleted `shell_class()`
(which recomputed the class per `ThemeSelection`).

### 3. `ThemeBuilder`'s launcher is playground-only wiring, lives in `components/`, wraps but never forks the registry component

The registry `ThemeBuilder` component itself is self-contained (per
design.md §7d) — it doesn't know or care whether it's opened from a
sidebar footer button, a settings page, or anywhere else. Playground
needs *some* trigger to open it (matching `theme.rs`'s old
`ThemeLauncher`/`ThemeModal` pattern, which reused the installed
`Dialog`). That trigger — a small component that renders a button and a
`Dialog`/`DialogContent` containing `ui::ThemeBuilder {}` — is
playground-specific composition, not registry material, so it lives at
`apps/playground/src/components/` (e.g. `theme_builder_launcher.rs`),
exactly like `demo.rs`/`controls.rs` do after
`2026-08-30-refactor-playground-structure`. It contains zero theme logic
of its own — it only composes `Dialog` (installed) + `ThemeBuilder`
(installed).

### 4a. `theme-builder` needed an unmount-cleanup fix, found while wiring the launcher

Live-verifying `ThemeBuilderLauncher` (per the Migration Plan's step 7)
surfaced a genuine defect in `theme-builder` itself (task 4.8k), not a
playground composition issue: `ThemeBuilder`'s `use_effect` applies all 28
tokens — including `--background`/`--foreground`, which `mode-toggle`'s
`.dark` class selector also defines — via
`adico_primitives::theme_mode::apply_root_properties` as soon as it
mounts, even to display its own untouched default state. Since an inline
style always wins over a class-selector rule for the same property, and
nothing removed those properties on unmount, opening
`ThemeBuilderLauncher` even once permanently froze the whole page at
`ThemeBuilder`'s last-applied appearance — breaking `mode-toggle`'s
Light/Dark toggle for the rest of the session, in *any* consumer app that
mounts `theme-builder`, not just playground.

Raised to the user (this is a task-4.8k registry defect, confirmed via
`document.documentElement`'s inline `style` attribute and
`getComputedStyle`, not something playground's composition could work
around) with three options: clean up on unmount, gate the initial
`apply_root_properties` call behind a "has the user actually edited
anything" flag, or pause this change entirely. Chose the unmount-cleanup
fix: added `adico_primitives::theme_mode::clear_root_properties(names:
&[&str])` (new, `#[cfg(feature = "web")]`-gated exactly like its
`apply_root_properties` sibling, no-op elsewhere — calls
`root.style.removeProperty(name)` per name via `dioxus_document::eval`),
and call it from a `use_drop` cleanup in `ThemeBuilder` with every
property name `ThemeVariables::light().root_property_pairs()` names.
Rejected gating on a "touched" flag alone: it only narrows the window (the
poisoning would still occur after the user's first real edit, once they
close the dialog), so it doesn't fix the underlying problem by itself.

`registry/ui/theme_builder.rs`'s checksum in `registry.json` and the
installed `apps/playground/src/components/ui/theme_builder.rs` copy were
both refreshed through the real `adico add theme-builder --replace` path
(rebuilding the `adico` binary first, since it embeds registry source at
compile time) — not hand-patched. Live-reverified the exact failing
sequence afterward: open the launcher, close it, then switch `ModeToggle`
to Dark — `--background` now correctly reads the `.dark {}` value again.

### 4. Sequencing: this change cannot start before 4.8k lands

`ThemeBuilder` doesn't exist until `build-adico-component-ecosystem` task
4.8k is implemented and the registry rebuilt. Attempting this change's
`adico add theme-builder` step before then would simply fail with an
unknown-component error. This is recorded as a hard dependency, not an
assumption to be verified only at execution time.

## Risks / Trade-offs

- [Deleting `theme.rs` removes the only place demonstrating "edit every
  semantic token live," reducing playground's usefulness as a parity-
  inspection surface until `theme-builder`'s launcher is wired] →
  Sequenced so `theme-builder`'s installation and launcher wiring land in
  the same change that deletes `theme.rs` — there is no committed
  intermediate state where playground has neither.
- [`Sidebar` at 21+ nav-item length surfaces a real registry defect
  (e.g. scroll behavior, spacing) not seen in the shorter `SidebarDemo`
  example] → If found, fixed in `registry/ui/sidebar.rs` directly as its
  own explicit, cited fix (matching the 5.3b/5.3c pattern), not routed
  around in playground; tasks include an explicit live `dx serve` check
  at full nav length specifically to surface this before calling the
  change done.
- [`onclick` + `use_navigator()` navigation behaves subtly differently
  from `Link` — e.g. missing `prefetch`/right-click "open in new tab"
  support `Link` gives for free] → Named explicitly, not silently
  accepted: right-click/middle-click "open in new tab" on a sidebar nav
  item will not work the same way it does with a real `<a href>`. This is
  an inherent trade-off of `SidebarMenuButton` having no `as_child`
  escape hatch (see Decision 1), not something this change can avoid
  without modifying the registry component, which is explicitly out of
  scope. Recorded here rather than discovered later as a "regression."

## Migration Plan

1. Confirm `build-adico-component-ecosystem` task 4.8k is done and
   `theme-builder` is a real registry item (`adico list` shows it).
2. Confirm `2026-08-30-refactor-playground-structure` has landed
   (`routes.rs`, `pages/`, `components/demo.rs`,`controls.rs` exist).
3. `adico add theme-builder` in `apps/playground`.
4. Delete `apps/playground/src/theme.rs`; remove `mod theme;` and the
   `ThemeSelection`/theme-context wiring from `main.rs`'s `App`.
5. Add `apps/playground/src/components/theme_builder_launcher.rs` (Dialog
   + installed `ThemeBuilder`); register it in `components/mod.rs`
   outside the managed block.
6. Rewrite `Layout` in `routes.rs`: `SidebarProvider > Sidebar >
   (SidebarHeader with brand link, SidebarContent > SidebarGroup >
   SidebarMenu > SidebarMenuItem > SidebarMenuButton per nav_items() with
   onclick+is_active, SidebarFooter with ModeToggle + ThemeSwitcher +
   ThemeBuilderLauncher, SidebarRail) > SidebarInset > SidebarTrigger +
   Outlet::<Route>`.
7. Verify: `cargo check --workspace`, `cargo fmt --all --check`, live
   `dx serve` exercising every route, sidebar collapse via
   `SidebarTrigger`, `ModeToggle`'s Light/Dark/System, `ThemeSwitcher`'s
   palette swatches, and `ThemeBuilder`'s live token edits + CSS export;
   `openspec validate playground-uses-registry-theme-and-sidebar --strict`.

No rollback beyond `git revert`; touches only `apps/playground`.
