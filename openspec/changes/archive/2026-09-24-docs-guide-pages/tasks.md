# Tasks

Design decisions referenced below are `D1`–`D5` in `design.md`.

## 1. Token derivation (D1)

- [x] 1.1 Add `apps/web/src/pages/docs/tokens.rs` — `include_str!` the project's
  `tailwind.css` and parse `:root { … }` and `.dark { … }` into
  `(name, light, dark)` triples behind a `OnceLock`. Verify with unit tests
  against the real committed stylesheet: the parse finds the known token count,
  every token has both a light and a dark value, a known token (`--background`)
  has the expected pair, and a malformed input yields an empty list rather than
  panicking

  — **done: 6 tests. The real committed stylesheet parses to 39 tokens, each with a light and a dark value; `--background` pins its expected pair; `--radius` (declared only in `:root`) is kept with matching values and correctly yields no swatch; malformed and unterminated input yields an empty list.**

## 2. Docs shell (D2, D3, D4)

- [x] 2.1 Add `apps/web/src/components/docs_nav.rs` — the guides + components
  nav list, composed from the installed `Sidebar` family, one shared rendering
  so a new guide route appears without a second edit
  — **done: `docs_nav.rs`, composed from the installed `Sidebar` family. `SidebarMenuButton` exposes no `onclick`, so it follows `nav.rs`'s existing precedent — a wrapping `div` plus a `min-w-0 truncate` label span — rather than adding an escape hatch to a registry component, which `adico-web-structure` #11 forbids.**
- [x] 2.2 Add `DocsLayout` to `routes.rs`, nested inside `SiteLayout` and
  sibling to `PlaygroundLayout`, with the sidebar `sticky` inside `main` (D3).
  Verify the playground's nesting and the two shell-free harness routes keep
  their exact current position
  — **done: `DocsLayout` nested inside `SiteLayout`, sibling to `PlaygroundLayout`. The playground subtree and both shell-free harness routes keep their exact prior nesting.**
- [x] 2.3 Verify no document-level scrollbar appears on any route
  (`documentElement.scrollWidth/scrollHeight` vs `clientWidth/clientHeight`),
  and that `main.rs` and `main`'s classes are unchanged (`git diff` is empty for
  `main.rs`)

  — **verified, not assumed: scripted audit across 40 page loads found `documentElement.scrollHeight > clientHeight` false everywhere — the app still scrolls inside `main`, never the document. `git diff` for `main.rs` shows only the Change-1 font-asset additions; `main`'s classes are untouched.**

## 3. Guide pages

Each under `apps/web/src/pages/docs/guides/`, one file per route, reusing
`Prose` and `CodeBlock`.

- [x] 3.1 `/docs/installation` — `adico init`, `adico add`, `components.json`,
  and what the CLI does *not* wire automatically. Verify every command shown is
  one the CLI actually accepts
  — **done.**
- [x] 3.2 `/docs/tailwind` — the per-project pipeline: `@import "tailwindcss"`,
  `@source`, the `adico:theme:start`/`end` region and why it must never be
  hand-edited, `document::Stylesheet { href: asset!(…) }` plus the `document`
  feature, `dx serve` auto-compilation, and `adico css build` / `adico css
  check`. Verify against `apps/web`'s own real files
  — **done, written against `apps/web`'s own real files.**
- [x] 3.3 `/docs/theming` — the derived token table with live swatches for both
  appearances (D1), the `--radius` scale, `Tone`, the palette presets, and an
  embedded live `ThemeBuilder` (D5). Verify the token count rendered equals the
  count in `tailwind.css`
  — **done: 39 tokens rendered, matching the stylesheet, with light and dark swatches side by side; plus the `--radius` scale with live specimens, `Tone`, and an embedded live `ThemeBuilder`.**
- [x] 3.4 `/docs/dark-mode` — the `.dark` class vs `prefers-color-scheme`, why
  the app declares `@custom-variant dark`, `ModeToggle`,
  `use_persisted_theme_mode`, and the known first-paint flash. Verify the
  described behavior against what the compiled stylesheet actually does
  — **done. Its central claim is checked against the compiled artifact rather than asserted: with `@custom-variant dark` declared, `dark:hover:bg-accent/50` compiles to `:is(.dark *)`, not `@media (prefers-color-scheme: dark)`.**
- [x] 3.5 `/docs/typography` — the type scale with live specimens, the font
  tokens, and how to swap the typeface. Verify each documented step renders a
  specimen in that step
  — **done: every documented step renders its own live specimen.**
- [x] 3.6 `/docs/spacing` — the spacing rhythm and the `Radius` enum, including
  its deliberate off-by-one (`Radius::Lg` → `rounded-xl`). Verify the documented
  mapping against `adico_lib::variants`
  — **done: the `Radius` table renders a live box per arm beside the class it actually emits, so the deliberate off-by-one (`Radius::Lg` → `rounded-xl`) is visible rather than merely stated.**
- [x] 3.7 Add the guides section to `/docs` (D4) and verify every guide route is
  linked from it

  — **done, and verified: the audit asserts every one of the six guide hrefs is linked from `/docs`; none were missing.**

## 4. Validation

- [x] 4.1 Repo baseline: `cargo fmt --all --check`, `cargo check --locked
  --workspace`, clippy over the five core crates with `-D warnings`, core-crate
  tests, `cargo test --locked -p adico-web`, `openspec validate --all --strict`
  — **all pass:** fmt clean, workspace check, clippy `-D warnings` exit 0, core tests exit 0, `cargo test -p adico-web` 142 passed / 0 failed, `openspec validate --all --strict` 18 passed / 0 failed.
- [x] 4.2 `cargo check --locked -p adico-web --target wasm32-unknown-unknown`
  — **passes.**
- [x] 4.3 `adico css build` then `adico css check` from `apps/web`
  — **done: `assets/tailwind.css is up to date`.**
- [x] 4.4 **The layout regression surface.** Let `dx serve` finish rebuilding,
  then run `playground-resizable-split.spec.ts`, `--project=mobile`, and
  `--project=desktop-invariance`. A second nested layout is exactly the kind of
  change that broke the 70/30 split before
  — **all pass**, after waiting for `Build completed successfully`: `playground-resizable-split` 3/3, `--project=mobile` 21/21, `--project=desktop-invariance` 14/14. The nested layout did not disturb the playground geometry.
- [x] 4.5 Scripted browser audit of all six guide routes plus `/docs`, `/`, and
  a playground route, in light and dark: no console errors, no document-level
  scroll, no horizontal overflow, every guide reachable from `/docs`
  — **done: 40 checks (10 routes × light/dark × 1440px and 390px). Zero console errors, zero page errors, zero document-level horizontal or vertical scroll, every page has an `h1`, no literal backticks or `**` outside code blocks, all six guides linked from `/docs`.**
  Two real defects were caught by this audit and fixed:
  (a) **`cargo fmt` silently corrupted displayed code.** It re-indents the continuation lines of a multi-line raw string nested inside `rsx!`, so snippets gained leading whitespace on every line but the first. Fixed by `normalize_indent` in `code_block.rs`, with tests covering formatter-added indent, preserved relative nesting, and already-flush text.
  (b) **Headings and alert titles bypassed `Prose`**, so `` `dark:` `` rendered as a literal backtick in a section title. Fixed by adding `ProseInline` (same segmenter, no wrapping `<p>`).
  One audit *false positive* was corrected rather than 'fixed' in the source: backticks inside a `<pre>` are legitimate code content (a CSS comment naming `adico add`), so the check now excludes code blocks.
- [x] 4.6 Report which checks ran and which did not, with reasons

  — **Ran and passed:** `cargo fmt --all --check`, `cargo check --locked --workspace`, `cargo check --locked -p adico-web --target wasm32-unknown-unknown`, clippy over the five core crates with `-D warnings`, core-crate tests, `cargo test --locked -p adico-web` (142), `openspec validate --all --strict` (18/0), `adico css build` + `adico css check`, Playwright `playground-resizable-split` (3/3), `--project=mobile` (21/21), `--project=desktop-invariance` (14/14), and a 40-check scripted browser audit.
  **Did not run, with reasons:** `cargo run -p adico-xtask -- playground-controls check` — pre-existing failure on `apps/playground/src/components/ui`, deleted by `c55c500`; this change touches no `apps/web/src/components/ui/*`. A bare `npm test` — the suite shares one `baseURL` across specs targeting different dev servers; the three projects targeting `apps/web` were each run and pass.
  **Note:** interactive browser screenshots were taken via Playwright rather than the Chrome extension, which disconnected mid-session. The scripted audit is the stronger evidence and covers more ground.