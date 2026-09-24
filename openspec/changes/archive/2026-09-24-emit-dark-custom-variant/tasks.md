# Tasks

Design decisions referenced below are `D1`–`D3` in `design.md`.

## 1. Emit the variant (D1, D2)

- [x] 1.1 Emit `@custom-variant dark (&:is(.dark *))` from
  `theme_region()` in `packages/adico-cli/src/css.rs`, at the top of the
  region. Verify by asserting on the generated string, not only by eye
  — **done: emitted at the top of the region, before `@theme`, with a comment explaining the trap. Asserted on the generated string, and the alignment of the generated CSS comment was itself fixed after inspecting the emitted bytes — a first attempt left its continuation lines flush against the margin.**
- [x] 1.2 Verify `adico-cli`'s own tests still pass, and add or extend one so
  the generated region is asserted to contain the variant — this is a
  behavioral contract for every consumer, not an implementation detail

  — **done: `the_generated_region_binds_the_dark_variant_to_the_theme_class` asserts the declaration is present, that it precedes `@theme` (Tailwind must register a variant before generating utilities that use it), and that the `.dark` token block it pairs with is still emitted. `cargo test -p adico-cli` passes.**

## 2. Adopt it in apps/web (D3)

- [x] 2.1 Refresh `apps/web`'s theme region through the real CLI and verify the
  regenerated region contains the variant
  — **done via the real CLI (`adico add … --replace`); the regenerated region carries the variant.**
- [x] 2.2 Remove the hand-written declaration and its comment from
  `apps/web/tailwind.css`'s prefix, and update the README section that
  documents it
  — **done.** Exactly one declaration now exists in `apps/web/tailwind.css`, inside the generated region. The prefix comment and the README both claimed custom variants live in the prefix; both were corrected, including the README's worked example, so neither stands as a false explanation.
- [x] 2.3 Verify the compiled stylesheet still resolves `dark:` utilities
  against `:is(.dark *)` and **not** `@media (prefers-color-scheme: dark)` —
  measured in the compiled output, since that is the artifact that matters

  — **verified in the compiled artifact, which is the thing that matters: `dark\:hover\:bg-accent\/50` resolves under `&:is(.dark *)`, and the whole stylesheet contains **0** `prefers-color-scheme` rules.**

## 3. Validate

- [x] 3.1 Repo baseline: `cargo fmt --all --check`, `cargo check --locked
  --workspace`, clippy over the five core crates with `-D warnings`,
  core-crate tests, `cargo test --locked -p adico-web`,
  `openspec validate --all --strict`
  — **all pass:** fmt, workspace check, wasm32, clippy `-D warnings`, core tests, `adico-web` tests, `openspec validate --all --strict` 14/0.
- [x] 3.2 `adico css build` + `adico css check` from `apps/web`; registry gates
  (`registry validate`, `registry build --check`) since the CLI's embedded
  output may be affected
  — **pass:** `adico css build` + `adico css check` up to date; `registry validate` and `registry build --check` both pass.
- [x] 3.3 Verify the appearance mismatch case end to end: with the OS
  preferring one appearance and the app set to the other, a literal `dark:`
  utility follows the app, not the OS
  — **verified, after a first attempt that proved nothing.** Seeding the persisted theme in `localStorage` did not take (it loads asynchronously after mount), so that run only showed the class tracking the OS — which would have been a false negative. Tested the CSS semantics directly instead: a probe carrying `dark:text-emerald-500` is inert without the `dark` class and active with it, **identically under `colorScheme: 'light'` and `colorScheme: 'dark'`**. The utility follows the class, not the OS, which is the whole contract.
- [x] 3.4 Playwright: `playground-resizable-split`, `--project=mobile`,
  `--project=desktop-invariance`, plus the theme-related suites
  — **`playground-resizable-split` 3/3, `playground-enriched-demos` 4/4, `--project=mobile` 21/21, `--project=desktop-invariance` 14/14, plus a 32-check browser audit with 0 problems.**
  `mode-toggle.spec.ts` (3) and `theme-switcher.spec.ts` (2) fail — **pre-existing and structurally impossible to pass here**, not a regression. Both `goto("/")` and assert on `examples/basic-spa`'s markup: `theme-switcher` looks for `role="radio"` named "Rose palette", `mode-toggle` for `role="option"`. Measured on `apps/web`'s `/`: **0** `role="radio"` and **0** `role="option"` elements exist — adico's own `ThemeSwitcher` is a `Select` and its `ModeToggle` a `DropdownMenu`. A change that only adds one CSS line cannot alter element roles. Same shared-`baseURL` constraint recorded in the merge change's task 9.8.
- [x] 3.5 Report which checks ran and which did not, with reasons

  — **Ran and passed:** fmt, workspace check, wasm32, clippy, core-crate tests, `adico-web` tests, `registry validate`, `registry build --check`, `openspec validate --all --strict`, `adico css build` + `adico css check`, four Playwright suites, a 32-check browser audit, and a direct CSS-semantics test under both OS colour schemes.
  **Ran and failed, pre-existing:** `mode-toggle.spec.ts`, `theme-switcher.spec.ts` (see 3.4).
  **Did not run:** `xtask playground-controls check` (pre-existing breakage on paths deleted by `c55c500`) and bare `npm test` (same shared-`baseURL` constraint).
  **Left as found:** `component-props check` staleness — pre-existing and unrelated.