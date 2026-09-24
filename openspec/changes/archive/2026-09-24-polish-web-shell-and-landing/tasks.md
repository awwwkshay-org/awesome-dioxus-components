# Tasks

Design decisions referenced below are `D1`–`D4` in `design.md`.

## 1. De-duplicate the playground shell (D1)

- [x] 1.1 Remove the logo, the "Adico Playground" heading, the `ThemeSwitcher`
  and the `ModeToggle` from `PlaygroundLayout` in **both** presentations,
  keeping `ThemeBuilderLauncher`. Verify by asserting in a live page that the
  playground renders exactly one adico logo, one `ThemeSwitcher` and one
  `ModeToggle` — counted from the DOM, not by eye
  — **done, counted from the DOM not by eye: exactly 1 `img[alt="adico logo"]` on a playground page (was 2), and "Adico Playground" no longer appears anywhere. `ThemeSwitcher` and `ModeToggle` now have exactly one call site each in `routes.rs`, both inside `SiteLayout`; `PlaygroundLayout` keeps only `ThemeBuilderLauncher`, in both presentations.**
- [x] 1.2 Verify the `ResizablePanelGroup` geometry is untouched: the nav and
  content panels keep `flex-basis` 18%/82%, and the preview/controls split
  keeps 70/30

  — **unchanged: nav/content `flex-basis` 18%/82%, preview/controls 70%/30%, measured live.**

## 2. Navigation filter (D2)

- [x] 2.1 Add a filter to `apps/web/src/components/nav.rs`, composed from the
  installed `Input`, shared by both presentations. Verify typing narrows the
  list, that the remaining entries stay in flat A→Z order, and that clearing
  restores all 69
  — **done: filter in `nav.rs`, shared by both presentations. 69 entries → typing `too` yields exactly `Toolbar`, `Tooltip` (verified still sorted) → clearing restores all 69.**
- [x] 2.2 Verify a filter matching nothing renders an explicit empty state
  rather than a blank column, and that filter state does not survive a
  navigation

  — **done: a non-matching filter renders "No component matches …" rather than a blank column; filter state is a `use_signal` in `NavList`, so it is per-mount and does not survive navigation or a sheet close.**

## 3. Canvas legibility (D3)

- [x] 3.1 Raise the preview grid's contrast in `apps/web/src/components/demo.rs`.
  Verify the grid is actually rendered in both light and dark by sampling the
  computed background against the panel's own surface — do not judge by eye
  — **done, measured not eyeballed: the computed `background-image` resolves to `rgba(226, 232, 240, 0.45)` at `32px 32px`, where it previously did not render at 0.08.**
- [x] 3.2 Verify panning and the Center control still behave exactly as before

  — **unchanged: `playground-resizable-split` 3/3 and `playground-enriched-demos` pass, which exercise the pan/drag machinery.**

## 4. Landing showcase (D4)

- [x] 4.1 Add a component showcase to `apps/web/src/pages/index.rs`, composed
  from installed registry components, excluding popup-family components.
  Verify real components render and remain interactive
  — **done: buttons, badges, form controls, tabs and progress — all live installed components, all interactive. Popup-family components excluded per D4.**
- [x] 4.2 Verify no layout shift or horizontal overflow at 1440px and 390px

  — **done, and this caught a real defect (below).** Final state: 0 horizontal overflow and 0 document-level scroll across 32 checks at 1440px and 390px in both themes.

## 5. Validation

- [x] 5.1 Repo baseline: `cargo fmt --all --check`, `cargo check --locked
  --workspace`, clippy over the five core crates with `-D warnings`,
  core-crate tests, `cargo test --locked -p adico-web`,
  `openspec validate --all --strict`
  — **all pass:** fmt, workspace check, clippy `-D warnings`, core tests, `cargo test -p adico-web` 142/0, `openspec validate --all --strict` 14/0.
- [x] 5.2 `cargo check --locked -p adico-web --target wasm32-unknown-unknown`,
  `adico css build` + `adico css check`
  — **both pass.**
- [x] 5.3 **The shell regression gate.** With `dx serve` fully rebuilt:
  `playground-resizable-split.spec.ts`, `--project=mobile` (covers
  `responsive-shell`), `--project=desktop-invariance` (covers
  `responsive-desktop-shell`)
  — **all pass:** `playground-resizable-split` 3/3, `--project=mobile` 21/21, `--project=desktop-invariance` 14/14, plus `playground-enriched-demos` + `playground-time-picker` 7/7.
- [x] 5.4 Scripted browser audit across `/`, `/playground/*`, `/docs` in light
  and dark at 1440px and 390px: no console errors, no document-level scroll,
  no horizontal overflow
  — **32 checks (8 routes × light/dark × 1440px and 390px): zero problems.**
  Getting there surfaced a genuine defect, which is the substantive finding of this change. The audit flagged the *document itself* scrolling on `/` — the one thing this app's `h-dvh` layout is built to prevent. Root cause: `adico_primitives`' `BubbleInput` (the hidden native input behind `Checkbox`/`Switch`) is `position: absolute` with **no positioned ancestor**, so its containing block is the initial containing block and `main`'s `overflow-y: auto` never clips it; it extends `documentElement.scrollHeight` instead.
  Established by measurement, not inference: baseline `/` measured 900 and mine 1027; bisecting the showcase isolated it to the checkbox group; and it reproduces against a real `dx build` served statically, so it is not a dev-server artifact.
  **Pre-existing:** `/docs/components/checkbox` already leaked (977 vs 900) before this change. Mitigated here by establishing a containing block on the app's own wrapper — ordinary composition, not a registry patch — and recorded in full in `FOLLOWUPS.md`, since the real fix is in `packages/adico-primitives` and needs its own change.
- [x] 5.5 Report which checks ran and which did not, with reasons

  — **Ran and passed:** fmt, workspace check, wasm32, clippy, core tests, 142 `adico-web` tests, `openspec validate --all --strict`, `adico css build` + `adico css check`, four Playwright suites, and a 32-check scripted browser audit.
  **Did not run, with reasons:** `xtask playground-controls check` (pre-existing failure on `apps/playground/src/components/ui`, deleted by `c55c500`; this change touches no `apps/web/src/components/ui/*`) and bare `npm test` (shared `baseURL` across specs targeting different dev servers).
  **Left as found:** `component-props check` staleness, pre-existing and unrelated (`badge`, `button`, `color_picker`, `input_otp`, `tag_group`).