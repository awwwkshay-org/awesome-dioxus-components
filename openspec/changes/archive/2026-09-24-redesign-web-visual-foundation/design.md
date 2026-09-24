## Context

See `proposal.md` — Why. The design-relevant current state is four constraints
discovered in `apps/web` and the CLI, each of which rules out an otherwise
obvious implementation:

1. **The theme marker region is regenerated, not merged.**
   `packages/adico-cli/src/css.rs:483-498` rebuilds `tailwind.css` as
   `prefix + fresh theme_region() + suffix`, keeping only the bytes before
   `/* adico:theme:start */` and after `/* adico:theme:end */`. Anything written
   between them is destroyed by the next `adico add` of an item declaring
   `semanticTokens` or a `radiusToken`.
2. **No `dark` custom variant exists anywhere in the repo.** Tailwind v4's
   default therefore applies, and the compiled artifact confirms it:
   `apps/web/assets/tailwind.css:3490-3492` emits
   `.dark\:hover\:bg-accent\/50 { @media (prefers-color-scheme: dark) { … } }`.
   Meanwhile the runtime theme switch is purely class-based —
   `packages/adico-primitives/src/theme_mode.rs:165-176` adds and removes the
   `dark` class on `document.documentElement`. The two mechanisms are wired to
   different signals.
3. **`cn()` is a join, not `tailwind-merge`** (`apps/web/src/adico_lib/cn.rs`):
   it filters empties and joins with spaces, with no conflict resolution. Since
   equal-specificity Tailwind utilities resolve by stylesheet order rather than
   class-attribute order, passing a competing utility through a component's
   `class` prop is not reliably last-wins.
4. **`h-dvh` is load-bearing.** `apps/web/src/main.rs` uses
   `h-dvh … overflow-hidden` with `routes.rs`'s `main` as the scroll container,
   because `Demo`'s percentage `flex-basis` only resolves against a *definite*
   ancestor height. The merge change's task 9.8 records `min-h-dvh` silently
   degrading the playground's 70/30 split into content-based auto-sizing.

## Goals / Non-Goals

**Goals (design level)**
- Every byte this change adds to `tailwind.css` survives an arbitrary future
  `adico add`.
- Typography is expressed as tokens, so a later guide page can document it and a
  consumer can swap it, rather than as per-element font classes.
- The three fixes are minimal and independently revertable.

**Non-Goals (design level)**
- No change to the document scroll model. Nothing here introduces sticky
  positioning, anchor navigation, or scroll-spy — those arrive with the docs
  sidebar in a later change and will have to target `main`'s scroll container.
- No restyling of installed registry components. Layout changes act on the
  app's own wrapper elements, never by passing competing utilities into a
  registry component's `class` prop (constraint 3).
- No `packages/` change. The durable home for the `dark` custom variant is
  `css.rs::theme_region()`; that is a separate CLI change.

## Decisions

### D1 — All app-owned CSS goes in the prefix, above `/* adico:theme:start */`

`tailwind.css` gains a block between line 2 (`@source "./src";`) and line 4
(the start marker) holding `@font-face` rules, an `@theme` with the font
tokens, the type-scale `@utility` definitions, and the `@custom-variant`.

*Alternative rejected — writing inside the marker region.* It is the natural
place for `@theme` content and sits beside the existing token block, but
constraint 1 means the next `adico add` deletes it with no warning and no diff
the developer would think to check.

*Alternative rejected — a second stylesheet.* A separate `fonts.css` linked
alongside the compiled output would survive regeneration, but Tailwind `@theme`
tokens must be in the compiled entry to generate utilities, so the fonts and
the tokens would have to live in different files. One prefix block is simpler.

*Consequence:* the prefix is now meaningful, not incidental. The spec change
records this so a future contributor does not "tidy" it into the marker region.

### D2 — Self-hosted variable `woff2`, subset to latin

Geist (UI/display) and Geist Mono (code), both SIL OFL 1.1 — the same typeface
family shadcn/ui uses, which is the visual reference class the proposal targets,
and license-compatible with this repo's MIT OR Apache-2.0 dual license as a
separately-licensed asset.

Variable `woff2`, latin subset only, `font-display: swap`, with a real fallback
stack (`ui-sans-serif, system-ui, …`). Two files rather than a weight set per
family keeps the vendored bytes small while giving the full weight axis the type
scale needs.

*Alternative rejected — Google Fonts `<link>`.* One line of setup, but it adds a
third-party request on every page load, breaks offline `dx serve`, and would
make the site's own "you own the source" pitch ring hollow.

*Alternative rejected — system font stack only.* Zero bytes, but it is what the
site does today, and it is the specific thing that makes it read as unfinished.

Fonts land in `apps/web/assets/fonts/` with their `OFL.txt` alongside. Per
`docs/adico/*` provenance practice, a vendored third-party asset is recorded —
this change adds the license file and an entry noting origin and version.

### D3 — `@custom-variant dark (&:is(.dark *))` at app level, CLI fix filed separately

This single line makes literal `dark:` utilities resolve against the `dark`
class that `theme_mode.rs` already applies, closing the gap in constraint 2.

The affected utilities are few and known — `dark:hover:bg-accent/50` (Button
ghost), `dark:aria-invalid:ring-destructive/40`, `dark:hover:bg-white/10`,
`dark:text-emerald-500`, and the `dark:focus-visible:ring-*/40` set — so the
blast radius is hover, focus-ring, and invalid-state colour on a handful of
components.

*Alternative rejected — fixing `css.rs::theme_region()` in this change.* That is
the correct long-term home and every consumer needs it, but it changes rendering
in every project that ever runs `adico add`, and it would drag a CLI change,
`registry build --check`, and the embedded registry into what is otherwise an
app-only change. Filed as its own change; when it lands, the app-level line
becomes redundant and is removed.

*Note:* this is a deliberate, visible behavior change for visitors whose OS and
app themes disagree — recorded as a scenario in the spec rather than slipped in.

### D4 — Prose rendering is a parser, not a markdown dependency

`apps/web/src/components/prose.rs` exposes a component that takes a `&str`,
splits it on backtick pairs, and emits alternating text and styled `<code>`
nodes. Unpaired backticks render literally.

*Alternative rejected — a markdown crate (`pulldown-cmark` or similar).* It
would handle bold, links, and lists too, but registry `documentation` fields are
single-paragraph plain prose whose only markdown construct is the inline-code
span — verified across all 69 items. A parser dependency compiled to wasm to
handle one construct is the kind of anticipatory abstraction CLAUDE.md warns
against. If registry prose later grows real markdown, this decision gets
revisited with a concrete need behind it.

This lives under `apps/web/src/components/` as app-level wiring per
`adico-web-structure` #7/#11 — it is not a registry component and does not
require one.

### D5 — Landing-page counts read from the embedded manifest

`pages/docs/data.rs` already `include_str!`s `registry/registry.json` at compile
time behind a `OnceLock`. The landing page reuses that same accessor to count
`registry:ui` items rather than carrying the literals `69` and `67`, which are
already at risk of going stale. The primitive count has no manifest source, so
it stays a literal for now with a comment saying why.

### D6 — Equal-height card grids via grid row alignment, not fixed heights

The "Why adico" and "Install" rows are CSS grid; `items-stretch` on the grid
plus `h-full` on the app's own card wrapper gives equal heights without touching
`Card`'s own classes. The install-command rows get the `min-w-0` + wrap
treatment rather than a truncating overflow, so the full command is readable —
the current ellipsis hides the part a user most needs to copy.

## Risks / Trade-offs

- **The `dark:` fix changes what some visitors see** → It is the fix, not a
  regression, but it is recorded as a spec scenario and called out in the
  proposal's Impact so it is not discovered as a surprise. Verified manually
  with OS theme set opposite the app theme.
- **A future `adico add` could still clobber the prefix if `css.rs`'s splice
  logic changes** → The spec now states the rule, and `adico css check` runs in
  verification. A `styling-usage`-style guard is not warranted for one block.
- **Vendored font binaries add repo weight** → Variable + latin subset + `woff2`
  only keeps it to roughly two files in the low hundreds of KB. No weight set,
  no `woff` fallback, no italic axis unless a page needs one.
- **Font swap causes a brief layout shift** → `font-display: swap` with a
  metric-adjacent fallback stack. Per the standing no-layout-shift preference,
  this is checked visually at both widths rather than assumed.
- **Type-scale utilities could drift from what pages actually use** → The scale
  is deliberately small (display / heading / lead / body / caption). Pages are
  migrated to it in this change so there is no half-converted state.

## Migration Plan

No data, no API, no consumer-facing contract changes — `apps/web` is an
application, not a published crate. Deployment is the existing `dx build`. The
change is revertable as a single commit; reverting restores the system font
stack and the OS-driven `dark:` behavior.

**One ordering dependency, and it is real.** `openspec validate --all --strict`
passes with this change's MODIFIED delta in place (verified: 16 passed, 0
failed) even though `adico-web-structure` is not yet in `openspec/specs/`.
Validation is satisfied, but **archiving this change requires that capability to
exist in `openspec/specs/` first**, since a MODIFIED delta is applied against a
base spec. So:

- Implementation and review of this change may proceed now.
- Archiving it must follow `2026-09-24-merge-apps-into-web` being unblocked
  (its task 9.8) and archived.

If that merge change is instead revised or abandoned, this change's delta is
re-pointed at whatever capability replaces it — the requirement text is
independent of which file it lands in.

## Open Questions

- Whether the display face and the UI face should differ (a distinct display
  face for the landing hero only). Deferred: it changes no requirement, no task
  boundary, and no spec — it is a token value swap once the scale exists.
