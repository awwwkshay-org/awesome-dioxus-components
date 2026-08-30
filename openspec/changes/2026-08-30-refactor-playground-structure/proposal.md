## Why

`apps/playground` grew its router and every page into two flat top-level
files: `src/main.rs` (171 lines) carries the `Route` enum, `nav_items()`,
the `Layout` shell, `Home`, asset consts, and the `App` entrypoint all
together, and `src/pages.rs` (911 lines) carries all 21 currently-routed
page components in one file. The just-completed full-catalog install (24
more components added to `src/components/ui/`) has no pages or routes yet,
and adding them the current way would keep growing both files without
bound. Splitting routing concerns into their own module and giving each
page its own file under a `pages/` directory now, before the next batch of
pages is added, keeps the playground navigable and keeps future component
pages a one-file addition instead of an edit to an already-large shared
file.

## What Changes

- Add `apps/playground/src/routes.rs`: the `Route` enum, `nav_items()`, and
  the `Layout` shell component move here from `main.rs`. `main.rs` keeps
  only the `App` entrypoint, its asset consts, and the `adico:start`/
  `adico:end` managed module block.
- Replace `apps/playground/src/pages.rs` with `apps/playground/src/pages/`,
  using a TanStack Start-style file-per-route directory convention: one
  file per page named after its route's path segment (`pages/button.rs` for
  `/button`, `pages/badge.rs` for `/badge`, ..., `pages/sidebar.rs` for
  `/sidebar`), and `pages/index.rs` for the root route (`/`) — TanStack
  Start's own convention for a directory's index route — replacing the
  `Home` component currently defined in `main.rs`. All files are
  aggregated by `pages/mod.rs`. (dioxus-router has no file-system route
  generation the way TanStack Start does, so `routes.rs`'s `Route` enum
  still declares each path explicitly; this borrows only the file-naming/
  layout convention, not automatic route derivation — see design.md
  Decision 1.) The two `DatePickerPage` SSR-render tests move into
  `pages/date_picker.rs`'s own `#[cfg(test)]` module instead of a single
  shared test block at the end of the old `pages.rs`.
- No page's rendered output, props, or route path changes. No component
  demo composition changes. `demo.rs`, `controls.rs`, and `theme.rs` are
  unaffected — they stay shared top-level modules imported by page files,
  not moved under `pages/`.

## Capabilities

### New Capabilities

- `adico-playground-structure`: Defines where `apps/playground`'s router
  and page components live — `routes.rs` for routing/navigation/shell,
  `pages/` with one file per page component — so future page additions
  have a fixed place to land instead of growing an existing shared file.

### Modified Capabilities

- None.

## Impact

- Affected code: `apps/playground/src/main.rs`, `apps/playground/src/pages.rs`
  (deleted), new `apps/playground/src/routes.rs` and
  `apps/playground/src/pages/{mod.rs,index.rs,button.rs,badge.rs,card.rs,
  input.rs,textarea.rs,skeleton.rs,item.rs,pagination.rs,dialog.rs,sheet.rs,
  select.rs,combobox.rs,tooltip.rs,popover.rs,hover_card.rs,dropdown_menu.rs,
  context_menu.rs,menubar.rs,calendar.rs,date_picker.rs,sidebar.rs}`.
- No change to `apps/playground/Cargo.toml`, `components.json`,
  `adico.lock`, installed `src/components/ui/*.rs`, or `tailwind.css`/
  `assets/tailwind.css` — this is a pure internal module reorganization of
  already-working code, not a component or CLI change.
- No change to `parity.json` or any registry/provenance record — playground
  is a development/demo surface, not a `parity.json` evidence source.
- Out of scope (explicit non-goal, not silently dropped): adding pages or
  routes for the 24 components installed in the prior session
  (`accordion`, `alert-dialog`, `aspect-ratio`, `avatar`, `checkbox`,
  `collapsible`, `color-picker`, `drag-and-drop-list`, `label`,
  `mode-toggle`, `progress`, `radio-group`, `scroll-area`, `slider`,
  `switch`, `tabs`, `tag-group`, `theme-switcher`, `toast`, `toggle`,
  `toggle-group`, `toolbar`, `virtual-list`) that have no page today. This
  change only reorganizes the 21 pages that already exist; a follow-up
  change adds the missing pages into the new structure this change
  establishes.
