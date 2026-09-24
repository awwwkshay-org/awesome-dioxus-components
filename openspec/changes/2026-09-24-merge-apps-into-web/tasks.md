## 1. Route-nesting prerequisite check

- [ ] 1.1 In a scratch branch or throwaway crate, confirm dioxus-router renders nested `#[layout(...)]` scopes as nested `Outlet`s (not parallel siblings) — verify by rendering a two-level nested layout with a portalled popup (e.g. a Dialog) inside the inner scope and confirming it positions correctly. If it produces parallel Outlets instead, record the flat-`SiteLayout` fallback from design.md's Decisions section as the chosen route-tree shape before proceeding to section 3.

## 2. Crate move and scaffold

- [ ] 2.1 `git mv apps/playground apps/web` and verify with `git status` that the move is tracked as a rename, not a delete+add
- [ ] 2.2 Update `apps/web/Cargo.toml`: `name = "adico-web"`, add `serde`/`serde_json` as explicit pinned entries (docs currently inherits them via `.workspace = true`), drop the `server` feature entry — verify `cargo check --locked -p adico-web` still succeeds (expect only errors from missing home/docs modules, addressed in section 3)
- [ ] 2.3 Update `apps/web/Dioxus.toml`: `name = "adico-web"`, `[web.app] title = "adico"`
- [ ] 2.4 Update root `Cargo.toml` `[workspace].members`: remove `"apps/docs"` and `"apps/home"`, rename the `"apps/playground"` entry to `"apps/web"` — verify `cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name'` lists `adico-web` and no longer lists `adico-home`/`adico-docs`/`adico-playground`
- [ ] 2.5 Delete `apps/home/` and `apps/docs/` (their content is folded into `apps/web` in section 3) — verify `git status` shows both paths removed

## 3. Fold in home and docs content

- [ ] 3.1 Move `apps/home/src/pages/index.rs` to `apps/web/src/pages/index.rs`, `apps/home/src/components/cta_link.rs` to `apps/web/src/components/cta_link.rs`, register both in their respective `mod.rs` files — verify `cargo check --locked -p adico-web` reports only route-wiring errors (addressed in section 4), not missing-file errors
- [ ] 3.2 Create `apps/web/src/pages/playground/` and move every existing `apps/web/src/pages/*.rs` demo page file into it (except `index.rs`, which becomes the playground index at `pages/playground/index.rs`), updating `pages/mod.rs`/`pages/playground/mod.rs` accordingly — verify `cargo check --locked -p adico-web` compiles once routes.rs is updated in section 4
- [ ] 3.3 Create `apps/web/src/pages/docs/index.rs` and `apps/web/src/pages/docs/component.rs`, porting `apps/docs/src/main.rs`'s two page bodies onto Tailwind utility classes and installed `Card`/`Table`/`Badge` registry components — remove all `DOCS_STYLE` hardcoded CSS — verify by rendering both pages via `dx serve` in light and dark mode and confirming no forced dark background remains
- [ ] 3.4 Update the two `include_str!` calls moved into `apps/web/src/pages/docs/*.rs` to account for the one-directory-deeper nesting (`../../../../registry/registry.json`, `../../../../statics/component_props.json`) — verify `cargo check --locked -p adico-web` succeeds and the rendered docs pages show real component/prop data, not an empty list

## 4. Route tree and layouts

- [ ] 4.1 Rewrite `apps/web/src/routes.rs`'s `Route` enum per design.md's Decisions section: `SiteLayout` wraps `/` (`Home`), `/docs`, `/docs/components/:name`, and a `PlaygroundLayout`-wrapped `/playground` + `/playground/<segment>` subtree (×70, one variant per moved page); `/responsive/flow` and `/responsive/overlay?:case` stay outside every layout — verify `cargo check --locked -p adico-web` succeeds with zero unused-route or unreachable-page warnings
- [ ] 4.2 Compose `SiteLayout` from installed registry components (header/nav reusing home's `NavigationMenu`/`ModeToggle`/`ThemeSwitcher` composition, updated to link `/`, `/docs`, `/playground`) — verify the header renders identically to `apps/home`'s pre-merge shell, with working links
- [ ] 4.3 Rename playground's existing `Layout` component to `PlaygroundLayout`, update its internal route references (nav column links, active-route matching) to the `/playground/*` prefix — verify `cargo check --locked -p adico-web --target wasm32-unknown-unknown` compiles clean
- [ ] 4.4 Update `nav_items()` to emit `/playground/<segment>` hrefs — verify by rendering `/playground` via `dx serve` and confirming every nav entry navigates to a working `/playground/*` route

## 5. Fix incidental defects found during research

- [ ] 5.1 In `apps/web/adico.lock`, correct the stale checksum for `@adico/theme-switcher` to match the installed file's real sha256 — verify `adico add --check` (or the equivalent lock-verification command) reports no mismatch
- [ ] 5.2 Neutralize the dead demo hrefs in `apps/web/src/pages/playground/breadcrumb.rs` and `apps/web/src/pages/playground/responsive_flow.rs` (currently `/components`, `/components/registry`) to `#` — verify by grepping `apps/web/src/pages/playground/` for `"/components"` and confirming zero matches

## 6. CSS pipeline

- [ ] 6.1 Run `adico css build` from `apps/web`, verify `adico css check` reports `assets/tailwind.css is up to date`, and confirm every Tailwind class referenced across the landing, docs, and playground page trees is present in the compiled output

## 7. Playwright suite

- [ ] 7.1 Add the `/playground` prefix to the 27 `goto()` call sites in `playground-drag-and-drop-list.spec.ts`, `playground-resizable-split.spec.ts`, `playground-time-picker.spec.ts`, and `playground-enriched-demos.spec.ts` — leave every `goto("/")` targeting `examples/basic-spa`/`basic-ssr`/`tests/installation/*` fixtures and every `/responsive/*` goto untouched — verify by grepping `tests/playwright/*.spec.ts` for `goto("/` outside those four files and confirming no unintended change
- [ ] 7.2 Update `tests/playwright/README.md`'s playground instructions from `dx serve` in `apps/playground` to `apps/web`

## 8. Documentation

- [ ] 8.1 Update `apps/README.md` to describe one app (`apps/web`) instead of three
- [ ] 8.2 Update root `README.md`'s workspace table (three app rows → one `apps/web` row) and the "Project status" paragraph's app-path references
- [ ] 8.3 Update `docs/ci-cd.md`, `docs/architecture.md`, `docs/development.md`, `docs/validation.md`, and `CLAUDE.md`'s "Project overview" bullet — verify with `grep -rn "apps/docs\|apps/home\|apps/playground" --include=*.md .` that no reference to the three old paths remains outside `openspec/changes/archive/` and `openspec/specs/adico-*-structure` history

## 9. Validation

- [ ] 9.1 Run `cargo fmt --all --check` and verify it passes with no diff
- [ ] 9.2 Run `cargo check --locked --workspace` and verify `adico-web` builds along with the rest of the workspace, with no remaining reference to `adico-home`/`adico-docs`/`adico-playground`
- [ ] 9.3 Run `cargo check --locked -p adico-web --target wasm32-unknown-unknown` and verify it compiles clean
- [ ] 9.4 Run `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings` and verify it passes
- [ ] 9.5 Run `cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask` and verify it passes
- [ ] 9.6 Run `openspec validate --all --strict` and verify it passes, including the new `adico-web-structure` capability and the removal of `adico-playground-structure`/`adico-home-structure`
- [ ] 9.7 Manually run `dx serve` in `apps/web` and verify: `/` renders with working `/docs` and `/playground` links; `/docs` and a `/docs/components/:name` page render correctly in both light and dark mode; a `/playground/<x>` page with a portalled overlay (dialog/popover/select) opens and positions correctly; `/responsive/flow` renders with no site chrome
- [ ] 9.8 With `dx serve` still running, run `ADICO_PLAYWRIGHT_BASE_URL=http://localhost:3000 npm test` from `tests/playwright` and verify the full suite passes
