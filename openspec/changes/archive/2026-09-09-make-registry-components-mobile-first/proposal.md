## Why

`adico` ships source-distributed Dioxus components that consumers install into
their own production apps, and today those components are effectively
desktop-only: 60 of 69 `registry/ui/*.rs` files contain no responsive class at
all, and the 23 responsive utilities that do exist across the other 9 files are
inherited verbatim from upstream shadcn rather than authored here. Several
components actively break at a 375px viewport — `dialog.rs` renders edge-to-edge
with no gutter, `alert_dialog.rs` has no base `max-w` at all, `calendar.rs` is
hard-pinned to 288×320px, `toast.rs` overflows 16px off the left edge below its
one existing breakpoint, and `tabs.rs`/`menubar.rs`/`navigation_menu.rs` overflow
horizontally with no scroll affordance. There is also no automated way to detect
any of this: zero Playwright specs set a viewport, and
`docs/adico/m4-parity-audit.md` records "responsive" as unmeasurable for all 38
audited components.

This change sweeps every registry component to a mobile-first baseline — so a
consumer installing an `adico` component gets something usable on a phone
without having to patch the source they now own — and builds the test harness
needed to keep that property true going forward.

## What Changes

- Audit and, where needed, rewrite all 69 `registry/ui/*.rs` components to a
  mobile-first layout: the unprefixed Tailwind class is the ~375px layout,
  `sm:`/`md:` restore today's exact desktop value. No JavaScript viewport
  detection is introduced — every fix is pure CSS (Tailwind breakpoints and,
  where the box being measured is a panel rather than the viewport, container
  queries).
- Fix the concrete known failures: fixed-position overlays (`dialog`,
  `alert_dialog`) gain a viewport gutter clamp; positioner-anchored overlays
  (`popover`, `hover_card`, `dropdown_menu`, `select`, `combobox`,
  `context_menu`, and the menu-content `min-w-[12rem]`s) keep a definite width
  while clamping to the viewport; `calendar`/`date_picker`/`date_time_picker`/
  `time_picker`'s hard-pinned dimensions gain a fluid escape; horizontal-flex
  containers (`tabs`, `menubar`, `navigation_menu`, `button_group`,
  `data_table`'s toolbar/footer) gain wrap or scroll; the two remaining bare
  `vh` sizes (`drawer.rs`, `toast.rs`) move to `svh`; `toast.rs`'s missing
  `left-4` (found during this change's own investigation — the toast overflows
  16px off the left edge below `sm` because `w-full` is paired with `right-4`
  and no left anchor) is fixed alongside its other responsive work.
- `Sidebar`'s mobile behavior stays explicitly deferred (a real viewport-driven
  Sheet mode needs a working JS media-query primitive this change does not
  build), but gains a defensive `max-w-[85vw]` clamp so an open sidebar can
  never exceed the viewport. **Incidental fix, confirmed with the user**:
  while capturing this change's desktop geometry baseline, discovered that
  `sidebar.rs`'s `w-[--sidebar-width]` compiles under the pinned Tailwind
  v4.1.5 toolchain to the invalid declaration `width: --sidebar-width`
  (missing `var(...)`), silently dropped by the browser — Sidebar has never
  rendered its intended 256px open width on this toolchain, unrelated to
  viewport width. Fixed alongside the clamp above since it's the same file
  and a one-line syntax correction (`w-[--sidebar-width]` →
  `w-(--sidebar-width)`, Tailwind v4's own var-reference syntax).
- Add a Playwright mobile project (375×812) plus a desktop-invariance project
  that captures today's computed geometry *before* any component is touched, so
  "desktop unchanged" is a provable regression gate rather than a claim. New
  shell-free playground routes (`/responsive/...`) host realistic fixtures for
  this harness, since the existing demo pages are both too minimal (e.g. the
  Tabs demo has only 2 tabs) and sit inside a shell (`overflow-hidden` root,
  `overflow-visible` canvas) that would make a naive overflow check pass
  vacuously.
- Add a `registry checksums --write` `adico-xtask` subcommand so the ~69
  SHA-256 checksums this sweep touches in `registry/registry.json` are computed
  mechanically instead of hand-edited across 7 waves.
- Touch-target sizing (many controls sit at 24-32px against 44px platform
  guidance) is audited and recorded, not fixed, in this change — enlarging a
  tap target changes mobile-visible appearance rather than removing an
  overflow, which is a different kind of change with its own review. Explicit
  non-goal, tracked as follow-up.

**Non-goals:**
- The playground app shell (`apps/playground/src/routes.rs`'s `Layout`, the
  nav/content and preview/controls `ResizablePanelGroup`s) stays desktop-only,
  per the explicit Non-Goal recorded on 2026-09-08 in
  `openspec/changes/archive/2026-09-08-add-playground-resizable-panels/design.md`.
  The new `/responsive/...` routes are additive harness-only routes outside
  that shell, not a reversal of it.
- No JavaScript viewport/media-query primitive is introduced. `sidebar.rs`'s
  module doc already documents that upstream's `document::eval` viewport
  detection is non-functional in this Dioxus runtime; this change does not
  revisit that.
- Touch-target sizing is audited, not remediated (see above).

## Capabilities

### New Capabilities

None. Every requirement in this change extends an existing capability's
concerns (component behavior, validation coverage, playground structure,
registry tooling) rather than introducing a new one.

### Modified Capabilities

- `adico-existing-components`: adds requirements that every registry component
  lays out without horizontal overflow at a 375px viewport, that responsive
  adaptation is mobile-first and preserves today's desktop rendering, and that
  Sidebar's mobile presentation stays deferred with a defensive clamp.
- `adico-component-validation`: adds a requirement for an automated viewport
  test harness (mobile + desktop-invariance Playwright projects) and updates
  how responsive results are reported (replacing the current "unmeasurable"
  verdict once the harness exists).
- `adico-playground-structure`: adds a requirement for shell-free harness
  routes that render components full-bleed with realistic fixtures, outside
  the existing desktop-only `Layout`.
- `adico-registry`: adds a requirement for a `registry checksums --write`
  xtask subcommand that computes and writes `registry.json`'s file checksums,
  reducing manual-edit risk across a large multi-file sweep. `registry
  validate` remains the actual CI gate; this subcommand only removes a manual
  step feeding it.

## Impact

- **Code:** all 69 files under `registry/ui/*.rs` (a mix of fixes and
  verify-only passes), plus their copies in `apps/playground`,
  `examples/basic-spa`, and `examples/basic-ssr` (re-synced via `adico add
  --replace` after each wave — `tests/installation/*` fixtures need no manual
  resync, since their `src/components/` is gitignored and they install fresh
  during their own tests).
- **Tooling:** `packages/adico-xtask` gains a `registry checksums --write`
  subcommand; `registry/registry.json` gains updated checksums for every
  edited file (no schema change).
- **Tests:** `tests/playwright/playwright.config.ts` gains `desktop`,
  `mobile`, and `desktop-invariance` projects; new
  `apps/playground/src/pages/responsive.rs` fixtures and new
  `tests/playwright/responsive*.spec.ts` specs.
- **Docs:** `docs/validation.md`'s surface matrix gains a responsive-layout
  row; `docs/adico/m4-parity-audit.md` and `m4-acceptance.md`'s "unmeasurable"
  responsive verdicts are superseded once the harness lands.
- **Editorial, not CI-gated:** `statics/styling_usage/*.json`'s
  `inspirationNote` free text is updated for items this change touches, since
  some currently assert "class-for-class match with upstream" in a way that
  becomes less precise (or, for `dialog`/`alert_dialog`, more accurate, since
  the width-gutter clamp this change adds is what upstream shadcn already
  ships). `styling-usage check` does not validate this field, so nothing here
  is CI-blocking.
- **No breaking changes** to public component APIs — this is a CSS-only sweep.
  Desktop-visible geometry is preserved by construction (verified by the
  desktop-invariance suite), so no consumer-visible behavior change at ≥640px.
