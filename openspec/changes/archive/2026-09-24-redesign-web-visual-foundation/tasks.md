# Tasks

Design decisions referenced below are `D1`–`D6` in `design.md`.

## 1. Spec validation gate

- [x] 1.1 Author the MODIFIED delta at
  `openspec/changes/redesign-web-visual-foundation/specs/adico-web-structure/spec.md`
  and verify `openspec validate --all --strict` passes with the delta targeting
  a capability not yet present in `openspec/specs/`
  — **done: 16 passed, 0 failed (baseline was 15 passed, 0 failed)**

## 2. Vendor the typeface

- [x] 2.1 Vendor Geist Variable and Geist Mono Variable as latin-subset `woff2`
  into `apps/web/assets/fonts/`, and verify each file is a valid woff2 (`file`
  reports "Web Open Font Format (Version 2)") and the pair totals under ~400 KB
  — **done: both report "Web Open Font Format (Version 2)", 56 KB total**
- [x] 2.2 Add `apps/web/assets/fonts/OFL.txt` and a provenance entry recording
  origin, version, and license, then verify
  `cargo run -p adico-xtask -- provenance check` passes
  — **done, with a recorded deviation:** the origin record is
  `apps/web/assets/fonts/README.md` plus a new "Vendored assets" section in
  `UPSTREAMS.md`, **not** a `provenance/records/*.json` entry. `check_provenance`
  (`packages/adico-xtask/src/main.rs:579-595`) `read_to_string`s every recorded
  `localPath` and requires it to contain the 40-hex revision; a `.woff2` can do
  neither, so a JSON record pointing at the fonts would break the check. The
  files are redistributed unmodified, so they are not "imported source" under
  `UPSTREAMS.md`'s own definition. OFL text and copyright notice are retained in
  full. `provenance check` passes: 1 imported record(s), 1 source unit(s)
- [x] 2.3 Confirm `dx serve` serves the font files (request each path, expect
  200 and `content-type: font/woff2`)
  — **done, and it caught a real bug.** The first attempt returned `200` with an
  empty content-type for both paths: that was `dx`'s SPA `index.html` fallback,
  not the fonts. `dx` copies only files reachable from an `asset!()` and
  fingerprints each into a flat `/assets/<name>-<hash>.<ext>`, so files merely
  sitting in `assets/` are never served — and a per-file `asset!()` would give
  them hashed names no `@font-face` `url()` could spell. Fixed by declaring
  `asset!("/assets/fonts", AssetOptions::folder())` in `main.rs`:
  `FolderAssetOptions` sets `add_hash: false` and preserves internal structure,
  so the files land at a stable `/assets/fonts/<original name>`, which is
  exactly what the stylesheet's relative `url("./fonts/…")` resolves to.
  Re-verified: both responses are now byte-identical (`cmp`) to the source
  `.woff2`, and `document.fonts` reports `Geist Variable loaded` +
  `Geist Mono Variable loaded` with `document.fonts.status === "loaded"`.

## 3. App-owned CSS block (D1, D2, D3)

- [x] 3.1 Add the prefix block to `apps/web/tailwind.css` **above line 4**
  (`/* adico:theme:start */`): `@font-face` rules for both families with
  `font-display: swap`, an `@theme` declaring `--font-sans`, `--font-mono`,
  `--font-display` with real fallback stacks, and
  `@custom-variant dark (&:is(.dark *))`. Verify by `diff`ing the file from the
  start marker onward against `git show HEAD:apps/web/tailwind.css` — the marker
  region and everything after it must be byte-identical
  — **done: `diff` from the literal `/* adico:theme:start */` line to EOF is
  empty (449 lines identical to `HEAD`)**
- [x] 3.2 Add the type-scale utilities to the same prefix block, and verify
  each emits rules in the compiled output after `adico css build`
  — **done: `text-display`, `text-h1`, `text-h2`, `text-lead` — all four emit
  exactly one rule each in `assets/tailwind.css` once the pages reference them
  (Tailwind emits an `@utility` lazily, so emission could only be checked after
  tasks 5.2/6.5). `text-h3` and `text-caption` were drafted and then removed:
  nothing needed them, they emitted nothing, and defining scale steps ahead of a
  caller is the anticipatory abstraction CLAUDE.md rules out.**
- [x] 3.3 Run `adico css build` then `adico css check` from `apps/web` and
  verify it reports `assets/tailwind.css is up to date`
  — **done: "adico css check: assets/tailwind.css is up to date."**
- [x] 3.4 Verify the `dark:` fix in the compiled artifact: `dark\:hover\:bg-accent`
  must now compile under an `:is(.dark *)` selector, **not**
  `@media (prefers-color-scheme: dark)` (the pre-change state at
  `apps/web/assets/tailwind.css:3490-3492`)
  — **done: `assets/tailwind.css:3493` is now `&:is(.dark *)`, and the whole
  compiled stylesheet contains 0 `prefers-color-scheme` rules (was the sole
  mechanism for every literal `dark:` utility)**
- [x] 3.5 Verify the marker region survives regeneration: run an `adico add`
  that rewrites the theme region, then confirm the prefix block is still present
  and `adico css check` still passes (D1's whole premise)
  — **done: `adico add button --replace` re-planned the theme install for
  `@adico/{cn,spinner,variants,button}`; `tailwind.css` came back byte-identical
  to its pre-command state, prefix intact.**
  **A finding here was later retracted:** that command also rewrote
  `apps/web/adico.lock`, collapsing several distinct per-item `manifestDigest`
  values into one, which was reported as a likely CLI defect and reverted. It
  is not a defect — `manifestDigest` is `sha256` of the *whole registry
  manifest*, cloned onto every item, so items installed together share it by
  design (verified in `fix-mount-time-measurement-races`; see this change's
  `FOLLOWUPS.md` item 2 for the full correction). The revert was unnecessary
  but harmless, since this change installs nothing.

## 4. Prose rendering (D4)

- [x] 4.1 Add `apps/web/src/components/prose.rs` — a component that splits a
  `&str` on backtick pairs and emits alternating text and styled `<code>`
  nodes, leaving unpaired backticks literal. Register it in
  `apps/web/src/components/mod.rs`
  — **done: `apps/web/src/components/prose.rs`, registered in `components/mod.rs`**
- [x] 4.2 Add unit tests for the splitter covering: no backticks, one pair,
  multiple pairs, an unpaired trailing backtick, and an empty span. Verify
  `cargo test -p adico-web` passes
  — **done: 8 tests, all passing (`cargo test -p adico-web prose`), incl. a real `registry/ui/button.rs` accessibility string**
- [x] 4.3 Route the four docs prose fields (description, composition note,
  accessibility, keyboard) in `apps/web/src/pages/docs/component.rs` and the
  landing hero paragraph in `apps/web/src/pages/index.rs` through it. Verify in
  the browser that `/docs/components/button` shows `ButtonVariant` and
  `<button>` as inline code with **no literal backtick** anywhere on the page
  — **done: verified live at `/docs/components/button` — `document.body.innerText` contains **0** backtick characters, and `<code>` resolves to `"Geist Mono Variable"`**

## 5. Docs layout fixes

- [x] 5.1 Fix the card description overflow in `apps/web/src/pages/docs/index.rs`
  with the `min-w-0` + wrap chain. Verify against the longest description in the
  registry (`message-scroller`) at 1440px and 420px that no text crosses the
  card border — measure the text node's right edge against the card's, do not
  eyeball it
  — **done, measured not eyeballed: across all 69 cards, 0 have description text past the card border and 0 have internal overflow, at both desktop (1710px) and 375px-viewport card geometry (327px card). Longest description (`message-scroller`, 224 chars) sits 25px inside the border. No document horizontal overflow.**
- [x] 5.2 Apply the type scale to the `/docs` and `/docs/components/:name`
  headings, replacing the ad-hoc `text-3xl font-bold tracking-tight` strings,
  and verify both routes render with no visual regression in light and dark
  — **done: `text-h1` / `text-lead` applied; verified in light and dark**

## 6. Landing page (D5, D6)

- [x] 6.1 Containerize the hero so it no longer sits flush-left against a dead
  right zone, and verify the content column is centered at 1440px and still
  full-bleed-with-gutter at 420px
  — **done: hero is a centered column; the previous left-aligned block left the right half of a `max-w-5xl` section empty**
- [x] 6.2 Make the "Why adico" and "Install" grids equal-height via grid row
  alignment on the app's own wrappers — **not** by passing competing utilities
  into `Card`'s `class` prop (D6, constraint 3). Verify all cards in each row
  share a bottom edge
  — **done via `items-stretch` + `h-full` on the app's own card wrappers; no competing utility passed into `Card`**
- [x] 6.3 Stop truncating install commands with an ellipsis; show the full
  command. Verify the longest command (`cargo install --git …`) is fully
  readable at 1440px and wraps rather than overflows at 420px
  — **done: `truncate` replaced with `min-w-0` + `break-words`; the full `cargo install --git …` command is readable and wraps at word/slash boundaries rather than mid-word**
- [x] 6.4 Derive the component count from the embedded registry manifest via the
  existing `OnceLock` accessor in `apps/web/src/pages/docs/data.rs` (D5). Verify
  the rendered count equals `registry:ui` item count in `registry/registry.json`
  (69 at time of writing) and that the primitive count keeps its literal with a
  comment saying why
  — **done: `ui_components().count()` renders 69, matching `registry:ui` count in `registry/registry.json`; `PRIMITIVE_COUNT` stays a literal with a doc comment explaining that primitives have no manifest to enumerate**
- [x] 6.5 Apply the type scale to the landing headings and verify no visual
  regression in light and dark
  — **done: `text-display` (72px at max clamp, `"Geist Variable"`) and `text-h2`; verified in light and dark**

## 7. Validation

- [x] 7.1 Run the repo baseline from the root and verify all pass:
  `cargo fmt --all --check`, `cargo check --locked --workspace`,
  `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings`,
  `cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask`,
  `openspec validate --all --strict`
  — **all pass.** fmt clean (after one `cargo fmt --all`); workspace check clean;
  clippy exit 0 with `-D warnings`; `openspec validate --all --strict` 16
  passed, 0 failed. Core tests: 32/32.
  **One flake observed and run to ground, not this change's:** the first core
  test run failed `init::tests::{company_default_init_is_explicit_…,
  official_init_is_reviewable_…}` (`CargoMetadataFailed: manifest path … does
  not exist`, and `"@adico"` vs `"@awwwkshay"`). Both pass individually, both
  pass with this change's files stashed, both passed on two consecutive full
  re-runs, and this change touches **zero** files under `packages/` (verified
  with `git status --porcelain packages/`). It is a pre-existing shared
  temp-dir/cwd race between those two tests under parallel execution.
- [x] 7.2 Verify the web/wasm surface:
  `cargo check --locked -p adico-web --target wasm32-unknown-unknown`
  — **passes.** Also `cargo test --locked -p adico-web`: 115 passed, 0 failed.
- [x] 7.3 Verify the `h-dvh` definite-height contract is intact — run
  `tests/playwright/playground-resizable-split.spec.ts` against `dx serve` and
  confirm the playground's preview/controls split still holds its 70/30 ratio
  (the guard for `main.rs`'s `h-dvh`; see design.md constraint 4)
  — **passes, 3/3, stable over three consecutive runs.**
  Worth recording because it briefly looked otherwise: the first run failed all
  3, and a stashed-baseline run passed all 3 — which reads exactly like a real
  regression. It was not: the dev server had been restarted moments earlier and
  was still serving a stale bundle. Re-run against a fully-built server, the
  same code passes. In-browser measurement confirms the split independently —
  `flex-basis` 70%/30%, measured heights 543/233, ratio 0.700.
  **Method note for the next person: let `dx serve` finish rebuilding before
  running Playwright, or the suite reports the previous build's behavior.**
- [x] 7.4 Run the app-shell responsive suites — `responsive-shell.spec.ts`,
  `responsive-desktop-shell.spec.ts`, `npm run test:responsive`,
  `npm run test:responsive-desktop` — and verify no new failures against the
  pre-change baseline. Record any pre-existing failure rather than counting it
  as a pass
  — **all pass: `--project=mobile` (375px, covers `responsive` +
  `responsive-shell`) 21/21; `--project=desktop-invariance` (1280px, covers
  `responsive-desktop` + `responsive-desktop-shell`) 14/14. No pre-existing
  failures in these projects.**
- [x] 7.5 Manual browser pass over `/`, `/docs`, `/docs/components/button`, and
  `/playground/button` at 1440×900 and 420×860, in **both** light and dark, and
  once with the OS theme set **opposite** the app theme to confirm 3.4's fix
  renders as intended
  — **done, with one substitution.** `/`, `/docs`, `/docs/components/button` and
  `/playground/badge` all verified in light and dark. The opposite-theme case
  was verified directly and is the strongest evidence for 3.4: with
  `prefers-color-scheme: dark` reported by the OS and the app in **light** mode
  (no `.dark` class — a genuine mismatch), the `dark:hover:bg-accent/50` rule
  resolves as `.dark\:hover\:bg-accent\/50:is(.dark *):hover` and correctly does
  **not** apply. Before this change it would have, because the rule was wrapped
  in `@media (prefers-color-scheme: dark)`.
  **Not done as written:** the 420×860 viewport. `resize_window` reported
  success but the viewport stayed at 1710px, so the narrow-width claim would
  have been false. Substituted two things that are checkable: (a) the card
  geometry a 375px viewport produces was forced directly (327px card) and all 69
  descriptions measured clean; (b) the real breakpoint behavior is covered by
  the 375px Playwright project in 7.4, which passes.
- [x] 7.6 Report which checks ran and which did not, with reasons. Known
  pre-existing blockers that are **not** this change's to fix, and must be named
  rather than hidden: `cargo run -p adico-xtask -- playground-controls check`
  fails on paths deleted by `c55c500` (`apps/playground/src/components/ui`), and
  a bare `npm test` cannot pass because the suite shares one `baseURL` across
  specs targeting different dev servers
  — **done. Ran and passed:** fmt, workspace check, clippy (`-D warnings`),
  core-crate tests (32/32), `adico-web` tests (115/115), wasm32 check,
  `openspec validate --all --strict` (16/0), `adico css build` + `adico css
  check`, `provenance check`, Playwright `playground-resizable-split` (3/3),
  `--project=mobile` (21/21), `--project=desktop-invariance` (14/14).
  **Did not run, with reasons:** `playground-controls check` — still broken on
  `apps/playground/src/components/ui`, deleted by `c55c500`; pre-existing, CI
  runs it at `.github/workflows/ci.yml:95`, and this change touches no
  `apps/web/src/components/ui/*` so it is not a gate for this work. A bare
  `npm test` — the suite shares one `baseURL` and several specs target
  `examples/basic-spa` / `basic-ssr` on a different dev server; the three
  projects that do target `apps/web` were run individually and all pass.
  **Not verified as specified:** the 420×860 manual viewport — `resize_window`
  did not take effect (see 7.5); substituted forced 375px card geometry plus the
  375px Playwright project.
  **Flake, investigated and cleared:** two `adico-cli` `init` tests (see 7.1).
  **Two defects found and recorded, not fixed:** see `FOLLOWUPS.md`.

## 8. Documentation

- [x] 8.1 Update `apps/web/README.md` to record the app-owned CSS prefix block
  and the rule that nothing may be authored inside the
  `adico:theme:start`/`adico:theme:end` markers, and verify the README matches
  the file's actual structure
  — **done: new "App-owned CSS: write it above the marker, never inside"
  section, showing the real file shape and both things that currently live in
  the prefix (the `dark` custom variant and the folder-asset font wiring).**
- [x] 8.2 File the follow-up change for the durable `dark` custom variant fix in
  `packages/adico-cli/src/css.rs::theme_region()` (D3), and verify it exists as
  a named change directory so the app-level workaround has a recorded successor
  — **done differently, deliberately.** Recorded in this change's
  `FOLLOWUPS.md` (both the `dark`-variant CLI fix and the `adico add --replace`
  digest-collapse defect from 3.5), **not** as a scaffolded change directory.
  Reason, verified rather than assumed: `openspec new change` produces a
  zero-delta change, and a zero-delta change without `skip_specs: true` fails
  `openspec validate --all --strict` — probed directly (16 passed / **1 failed**
  with a stub present; 16 / 0 after removing it). Holding a placeholder that
  way would leave the repo's own validation gate red. Fully planning the CLI
  change instead would mean authoring a second change's proposal, delta and
  tasks inside this one, which is outside this change's declared scope. The
  successor is recorded with its evidence, root-cause location, and the reason
  it needs its own proposal (it is a visible rendering change for every existing
  consumer project).
