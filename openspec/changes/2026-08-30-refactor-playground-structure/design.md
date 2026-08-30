## Context

`apps/playground/src/main.rs` currently defines, in one file: the asset
consts, the `App` root component, the `Route` enum (21 routed pages plus
`Home`), `nav_items()` (the sidebar link list, hand-kept in sync with
`Route`), the `Layout` shell (sidebar nav + `Outlet` + theme modal), and the
`Home` landing page. `apps/playground/src/pages.rs` defines all 21 page
components plus a small `#[cfg(test)]` module with two `DatePickerPage`
SSR-render tests. Both files grew this way because the playground started
with a handful of components and every `adico add` since M2 appended one
more `#[route(...)]` variant, one more `nav_items()` entry, and one more
`XPage` function to the nearest existing file rather than a per-page
location. The playground now has 45 installed components (see the prior
session's full-catalog install) and only 21 pages; closing that gap without
first giving pages a fixed per-file home would mean 24 more edits to an
already 911-line file.

## Goals / Non-Goals

**Goals:**

- Give the router a single, obvious home (`routes.rs`) separate from the
  app entrypoint and separate from page bodies.
- Give every page its own file under `pages/`, so adding a page is "add a
  file and register it in `pages/mod.rs`" instead of "find a good insertion
  point in a 900-line file."
- Preserve every current page's rendered behavior, props, and route path
  exactly — this is a pure move/split, not a rewrite.
- Establish `adico-playground-structure` as the spec future page-adding
  changes (including the follow-up that adds pages for the 24 components
  with none today) are checked against.

**Non-Goals:**

- Adding pages/routes for any of the 24 components that don't have one yet
  — explicitly deferred to a follow-up change (see proposal.md Impact).
- Restructuring `demo.rs`, `controls.rs`, or `theme.rs`. They are shared
  UI helpers used *by* pages, not page bodies themselves, and stay at
  `src/` top level.
- Changing navigation UX, page layout, or the `Demo`/control-panel
  composition pattern pages use today.
- Introducing nested route layouts, route groups, or any router feature
  beyond what already exists (`#[layout(Layout)]` + flat routes).

## Decisions

### 1. One file per page, named by route segment (TanStack Start convention), not one `pages/mod.rs` re-export of the old blob

Considered keeping `pages.rs`'s content as-is but moving it verbatim into
`pages/mod.rs` (satisfying "pages are in a `pages` directory" literally
with the least effort). Rejected: it does nothing for the actual problem
(a single huge file with no per-component boundary) and the next 24-page
addition would still face the same "where do I insert this" problem this
change exists to solve. Splitting now, while the count is 21 and the
change is purely mechanical (each page is already a self-contained
`#[component] fn XPage()` block with no cross-page shared state), is lower
risk than splitting later after more pages accumulate.

Per explicit direction, the split follows TanStack Start's file-based
routing directory convention: one file per route named after its path
segment (`button.rs` for `/button`, `hover_card.rs` for `/hover-card`, ...),
and `index.rs` for a directory's index route — used here for `/`, replacing
the `Home` component's old home in `main.rs`. This is a naming/layout
convention only, not automatic route generation: dioxus-router has no
equivalent of TanStack Start's route-tree codegen (no plugin scans
`pages/` and derives `Route` from the file tree), so `routes.rs`'s `Route`
enum still declares every `#[route("/path")]` explicitly and still has to
be kept in sync by hand when a page is added or removed — this change
narrows *where* new page files land, it does not remove the manual
registration step in `routes.rs`/`pages/mod.rs`. If a future component page
needs a dynamic segment or nested layout, follow TanStack's further
conventions at that point (e.g. a `$param` file-name marker mapped to a
dynamic `#[route("/:param")]` variant) rather than inventing an ad hoc
scheme; none of today's 21 pages need one, so this change does not
introduce that machinery.

### 2. `routes.rs` owns `Route`, `nav_items()`, and `Layout` together

`nav_items()` returns `Vec<(&'static str, Route)>` and exists solely to
drive `Layout`'s sidebar list; `Layout` exists solely to render `Route`
variants via `Outlet::<Route>`. All three are the same concern (how
navigation and the routing shell work) and change together in practice —
splitting them into separate files would add indirection without a
matching benefit. `Home`, by contrast, is a page's *content* like any other
routed page, so it moves into `pages/index.rs` (the TanStack Start name for
a directory's index route, used here for `/`), not `routes.rs`, keeping the
"routes.rs describes how routing works, pages/ holds what each route
renders" boundary clean.

Alternative considered: keep `Layout` in `main.rs` since it's the visual
shell wrapping `App`. Rejected — `Layout` only exists in terms of `Route`
(it renders `Outlet::<Route>` and iterates `nav_items()`), so it is a
routing concern, not an app-bootstrap concern; `main.rs` should be reduced
to what's left once routing and pages are extracted (asset consts, `App`,
the managed `adico:start`/`adico:end` block).

### 3. Keep component/function names unchanged

`Route::Home {}` maps to the `Home` component and `Route::ButtonPage {}`
maps to `ButtonPage`, by dioxus-router's name-matching convention. This
change only moves *where* each component is defined, never renames a
component or a route path — reduces this to a mechanical, low-risk move
verifiable by exact-behavior comparison (identical rendered HTML, identical
route paths) rather than a rewrite that also needs new-name verification.

### 4. Per-page tests move with their page

The two `DatePickerPage` SSR tests (`#[cfg(all(test, feature = "server"))]`
at the bottom of the old `pages.rs`) move into `pages/date_picker.rs`'s own
`#[cfg(test)] mod tests` block, matching this repository's established
per-file test convention (every `registry/ui/*.rs` and
`packages/adico-primitives/src/*.rs` file already keeps its own tests
alongside its own code, per `docs/development.md`). No new tests are added
for other pages by this change — that stays a per-page-addition concern for
whichever change adds a page in the future.

## Risks / Trade-offs

- [Splitting 21 pages by hand risks a copy-paste mistake in one page's
  props/markup] → Tasks require a byte-for-byte diff check: each new
  `pages/*.rs` file's `#[component]` body must match the corresponding
  block in the old `pages.rs` exactly (whitespace/import reordering aside),
  verified by extracting each block and diffing before deleting the old
  file.
- [`main.rs`/`routes.rs`/`pages/mod.rs` import wiring breaks a route or
  drops a nav entry] → Tasks require a live `dx serve` check navigating to
  every one of the 21 routes plus `/`, confirming each renders and the
  sidebar lists all 21 links, not just a compile check.
- [This change silently expands scope to also add the 24 missing pages] →
  Proposal.md and design.md both name this as an explicit non-goal/
  follow-up; tasks.md does not include adding any new page.

## Migration Plan

1. Create `apps/playground/src/routes.rs` with `Route`, `nav_items()`, and
   `Layout` moved from `main.rs` (imports adjusted, e.g. `pages::Home` for
   `Route::Home {}`'s target).
2. Create `apps/playground/src/pages/` with one file per existing page
   (`home.rs` plus the 21 files named after their current `XPage`
   function, snake_cased) and a `pages/mod.rs` that declares each `pub mod`
   and re-exports each page component.
3. Delete `apps/playground/src/pages.rs`; update `main.rs` to declare
   `mod routes; mod pages;` and use `routes::{Route, Layout}` instead of
   local definitions.
4. Move the two `DatePickerPage` SSR tests into `pages/date_picker.rs`.
5. Verify: `cargo check --workspace`, `cargo fmt --all --check`,
   `cargo test -p adico-playground` (the two moved tests), a live
   `dx serve` pass over all 22 routes, `openspec validate
   refactor-playground-structure --strict`.

No rollback beyond `git revert`; this touches only `apps/playground`'s
internal module layout, nothing deployed or shared with consumers.
