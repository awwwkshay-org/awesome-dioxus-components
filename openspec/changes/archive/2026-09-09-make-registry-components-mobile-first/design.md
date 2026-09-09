## Context

See `proposal.md` for motivation. The relevant current-state facts, all
verified directly against the repo rather than assumed:

- 60 of 69 `registry/ui/*.rs` files have zero responsive Tailwind classes. The
  23 responsive utilities that exist across the other 9 files (`sheet.rs`,
  `alert_dialog.rs`, `dialog.rs`, `drawer.rs`, `breadcrumb.rs`, `toast.rs`,
  `empty.rs`, `theme_builder.rs`, `date_time_picker.rs`) are inherited verbatim
  from upstream shadcn, not authored here.
- No `--breakpoint-*` customization exists anywhere (`apps/playground/tailwind.css`,
  `packages/adico-cli/src/css.rs`), so the stock Tailwind v4 rem scale
  (40/48/64/80/96rem) applies in every consumer app the CLI generates CSS for.
  Tailwind v4.1.5 is the pinned toolchain version (`~/.dx/tools/`); container
  queries (`@container`, `@sm:`) and the `pointer-coarse:` variant are both
  core in this version, confirmed present in the compiled binary.
- `registry/lib/cn.rs`'s `cn()` is a plain space-join with no conflict dedupe.
  Base-vs-base class conflicts resolve by compiled stylesheet order, not
  argument order. `registry/ui/date_time_picker.rs:255-262` already documents
  the `!`-important workaround this forces when one component must override
  another's base class.
- `packages/adico-registry-core/src/lib.rs:1319-1329` reads each registry
  file's content from disk at `add` time and validates it against the
  manifest's checksum, raising `RegistryError::ChecksumMismatch` naming the
  file on a mismatch. Because `packages/adico-cli/src/main.rs` embeds
  `registry.json` via `include_bytes!` at compile time, a CLI binary built
  before a registry-source edit will mismatch against the edited file's new
  hash — so the CLI must be rebuilt before every `adico add --replace` that
  follows a `registry/ui/*.rs` or `registry.json` edit, and forgetting to do so
  now fails loudly rather than silently reinstalling stale content.
- `packages/adico-xtask/src/styling_usage.rs`'s `check_item` validates only
  `tailwindOnly` (raw-style constructs vs. `styleException`) and
  `tokenCompliant` (non-token colors/radius vs. their exceptions). It never
  reads `inspirationNote`, so responsive classes do not trip `styling-usage
  check` — but `contains_rounded_literal` (line 203) is a plain substring
  match against `rounded-{size}`, so a responsive rounded override
  (`sm:rounded-lg`) would trip condition (g) on any of the 41 items declaring a
  `radius: Radius` prop, with a misleading "stray rounded-* literal" message.
- `packages/adico-primitives/src/positioner.rs` already does horizontal
  collision handling: `clamp_cross` (line 104) shifts an anchored surface along
  its cross axis to stay in the viewport, the same behavior as Floating UI's
  `shift`. It publishes only `--adico-positioner-available-size` (a height
  variable, line 570) — there is no analogous width variable — and it measures
  the floating element's own rendered rect (line 260) to compute placement, so
  changing an anchored surface's width changes what the positioner measures.
- `apps/playground/src/main.rs`'s root is `h-screen w-screen overflow-hidden`,
  and the content panel in `routes.rs` is `overflow-y-auto`. Per the CSS
  overflow spec, `overflow-y: auto` with `overflow-x: visible` computes
  `overflow-x` to `auto` as well — so a too-wide component scrolls that panel
  instead of the document. Either mechanism means `document.scrollWidth <=
  document.clientWidth` would pass even when a component visibly overflows.
  `apps/playground/src/components/demo.rs`'s canvas is separately, deliberately
  `overflow-visible` (documented in its own module doc, to avoid clipping tall
  popups), which independently defeats the same check by a different route.
- Existing demo fixtures are frequently too minimal to exercise the failures
  this change fixes — e.g. `apps/playground/src/pages/tabs.rs` renders exactly
  two tabs, which cannot reproduce horizontal tab-list overflow.
- `tests/installation/*`'s `src/components/` is gitignored (confirmed via
  `git ls-files`), so those fixtures install fresh during their own
  integration tests and need no manual resync during this change's waves.

## Goals / Non-Goals

**Goals:**
- Every `registry/ui/*.rs` component renders without horizontal overflow at a
  375px viewport, verified by an automated harness, not just by hand.
- Every desktop-visible geometry value (≥640px) is unchanged from before this
  change, verified by an automated harness captured *before* any component is
  edited.
- All changes are pure CSS; no JavaScript viewport-detection dependency is
  introduced anywhere.
- The 69-component scope is a checked, resumable unit of work — a wave-based
  task list with an explicit per-component verdict, not a best-effort pass.

**Non-Goals:**
- The playground app shell (`Layout`, its two `ResizablePanelGroup`s) does not
  become responsive. This restates and respects the Non-Goal already recorded
  in `openspec/changes/archive/2026-09-08-add-playground-resizable-panels/design.md`.
- No `use_media_query` or other JS viewport-detection primitive is built.
  `Sidebar`'s viewport-driven mobile Sheet mode stays deferred (see the
  `adico-existing-components` delta spec) for exactly the reason its own
  module doc already gives.
- Touch-target sizing is audited and recorded, not remediated. Enlarging a tap
  target is a visible-appearance change on every viewport, which is a
  different risk profile than a mobile-only overflow fix, and folding it into
  this sweep would make "every override restores today's desktop value" false
  for those components. It is a named follow-up.
- No visual regression (pixel-diff) harness is introduced. `tests/visual/` is
  README-only today; this change measures geometry (bounding rects, computed
  styles), not pixels. That is a real, stated limitation, not a claim of full
  visual coverage.

The full mobile-first rule set (R0-R8) that every wave applies is authored
separately at `docs/adico/mobile-first-rules.md`, so it stands as a durable
reference outside this change's own lifecycle. The decisions below explain the
rationale; the doc is the checklist itself.

## Decisions

### D1: Mobile-first, not desktop-first with `max-*` overrides
Tailwind's own documentation states this directly: unprefixed utilities apply
to all screen sizes, `sm:`/`md:`/etc. apply at that breakpoint and up, and its
docs explicitly warn "Don't use `sm:` to target mobile devices." A desktop-first
approach (`w-[18rem] max-sm:w-full`) is possible via `max-*` variants but is
off-grain from how Tailwind's variant stacking is documented and, more
concretely, would diverge every touched component from the mobile-first shape
upstream shadcn already uses in the 9 files that have responsive classes today
(`sheet.rs`'s `w-3/4 sm:max-w-sm` is the in-repo model). Mobile-first also
composes correctly with `cn()`'s lack of merging, since overrides land in a
different media-query block rather than competing with a base class at equal
specificity.

**Alternative considered:** desktop-first `max-sm:` overrides, rejected because
it has zero precedent in the 69 files today and would make the diff read
backwards against upstream.

### D2: Two canonical rewrite forms, chosen by whether the original width already fits 375px
- **Form C (clamp, no breakpoint):** `<original> max-w-[calc(100%-2rem)]`. Used
  when the original fixed value already fits inside 343px (375px minus a 1rem
  gutter on each side) — the clamp is inert by construction above that width,
  so no breakpoint is needed and the desktop-preservation guarantee requires no
  reasoning about media-query precedence. Example: `popover.rs`'s `w-72`
  (288px) and `hover_card.rs`'s `w-64` (256px).
- **Form B (breakpoint restore):** `w-full max-w-[calc(100%-2rem)] sm:<original>`.
  Used when the original does not fit at 375px. The `sm:` variant restores the
  exact original value at ≥640px by winning on stylesheet order (Tailwind emits
  breakpoint variants after unprefixed utilities). Example: `dialog.rs`'s
  `max-w-lg` (512px).

**The discriminator that matters most is layout context, not just width.**
`w-full max-w-[X]` is not equivalent to `w-[X]` inside a flex or grid container:
`w-[X]` resolves `flex-basis: auto` to a definite width, while `w-full`
resolves to 100% of the flex container and competes with siblings before being
capped. `CalendarView` is exactly this case — it is a flex child in
`date_time_picker.rs`'s `sm:flex-row` and a block child inside
`date_picker.rs`'s `w-auto` popover, so the same rewrite must not be applied
identically in both contexts. Rule: inside a flex/grid parent, use
`w-full sm:w-[X]` (which emits the byte-identical `width: X` at ≥640px,
regardless of siblings); outside one, either canonical form above is safe.

**Alternative considered:** always using Form C everywhere for simplicity,
rejected because it silently breaks desktop rendering in flex/grid contexts
in a way that would only surface as a hard-to-attribute layout shift, not a
build error.

### D3: Positioner-anchored surfaces keep a definite width, never `w-full`
`positioner.rs` already handles horizontal collision (shifting an anchored
surface to stay in the viewport) but measures the floating element's own
rendered rect to do so. Switching an anchored surface from a fixed width to
`w-full` would change what gets measured — a `fixed`-positioned `w-full`
element measures as the full viewport width, feeding a different (and larger)
value into placement math than intended, in the same narrow-viewport regime
that has previously produced a stuck-`visibility:hidden` state on anchored
content. The safe rewrite is `w-[min(<original>,calc(100vw-2rem))]`: a
definite width at every viewport, clamped only when the viewport is narrower
than the original plus its gutter.

**Alternative considered:** relying entirely on `clamp_cross`'s existing
shifting behavior and leaving widths unchanged, rejected because shifting
repositions an overflowing box but does not shrink it — a `w-72` (288px)
anchored surface on a 320px-wide device (iPhone SE) still overflows by 8px+
regardless of position.

### D4: Sidebar gets a defensive clamp, not a CSS-only off-canvas collapse
An off-canvas collapse below `md:` was considered and rejected: Sidebar's
`Offcanvas` collapse is a manual prop with no default trigger below a
breakpoint, so a CSS-only auto-collapse would either require new JS to flip
that prop (reintroducing the dependency the deferral exists to avoid) or would
render the sidebar permanently uncollapsed-but-clipped, which is worse than
today. Shipping only `max-w-[85vw]` on the open panel is strictly defensive
(inert above ~301px, never makes the sidebar worse, never requires JS) and
keeps the change inside the "every override restores today's value or is a
named, reviewed exception" discipline the rest of this sweep follows.

### D5: `registry checksums --write` is scoped narrowly, not folded into `registry build`
`registry build`'s current contract is generate-and-validate, not
generate-and-mutate; changing that contract for every consumer of `registry
build` is a larger, riskier change than this sweep needs. A separate
`checksums --write` subcommand, following the repo's existing
`sync`/`check` convention (`styling-usage`, `primitive-usage`,
`primitive-compat`, `component-compat`), keeps `registry validate`'s role as
the actual gate unchanged and only removes the manual-transcription step that
is otherwise the highest-volume error source across a ~69-file, 7-wave sweep.

### D6: Touch-target sizing uses `pointer-coarse:`, deferred to a follow-up
Bumping control sizes at a Tailwind width breakpoint is measuring the wrong
axis — a narrow desktop window is not a touch device. Tailwind's
`pointer-coarse:` variant (confirmed present in the pinned v4.1.5 binary)
targets touch input directly and is inert on desktop by construction, which is
the right mechanism *when this work happens*. It is deferred out of this
change's scope per the Non-Goals above, and recorded here so the follow-up
starts from a decided mechanism rather than re-litigating width-breakpoint vs.
pointer-based sizing.

### D7: Harness measures per-element bounding rects, not `scrollWidth`
As documented in Context, a document- or canvas-level `scrollWidth` check would
pass vacuously in this app for two independent reasons (root `overflow-hidden`
clipping, and the demo canvas's deliberate `overflow-visible`). The harness
instead walks each rendered fixture's DOM and asserts
`getBoundingClientRect().right <= viewportWidth` (and `.left >= 0`) per visible
element, after animations settle. This works regardless of the ancestor
overflow chain, and names the specific offending element rather than only
reporting a page-level failure — important for triaging across 69 components.
Fixtures render on new shell-free playground routes (`/responsive/...`),
outside `Layout`, so the shell's own fixed-percentage panels never enter the
measurement.

### D8: Desktop-invariance baselines are captured before Wave 1 touches any component
The desktop-invariance Playwright project is written and run *before* any
registry source is edited, capturing today's actual computed geometry
(dialog `max-width`, calendar `width`/`height`, popover `width`, etc.) as the
literal expected values. This makes "desktop preserved" a regression gate
proven against a pre-change baseline, rather than an assertion written after
the fact against already-changed code (which would tautologically pass).

## Risks / Trade-offs

- **[`w-full max-w-[X]` silently breaks desktop rendering in flex/grid contexts]**
  → Mitigated by D2's context-aware rule; when a component's layout context is
  ambiguous, default to the flex-child form (`w-full sm:w-[X]`), which is
  correct in both contexts.
- **[Positioner-anchored width changes interact with the known
  stuck-`visibility:hidden` bug, worst in the same narrow-viewport regime this
  change targets]** → Mitigated by D3 (always keep a definite width), but Wave
  2 additionally requires live browser verification in Chrome, not just a
  passing automated check.
- **[No visual regression harness exists]** → "Desktop preserved" is verified
  by computed-geometry assertions (D8) plus a manual spot check in Chrome at
  375px per wave, not by pixel comparison. Stated as a limitation, not hidden.
- **[A 69-component, 7-wave sweep across 3 consumer apps is a large amount of
  work to keep resumable across sessions]** → `tasks.md`'s per-component
  checkboxes with recorded rule-category and verdict are the resumability
  mechanism; a wave's close-out (one CLI rebuild, three `adico add --replace`
  calls) is a single reviewable unit rather than 69 individual installs.
- **[`styling_usage`'s `inspirationNote` prose becomes stale for touched items]**
  → Does not fail CI (verified: `check_item` never reads that field), but is
  updated per wave for honesty. For `dialog`/`alert_dialog` specifically, the
  note becomes *more* accurate, since the gutter clamp this change adds is
  what upstream shadcn already ships.
- **[`sm:rounded-*` would trip `styling-usage check` with a misleading
  message]** → Prevented outright by design: no rule in this change ever
  proposes a responsive `rounded-*` class. Radius is owned by `radius.class()`.
- **[A long-lived `dx serve` session can serve a fresh Playwright page load a
  stale pre-rebuild bundle even after a full rebuild completes]** →
  Discovered in Wave 2: an already-open browser tab correctly reflected a
  hot-reload patch, but Playwright's fresh navigation to the same URL hit a
  stale bundle, producing spurious failures on already-fixed Wave 1
  components. Mitigation: restart `dx serve` fresh immediately before running
  the harness after every wave's reinstall, not only before manual Chrome
  spot-checks.
- **[Wave 6's "audit-only" ~24 components may hide real work]** → Two
  components initially assumed trivial (`alert.rs`, `card.rs`) already
  surfaced a two-column grid needing a fix during investigation. Budget for
  a handful of Wave 6 items being promoted into an earlier wave's category
  and re-reviewed there rather than rubber-stamped.
- **[Opening the new `/responsive/...` harness routes in a normal browser looks
  broken/unstyled compared to the playground's real navigation]** → Expected;
  documented in the routes' own module doc and in `tests/playwright/README.md`
  as harness-only, not a user-facing surface.

## Migration Plan

No data migration. Rollout is the wave sequence itself (Wave 0 → 6, plus a
deferred touch-target follow-up); each wave is independently revertable via
its own commit(s) since it ends in a clean, fully-validated state (registry
rebuilt, reinstalled, harness green). No feature flag is needed — every change
is a class-string edit with no runtime behavior branch, so there is nothing to
toggle; rollback is `git revert` of the wave's commit(s).
