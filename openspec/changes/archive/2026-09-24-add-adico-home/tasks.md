## 1. Workspace scaffold

- [x] 1.1 Add `"apps/home"` to root `Cargo.toml` `[workspace].members`, alphabetically before `"apps/docs"`, and verify `cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name'` lists `adico-home`
- [x] 1.2 Create `apps/home/Cargo.toml` modeled on `apps/playground/Cargo.toml`: `name = "adico-home"`, `version/edition/license.workspace = true`, one-line `description`, `publish = false`, explicit pinned `dioxus` dependency (`default-features = false`, features including `web`, `launch`, `macro`, `html`, `signals`, `hooks`, `asset`, `document`, `router`), `adico-primitives = { version = "=0.1.0", features = ["web"] }`, the pinned `web-sys` `Location` feature workaround, and the standard 4-entry `[features]` block — verify with `cargo check --locked -p adico-home` (expect a build error only for missing `src/main.rs`, not a manifest error)
- [x] 1.3 Create `apps/home/Dioxus.toml`: `name = "adico-home"`, `default_platform = "web"`, `[web.app] title = "adico"`
- [x] 1.4 Create a minimal placeholder `apps/home/src/main.rs` (bare `dioxus::launch` + empty `App`) so `adico init` in task 2.1 has a valid crate to operate on, and verify `cargo check --locked -p adico-home` succeeds

## 2. Real CLI installation

- [x] 2.1 Run `adico init` in `apps/home` and verify it produces `components.json`, `src/adico_lib/`, `src/components/`, and reserves `tailwind.css` — do not hand-author any of these
- [x] 2.2 Run `adico add <components>` for whatever the landing page shell and content need (expect something like `button`, `card`, `badge`, `navigation-menu`, `copy-button`, `mode-toggle`, `theme-switcher`, resolved transitively) and verify `apps/home/adico.lock` lists each installed item with a checksum
- [x] 2.3 Run `adico css build` in `apps/home` and verify `adico css check` reports `assets/tailwind.css is up to date`
- [x] 2.4 Copy `apps/playground/assets/web/*` (favicons, `site.webmanifest`) into `apps/home/assets/web/`, adjusting `site.webmanifest`'s name field for `adico-home`, and verify the files exist at their new paths

## 3. Routing and shell

- [x] 3.1 Create `apps/home/src/routes.rs` with a `Route` enum (`#[layout(Layout)] #[route("/")] Home {}`) and a `Layout` component composed from the installed registry components from task 2.2 (header/nav, mode-toggle, theme-switcher) — verify against the `adico-home-structure` spec's "Router definitions live in routes.rs" and "The landing page shell composes real registry components" requirements
- [x] 3.2 Create `apps/home/src/pages/index.rs` (aggregated by `src/pages/mod.rs`) with a hero section (name, tagline, install command rendered via the installed `CopyButton` component, CTA links to `/docs`, `/playground`, and the GitHub repo), a feature-highlights section (69 components, 67 primitives, source-owned/shadcn-style, dual `MIT OR Apache-2.0` license), and an install section listing the three real `v0.1.0` release channels (`brew install awwwkshay-org/tap/adico`, GitHub release binaries, `cargo install --git https://github.com/awwwkshay-org/awesome-dioxus-components --locked --package adico-cli`) — verify against the spec's "One page per file under pages/" requirement
- [x] 3.3 Rewrite `apps/home/src/main.rs` to hold only the entrypoint, asset consts (title/stylesheet/favicons from task 2.4), document head, `Router::<Route> {}` mount, and the CLI-managed `// adico:start`/`// adico:end` block — verify by re-reading the file against the spec's "main.rs is read for routing logic" scenario (no `#[derive(Routable)]`, no layout component defined there)

## 4. Documentation

- [x] 4.1 Update `apps/README.md` to mention the third app (`apps/home`) alongside `apps/docs` and `apps/playground`
- [x] 4.2 Update root `README.md`'s workspace table to add an `apps/home` row, and revise the "Project status" paragraph's "no hosted docs or registry site" line to reflect that the landing page now exists in-repo — without claiming it is hosted or deployed, since that remains a separate, later effort

## 5. Validation

- [x] 5.1 Run `cargo fmt --all --check` and verify it passes with no diff
- [x] 5.2 Run `cargo check --locked --workspace` and verify `adico-home` builds along with the rest of the workspace
- [x] 5.3 Run `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings` and verify it passes (this app doesn't add a new core crate, so this confirms nothing in `adico-cli`'s Cargo-editing path regressed)
- [x] 5.4 Run `cargo check --locked -p adico-home --target wasm32-unknown-unknown` and verify it compiles clean, matching how `examples/quickstart` was validated earlier this session
- [x] 5.5 Run `openspec validate --all --strict` and verify it passes, including the new `adico-home-structure` capability
- [x] 5.6 Manually run `dx serve` in `apps/home` and verify: the landing page renders, `mode-toggle`/`theme-switcher` actually switch the theme, and the install-command `CopyButton` copies the expected text to the clipboard
