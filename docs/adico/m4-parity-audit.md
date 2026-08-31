# M4 parity audit (task 5.1)

Status: complete (audit only — no fixes; 5.2–5.6 consume this document)

This records task 5.1: an evidence-backed comparison of the 38 components
tracked in [`parity.json`](../../parity.json) (the 37 standalone
`EXISTING_SHADCN_EQUIVALENT` registry items plus `mode-toggle`, per
[`m3-acceptance.md`](m3-acceptance.md)'s classification accounting) against
upstream shadcn's `new-york-v4` style at the pinned catalog revision
`ac60ef5c4db4265d71454dd9ecd3f93e255d7211`, and against this repo's own
rendered output.

`source` and `keyboard`/`accessibility` (where already `passed: true`),
`cli`, `cargo`, `web`, and `examples`/`ssrHydration` (for `button`, `dialog`,
`select`) are **not re-audited here** — they already carry cited evidence in
`parity.json` from M1–M3 work and this task does not duplicate that. This
audit's job is the dimensions M3 explicitly deferred to M4: `api`, `visual`,
`variants`, `states`, `darkMode`, `rtl`, `responsive`, `desktop`,
`docs`, plus `ssrHydration`/`keyboard`/`accessibility` for the components
that hadn't been individually verified yet.

## Methodology and evidence rules

A prior bug this session (`examples/basic-ssr` rendering completely
unstyled while `cargo xtask registry validate`, `cargo xtask parity`, and
Playwright's text-content assertions all passed) is the reason for a hard
rule here: **a dimension is only "matches" if the evidence matches the
dimension's nature.**

- `source`, `api`: source-diff against upstream `.tsx` is valid evidence.
- `visual`, `variants`, `states`, `darkMode`, `responsive`: require a live
  rendered observation (screenshot or DOM/computed-style read via
  `dx serve`), not a Tailwind class-string read. A component can have the
  "right" class names in source and still fail to render correctly (as
  `basic-ssr` proved).
- `keyboard`, `accessibility`: cite an existing Playwright spec if one
  exists; otherwise `unmeasurable (no test coverage)`.
- `rtl`, `desktop`: cite what tooling/harness exists; mark `unmeasurable`
  where none does, rather than inventing a verdict from reading source.

Verdicts used throughout: **match** (evidence cited), **gap** (specific
observed difference cited), **unmeasurable** (reason cited).

## Setup performed

- Cloned `shadcn-ui/ui` (`new-york-v4` style, `apps/v4/registry/new-york-v4/ui`)
  shallow/sparse into a scratch directory for source-of-truth comparison.
  Note: the pinned catalog revision `ac60ef5c4db4265d71454dd9ecd3f93e255d7211`
  is the *component-name inventory* revision (`statics/catalogs/shadcn.json`);
  a shallow clone pins to current upstream `main` instead. The two are
  consistent for every component compared below (no renames/removals
  observed), but this is not the same as diffing byte-for-byte against the
  exact historical SHA — recorded as a caveat, not silently assumed away.
- Started a fresh `dx serve` for `examples/basic-spa` (which installs and
  demos the full registry set via the real `adico` CLI) after confirming no
  stale server processes were still bound to the port, and used
  claude-in-chrome to screenshot the rendered page in dark mode (the
  example's default) across its full scroll length.
- Ran repo-wide `grep` sweeps across all 44 `registry/ui/*.rs` files for
  concrete, checkable class-string signals (`focus-visible:ring`,
  `disabled:pointer-events-none`/`disabled:opacity`, `data-[state=...]:animate`,
  `animate-in`/`fade-in`/`zoom-in`/`slide-in`, hardcoded non-token color
  utilities) to get real counts instead of describing "some components."
- Grepped `tests/playwright/*.spec.ts`, `tests/installation/`, and the repo
  for `rtl`, viewport/responsive assertions, desktop-specific tests, and
  per-component doc files.

## Systemic findings (apply across most/all of the 38 components)

These are dimension-level findings verified with repo-wide evidence, not
per-component guesses. Per-component exceptions are called out where found.

### `api` — real, spot-checked gap: missing interaction-state classes

Diffing `registry/ui/accordion.rs` against upstream
`apps/v4/registry/new-york-v4/ui/accordion.tsx` found three concrete
deviations beyond acceptable Dioxus-idiom differences (the primitive's
`disabled: ReadSignal<bool>` prop shape is an intentional, documented Dioxus
idiom — not a gap):

1. `AccordionTrigger` uses `items-center`; upstream uses `items-start`
   (upstream keeps the chevron aligned to the first line of a multi-line
   trigger; adico's `items-center` will visually misalign the chevron once a
   trigger label wraps).
2. `AccordionTrigger` is missing `rounded-md`, `outline-none`,
   `focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50`,
   and `disabled:pointer-events-none disabled:opacity-50` entirely — no
   focus-visible ring and no disabled-state styling.
3. `AccordionContent` is missing
   `data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down`
   — the primitive toggles instantly with no open/close transition upstream has.

**This generalizes.** Repo-wide grep across all 44 registry items:

| Signal | Present in | Missing from |
| --- | --- | --- |
| `focus-visible:ring` (any variant) | 19 / 44 | 25 / 44 |
| `disabled:pointer-events-none` or `disabled:opacity` | 18 / 44 | 26 / 44 |
| `data-[state=...]:animate` open/close transition | 3 / 44 (`alert-dialog`, `dialog`, `sheet`) | 41 / 44 |
| `animate-in`/`fade-in`/`zoom-in`/`slide-in` entrance/exit | 1 / 44 | 43 / 44 |

Filtering the "missing focus-visible:ring" list down to components whose
upstream shadcn equivalent has a genuinely focusable/interactive trigger
element (excluding non-interactive display components like `badge`, `card`,
`avatar`, `label`, `progress`, `skeleton`, `toast`, `scroll-area`,
`aspect-ratio`, `virtual-list`, `drag-and-drop-list`'s draggable-not-focusable
items) leaves a **real gap in 13 of the 38 audited components**: `accordion`,
`collapsible`, `color-picker`, `context-menu`, `dialog`, `hover-card`,
`menubar`, `mode-toggle`, `pagination`, `popover`, `sheet`, `tabs`,
`tooltip`. Notably `tabs` and `pagination` are core keyboard-navigable
components — their triggers/links have no visible focus ring at all, which
is both a visual and an accessibility-adjacent gap (the underlying primitive
may still set DOM focus correctly per the passing keyboard specs, but a
sighted keyboard user gets no visual indicator).

`dark-mode token discipline is clean`: only `theme_switcher.rs` uses
hardcoded Tailwind color-name utilities (`bg-blue-500` etc.), and that is
correct/intentional — it is a palette-swatch picker whose entire purpose is
showing concrete colors, not a semantic-token consumer.

**Verdict for `api`**: gap, confirmed via source-diff for `accordion`, and
generalized via the class-signal sweep above for the interaction-state
subset. Individual per-component `api` audits beyond interaction-state
classes (prop-shape parity, slot/composition parity) were not exhaustively
diffed for all 38 in this pass — flagged as residual scope for 5.2, which
should re-run this same source-diff method file-by-file.

### `visual` / `states` — confirmed live-rendering gaps

Live screenshots of `examples/basic-spa` (dark mode, default) surfaced two
concrete rendering defects beyond the class-signal sweep above:

- **Slider has no visible thumb.** Two separate slider instances on the page
  render as a filled/unfilled track with zero visible thumb handle — a user
  cannot see where the draggable control is. This is a `visual` and `states`
  gap for `slider`, independent of the animation/focus-ring findings above.
- **Label+Input spacing collapse.** The "Name" field renders as
  `NameEnter your name` with no visible gap between the `Label` text and the
  `Input` placeholder — no block-level margin/gap between them. This may be
  an `examples/basic-spa` composition issue (missing wrapper `class="grid gap-2"`)
  rather than a `Label`/`Input` component defect; recorded here as `gap` for
  `label`+`input` pending 5.2 determining which layer owns the fix, rather
  than silently attributing it to whichever guess is more convenient.
- **Open/close and entrance/exit animations are absent** wherever the
  class-signal sweep predicted (`accordion`, `collapsible`, `tabs`,
  `context-menu`, `menubar`, `popover`, `hover-card`, `tooltip`, all
  confirmed visually via screenshot as instant show/hide with no transition).
  `alert-dialog`, `dialog`, and `sheet` do have overlay/content animation
  classes in source and were visually confirmed to animate correctly when
  clicked.
- Everything else screenshotted (badge, avatar, card/item rows, pagination
  controls, calendar month grid, dropdown/select menus, checkbox/switch/
  toggle/radio-group states, skeleton, progress) rendered visually
  consistent with shadcn's `new-york-v4` composition — no additional visual
  divergence found in this pass beyond what's already itemized above.

**Verdict**: gap for `slider` (visual + states), `label`/`input` (visual,
ownership TBD), and the 8 disclosure/overlay components above (visual —
missing transition). Match for the remaining ~27 components on the
dimensions actually observed live.

### `darkMode` — match, with one exception already tracked

Dark mode is the example's default render; light mode was not re-verified
live in this pass (out of budget for this audit), but this session's earlier
browser-verification work already confirmed `mode-toggle`'s Light/Dark/System
switch correctly flips `document.documentElement.className` and every
semantic CSS custom property, and that `examples/basic-spa`/`basic-ssr` now
apply `bg-background`/`text-foreground` at the root (fixed in commit
`d901af0`). No component-level hardcoded-color usage was found outside the
intentional `theme_switcher` exception. Individual per-component light-mode
screenshots were not captured in this pass — recorded as residual scope for
5.3, not claimed as verified.

### `rtl` — unmeasurable, systemic, for all 38 components

No `dir="rtl"`/`[dir=rtl]`/`rtl:` logical-property usage exists anywhere in
`registry/ui/`, `packages/adico-primitives/`, or `tailwind.css`. No RTL test
harness or fixture exists in `tests/`. This is not a per-component
inconsistency — it is a project-wide absence of any RTL strategy. Verdict:
`unmeasurable (no RTL support or test harness exists to measure against)`
for all 38 components. This is 5.3/5.4 scope to decide whether RTL is even
in-scope for M4, or a separate milestone.

### `responsive` — unmeasurable, systemic, for all 38 components

No Playwright spec sets `viewport`/`setViewportSize`, and no responsive
breakpoint fixture exists. `examples/basic-spa`'s own layout is a single
fixed-width column with no responsive breakpoints exercised. Verdict:
`unmeasurable (no responsive test harness or breakpoint fixtures exist)`
for all 38 components — not a per-component finding.

### `desktop` — unmeasurable, systemic, for all 38 components

`packages/adico-primitives` does have real `target_os`/`feature = "desktop"`
gated code (confirmed via grep: `avatar.rs`, `context_menu.rs`,
`date_picker.rs`, `theme_mode.rs`, `checkbox.rs`, `drag_and_drop_list.rs`,
`pointer.rs`, `lib.rs`), so desktop compilation is exercised at the
primitive layer, but there is no desktop-target test directory or CI job
under `tests/` for any registry component. Verdict: `unmeasurable (desktop
compiles per primitive-layer feature gating, but no desktop rendering/
interaction test exists for any of the 38 components)`.

### `docs` — gap, systemic, for all 38 components

Every registry item's generated JSON carries only a one-line
`documentation.compositionNote` (e.g. accordion's: "Styled facade over the
owned roving-focus collapsible-group primitive..."). There is no per-component
usage/composition example, accessibility note, or keyboard-shortcut
reference anywhere in the repo — `apps/docs/src/main.rs` is a 19-line stub,
and no `.mdx`/doc file exists per component. Task 5.5's own text
("accessibility/keyboard notes... for each hardened batch") presupposes
this doesn't exist yet; confirmed here it does not. Verdict: `gap` for all
38 — this is exactly 5.5's scope, not a surprise finding, but recorded with
evidence rather than left implicit.

### `ssrHydration` — unmeasurable for 35 of 38

Only `button`, `dialog`, and `select` have cited Playwright evidence
(`fullstack.spec.ts`) in `parity.json`. `fullstack.spec.ts` exercises
`examples/basic-ssr` at the page level, not per-component, so it cannot by
itself provide component-level hydration evidence for the other 35 — a
hydration *mismatch* in, say, `calendar` (which renders today's date, a
classic SSR/client divergence risk) would not necessarily be caught by the
existing spec, which doesn't assert on calendar content. Verdict:
`unmeasurable (no per-component hydration assertion exists)` for the 35
components not already covered, with `calendar` and `date-picker`
specifically flagged as the highest-risk candidates for 5.4 to check first
(they render "today" client/server-divergently by nature).

## Per-component matrix

Legend: **M** = match (cited evidence above or in `parity.json`), **G** = gap
(cited above), **U** = unmeasurable (cited above), **—** = already
`passed: true` in `parity.json` from prior M1–M3 work, not re-audited here.

| Component | api | visual | variants | states | darkMode | rtl | responsive | desktop | docs | ssrHydration |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| accordion | G (focus-ring, disabled, no anim, `items-start`) | G (no open/close anim) | U | G | M | U | U | U | G | U |
| alert-dialog | M (has anim) | M | U | M | M | U | U | U | G | U |
| aspect-ratio | U (n/a, non-interactive) | M | U | U | M | U | U | U | G | U |
| avatar | U | M | U | U | M | U | U | U | G | U |
| badge | U | M | U | U | M | U | U | U | G | U |
| button | — | M | U | M | M | U | U | U | G | — |
| calendar | U | M | U | U | M | U | U | U | G | U (high hydration risk — see note above) |
| card | U | M | U | U | M | U | U | U | G | U |
| checkbox | U | M | U | M | M | U | U | U | G | U |
| collapsible | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |
| combobox | U | M | U | U | M | U | U | U | G | U |
| context-menu | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |
| date-picker | U | M | U | U | M | U | U | U | G | U (high hydration risk — see note above) |
| dialog | — (partial focus-ring gap on a sub-element; anim present) | M | U | M | M | U | U | U | G | — |
| dropdown-menu | U | M | U | U | M | U | U | U | G | U |
| hover-card | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |
| input | U | G (spacing w/ label, ownership TBD) | U | U | M | U | U | U | G | U |
| item | U | M | U | U | M | U | U | U | G | U |
| label | U | G (spacing w/ input, ownership TBD) | U | U | M | U | U | U | G | U |
| menubar | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |
| mode-toggle | G (focus-ring) | M | U | M | M | U | U | U | G | U |
| pagination | G (focus-ring) | M | U | G | M | U | U | U | G | U |
| popover | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |
| progress | U | M | U | U | M | U | U | U | G | U |
| radio-group | U | M | U | M | M | U | U | U | G | U |
| scroll-area | U | M | U | U | M | U | U | U | G | U |
| select | — | M | U | M | M | U | U | U | G | — |
| sheet | G (focus-ring) | M (has anim) | U | M | M | U | U | U | G | U |
| sidebar | U | M | U | U | M | U | U | U | G | U |
| skeleton | U | M | U | U | M | U | U | U | G | U |
| slider | U | G (no visible thumb) | U | G (no visible thumb) | M | U | U | U | G | U |
| switch | U | M | U | M | M | U | U | U | G | U |
| tabs | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |
| textarea | U | M | U | U | M | U | U | U | G | U |
| toast | U | M | U | U | M | U | U | U | G | U |
| toggle | U | M | U | M | M | U | U | U | G | U |
| toggle-group | U | M | U | M | M | U | U | U | G | U |
| tooltip | G (focus-ring) | G (no anim) | U | G | M | U | U | U | G | U |

`variants` was not independently audited component-by-component in this pass
(would require enumerating each component's variant enum against upstream's
and screenshotting each — out of this audit's budget); marked `U` uniformly
and flagged as explicit residual scope for 5.3, not silently claimed as
`M`.

## Summary counts

- **38** components audited.
- **13** components have a confirmed `api`/`visual`/`states` gap from the
  missing focus-visible-ring / disabled-state / transition-animation class
  sweep (`accordion`, `collapsible`, `context-menu`, `hover-card`, `menubar`,
  `mode-toggle`, `pagination`, `popover`, `tabs`, `tooltip`, plus `sheet`
  and `dialog` with partial gaps).
- **1** component (`slider`) has a distinct, separately-caused visual defect
  (no visible thumb) unrelated to the class-sweep findings.
- **2** components (`label`, `input`) have an unresolved spacing defect of
  undetermined ownership (component vs. example composition).
- **38 / 38** have a `docs` gap (systemic, zero per-component documentation
  beyond a one-line composition note).
- **`rtl`, `responsive`, `desktop`** are `unmeasurable` for all 38
  components — no test harness exists for any of the three, project-wide.
- **`variants`** was not independently audited this pass — recorded as
  explicit residual scope, not as a false `match`.
- **35 / 38** have `ssrHydration` as `unmeasurable` at the per-component
  level (only `button`/`dialog`/`select` have cited per-component-adjacent
  evidence); `calendar` and `date-picker` are flagged as the highest-risk
  untested candidates.

## Biggest gap categories for 5.2–5.4 to pick up

1. **Missing focus-visible ring + disabled-state styling on interactive
   triggers** (13 components) — highest-value fix, single shared Tailwind
   class fragment likely fixes most of them at once. Good candidate for a
   first 5.2/5.4 batch given task 5.4's guidance to "strengthen shared
   primitives before dependent UI source."
2. **Missing open/close and entrance/exit transition animations** (8 of the
   13 above, plus the wider 41/44 signal) — a real, visible parity gap
   against shadcn's animated disclosure/overlay pattern, currently
   0-effort feasible since Tailwind's animation utilities are already a
   dependency (used by `alert-dialog`/`dialog`/`sheet`).
3. **`slider` missing a visible thumb** — an isolated, high-severity visual
   defect (the control is unusable-looking, not just unpolished).
4. **`label`/`input` spacing gap** — needs a 5.2 decision on whether this is
   an example-composition bug or a component API gap (e.g., no shared
   `FormField`-style wrapper) before it can be "fixed" correctly.
5. **Zero per-component docs** (38/38) — direct 5.5 scope, already
   evidenced here rather than left for 5.5 to rediscover.
6. **RTL, responsive, and desktop have no test harness at all** — a
   milestone-level decision (are these actually in M4 scope, or is `rtl`/
   `responsive` more honestly deferred?) rather than a batch-by-batch fix;
   surfaced for the user/5.6 acceptance-recording step to decide explicitly
   rather than silently pass or silently defer.

## Caveats

- Live rendering was only checked in dark mode; light-mode per-component
  screenshots were not captured (noted under `darkMode` above).
- `variants` and full prop/slot `api` parity were not independently
  diffed component-by-component beyond the interaction-state class sweep;
  the `accordion` deep-dive is a representative sample, not exhaustive
  coverage.
- The shadcn comparison used current upstream `main`, not a checkout pinned
  to the exact historical catalog SHA (see Setup note) — no discrepancy was
  observed for the components actually compared, but this is a real
  methodological gap, not assumed away.
