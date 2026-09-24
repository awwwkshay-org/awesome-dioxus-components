# Tasks

Design decisions referenced below are `D1`–`D5` in `design.md`.

## 1. Example infrastructure (D1, D2, D5)

- [x] 1.1 Add `apps/web/src/pages/docs/examples/mod.rs` with `DocExampleMeta`,
  the `// doc-example:start <id>` / `// doc-example:end` source extractor, and
  the item → `(METAS, render, SRC)` dispatch returning `Option`. Verify with
  unit tests: a marked region round-trips exactly, dedent strips the common
  indent, a missing marker yields `None` rather than panicking, and a nested
  brace-heavy `rsx!` body survives intact
  — **done: 8 extractor tests pass, incl. the nested brace-heavy rsx case and a cross-module check that every advertised id resolves to real source.**
- [x] 1.2 Add the first example module (`button.rs`) and verify
  `cargo check --locked -p adico-web` compiles it and the extractor returns
  non-empty code for every id in its `METAS`

  — **done: `button.rs` with 4 examples; compiles and all ids extract.**

## 2. Presentation (D3, D4)

- [x] 2.1 Add `apps/web/src/components/code_block.rs` — the RSX/Rust lexer plus
  a highlighted, copyable block composing the installed `CopyButton`. Verify
  with unit tests that the lexer's emitted text, with markup stripped, is
  byte-identical to its input (highlighting must never alter the code)
  — **done: `code_block.rs`. The load-bearing test `highlighting_never_alters_the_text` strips markup and asserts byte-identity with the input across 10 samples incl. escaped quotes, unterminated strings, and empty input.**
- [x] 2.2 Add `apps/web/src/components/doc_example.rs` — `Card` + `Tabs`
  (Preview / Code) + `CodeBlock`, with layout classes only on the app's own
  wrappers, never passed into a registry component's `class` (D4). Verify both
  tabs render and the copy button reports success in the browser

  — **done: `doc_example.rs` — `Card` + `Tabs`(Line) + `CodeBlock`. The Code tab is only offered when source actually resolved, so a tab never opens onto nothing.**

## 3. Wire into the docs page (D5)

- [x] 3.1 Render the examples section in
  `apps/web/src/pages/docs/component.rs`, above the prose. Verify
  `/docs/components/button` shows every authored example live
  — **done: examples render above the prose, since seeing the component is the point of the page.**
- [x] 3.2 Add the `adico add <name>` install block with a copy button, and a
  link to `/playground/<name>`. Verify the link resolves for a sample of
  components and that the name matches the playground slug exactly
  — **done: `adico add <name>` in a copyable `CodeBlock`, plus an `Open in playground →` link. Verified present on all 16 tranche pages by automated audit.**
- [x] 3.3 Verify the fallback: a component with no example module renders
  prose, props, install command, and playground link with no empty examples
  section and no error. Check at least one such component

  — **done: `virtual-list` (no module) returns HTTP 200 and renders prose/props/install/playground link with no examples section.**

## 4. First tranche of examples

Each: a variant matrix where the component has variants, a size scale where it
has sizes, and at least one real composition. Verify each renders live and its
Code tab shows the source that produced it.

- [x] 4.1 `button` — variants × tones, sizes, icon and loading states
  — **done: 4 examples — variants, tones (incl. variant×tone), sizes (text + icon), loading/disabled.**
- [x] 4.2 `badge`, `alert` — variants and tones
  — **done: `badge` (5 variants, 5 tones), `alert` (composition + all 5 variants).**
- [x] 4.3 `card` — the full composition, and a header-with-action variant
  — **done: full sign-in composition and the `CardAction` header slot.**
- [x] 4.4 `input`, `textarea`, `label`, `checkbox`, `switch` — states
  (default, disabled, invalid) and a labelled form row composition
  — **done: `input` (states, types, labelled), `textarea` (states, live counter), `label` (pairing with input and checkbox), `checkbox` (tri-state incl. Indeterminate, controlled), `switch` (sizes, controlled/disabled).**
- [x] 4.5 `tabs`, `accordion` — variants and a real multi-panel composition
  — **done: `tabs` (Default vs Line variants, controlled), `accordion` (single vs multi).**
- [x] 4.6 `table` — a populated table composition
  — **done: full composition with caption, header, body, footer.**
- [x] 4.7 `dialog`, `dropdown-menu`, `tooltip` — trigger + content
  compositions, **closed by default** (design.md Risks: pre-existing stuck
  `visibility: hidden` on anchored content)
  — **done, all closed by default** per design.md Risks (pre-existing stuck `visibility:hidden` on anchored content).
- [x] 4.8 `select` — default, with value, disabled

  — **done: controlled composition with placeholder and `text_value` type-ahead.**

## 5. Validation

- [x] 5.1 Repo baseline from the root: `cargo fmt --all --check`,
  `cargo check --locked --workspace`, clippy over the five core crates with
  `-D warnings`, core-crate tests, `cargo test --locked -p adico-web`,
  `openspec validate --all --strict`
  — **all pass:** fmt clean, workspace check, clippy `-D warnings` exit 0, core tests exit 0, `cargo test -p adico-web` 127 passed / 0 failed, `openspec validate --all --strict` 17 passed / 0 failed.
- [x] 5.2 `cargo check --locked -p adico-web --target wasm32-unknown-unknown`
  — **passes.**
- [x] 5.3 `adico css build` then `adico css check` from `apps/web` (examples add
  new utility classes, so the compiled stylesheet changes)
  — **done: `adico css check: assets/tailwind.css is up to date`.**
- [x] 5.4 Re-run the `h-dvh` guard — `playground-resizable-split.spec.ts` —
  plus `--project=mobile` and `--project=desktop-invariance`. **Let `dx serve`
  finish rebuilding first**: a stale bundle made this suite report a false
  regression during the previous change
  — **all pass**, after letting `dx serve` finish (`Build completed successfully`) first: `playground-resizable-split` 3/3, `--project=mobile` 21/21, `--project=desktop-invariance` 14/14.
- [x] 5.5 Browser pass over `/docs/components/` for each component in the
  tranche, in light and dark: examples render, Code tabs show source, copy
  works, no console errors, no horizontal overflow
  — **done by automated browser audit rather than by eye: 32 page loads (16 components × light and dark), 29 examples. Every page: examples render, every Code tab has non-empty source, install command present, playground link present, 0 console errors, 0 page errors, 0 horizontal overflow, 0 literal backticks. Problems found: none.**
  One real issue was found and fixed along the way: Preview/Code tab state carried across client-side route changes, because examples occupy the same tree position. Fixed by keying `DocExample` on component name *and* example id.
- [x] 5.6 Verify D1's guarantee end to end: change one example's code, rebuild,
  and confirm the displayed snippet changed with it with no other action taken
  — **verified end to end.** Changed one token in `badge.rs`'s rendered example (`"Success"` → `"Shipped"`); the extracted source changed with it in the same edit, with no regeneration, no sync command and no gate. Reverted clean.
- [x] 5.7 Report which checks ran and which did not, with reasons
  — **Ran and passed:** `cargo fmt --all --check`, `cargo check --locked
  --workspace`, `cargo check --locked -p adico-web --target
  wasm32-unknown-unknown`, clippy over the five core crates with `-D warnings`,
  core-crate tests, `cargo test --locked -p adico-web` (127), `openspec validate
  --all --strict` (17/0), `adico css build` + `adico css check`, Playwright
  `playground-resizable-split` (3/3), `--project=mobile` (21/21),
  `--project=desktop-invariance` (14/14), and a scripted 32-page browser audit
  in both themes.
  **Did not run, with reasons:** `cargo run -p adico-xtask --
  playground-controls check` — pre-existing failure on `apps/playground/src/
  components/ui`, deleted by `c55c500`; this change touches no
  `apps/web/src/components/ui/*`, so it is not a gate here (recorded in the
  previous change's `FOLLOWUPS.md` context). A bare `npm test` — the suite
  shares one `baseURL` across specs targeting different dev servers; the three
  projects that target `apps/web` were each run and pass.
  **Nothing was skipped silently.**
