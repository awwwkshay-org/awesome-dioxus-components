# Tasks

Design decisions referenced below are `D1`–`D2` in `design.md`.

## 1. Reproduce

- [x] 1.1 Against a real `dx build` served statically (not `dx serve`, so the
  result cannot be blamed on the dev overlay), confirm
  `/docs/components/checkbox` has `documentElement.scrollHeight` greater than
  the viewport while `/docs/components/switch` does not

  — **reproduced against a real `dx build` served statically:** `/docs/components/checkbox` 977 vs a 900 viewport and genuinely scrollable; `/docs/components/switch` and `/docs/components/button` both exactly 900.

## 2. Fix the primitive (D1)

- [x] 2.1 Zero the hidden input's box in
  `packages/adico-primitives/src/checkbox.rs`, matching `switch.rs`. Verify
  the input is still a real `input[type=checkbox]` carrying its name, value
  and checked state
  — **done, on the second attempt.** The first attempt added `width: "0", height: "0"` as individual attributes and **silently did nothing**: `width`/`height` are real HTML attributes on `<input>`, so Dioxus emitted them as attributes (inert on a checkbox) rather than as CSS — unlike `position`/`opacity`/`transform`, which it does map to style. Caught by re-measuring rather than trusting the edit: the input was still 13×13. Fixed by moving the whole declaration into one `style` string, which is exactly why `switch.rs` uses one. The input remains `input[type=checkbox]` with its name, value and checked state intact.
- [x] 2.2 Remove the app-level mitigation from `apps/web/src/pages/index.rs`
  (D2), including its now-obsolete comment

  — **done**, including its comment, which would otherwise have stood as a false explanation once the cause was gone.

## 3. Verify

- [x] 3.1 Re-measure the same routes on a fresh `dx build`: no route's
  document scrolls, and `/docs/components/checkbox` matches the viewport
  exactly
  — **done: every route measured at exactly 900 against a 900 viewport, none scrollable — `/` and `/docs/components/checkbox` included. The hidden inputs measure 0×0.**
- [x] 3.2 Verify the checkbox still works end to end — toggling updates the
  hidden input's checked state — rather than only that the leak is gone
  — **verified end to end, not just by absence of the leak.** On the `Controlled` example: `aria-checked` false→true on click, its label tracked `Unchecked`→`Checked`, and the hidden native input's `.checked` followed. All 5 hidden inputs on the page are 0×0.
- [x] 3.3 Repo baseline: `cargo fmt --all --check`, `cargo check --locked
  --workspace`, clippy over the five core crates with `-D warnings`,
  core-crate tests (these include the primitive's own), `cargo test --locked
  -p adico-web`, `openspec validate --all --strict`
  — **all pass:** fmt, workspace check, clippy `-D warnings`, core-crate tests (these include `adico-primitives`' own), `cargo test -p adico-web`, `openspec validate --all --strict` 14/0.
- [x] 3.4 `cargo check --locked -p adico-web --target wasm32-unknown-unknown`;
  registry gates if any registry source changed (`registry validate`,
  `registry build --check`)
  — **pass.** `registry validate` and `registry build --check` both pass and the registry is untouched — as design.md predicted, the hidden input lives in the primitive, not in `registry/ui/checkbox.rs`, so no registry source, lockfile or installed copy changed.
- [x] 3.5 Playwright: the suites that exercise checkbox and form controls, plus
  `playground-resizable-split`, `--project=mobile`, `--project=desktop-invariance`
  — **`playground-resizable-split` 3/3, `playground-enriched-demos` 4/4, `wave2-state` 1/1, `--project=mobile` 21/21, `--project=desktop-invariance` 14/14.**
  `wave3.spec.ts` fails 7/7 — **pre-existing and structurally unrelated**, not a regression: it does `page.goto("/")` and asserts on `examples/basic-spa` content ("Hover me", "Open popover"), so it cannot pass against `apps/web`'s dev server. This is the documented single-shared-`baseURL` constraint recorded in the merge change's task 9.8.
- [x] 3.6 Re-run the scripted browser audit that found this, confirming zero
  document-level scroll across every route in both themes at both widths
  — **32 checks (8 routes × light/dark × 1440px and 390px): zero problems**, and separately zero document-scrolling routes against the production build.
- [x] 3.7 Report which checks ran and which did not, with reasons

  — **Ran and passed:** fmt, workspace check, wasm32, clippy, core-crate tests, `adico-web` tests, `registry validate`, `registry build --check`, `openspec validate --all --strict`, `adico css check`, five Playwright suites, a 32-check browser audit, and a production-build measurement.
  **Ran and failed, pre-existing:** `wave3.spec.ts` (see 3.5).
  **Did not run:** `xtask playground-controls check` (pre-existing breakage on paths deleted by `c55c500`) and bare `npm test` (same shared-`baseURL` constraint).
  **Left as found:** `component-props check` staleness (`badge`, `button`, `color_picker`, `input_otp`, `tag_group`) — pre-existing and unrelated to this change.