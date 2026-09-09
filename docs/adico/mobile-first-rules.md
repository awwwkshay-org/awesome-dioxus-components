# Mobile-first responsive rules (R0-R8)

Status: authored for `make-registry-components-mobile-first`; applied across
that change's Waves 1-6.

This is the checklist every `registry/ui/*.rs` component is audited against.
Each rule states its trigger, the rewrite, and why the rewrite preserves
today's desktop rendering. See
`openspec/changes/make-registry-components-mobile-first/design.md`'s
Decisions section for the rationale behind the two canonical forms (D2) and
the positioner-specific form (D3) referenced below.

## R0 — Global prohibitions

- **Never emit a responsive `rounded-*` class** (`sm:rounded-lg`,
  `max-sm:rounded-none`, …). `packages/adico-xtask/src/styling_usage.rs`'s
  `contains_rounded_literal` is a plain substring match against
  `rounded-{size}`, so any responsive rounded override trips `styling-usage
  check`'s condition (g) on the 41 items declaring a `radius: Radius` prop,
  with a misleading "stray rounded-* literal" message. Radius is owned by
  `radius.class()`; there is nothing to gain from a responsive override.
- **Never build a class with `format!()`.** Tailwind's static scanner needs
  literal class text in source. Per-variant classes are literal match arms.
- **Only `sm:` (640px) and, where genuinely needed, `md:` (768px).** No
  `lg:`/`xl:`/`2xl:`. Fewer breakpoints keeps the diff reviewable.
- **Prefer `calc(100%-2rem)` over `100vw` in clamps.** On a `fixed` or
  positioner-placed element the containing block is the viewport, so a
  percentage-based clamp is correct and — unlike `100vw` — excludes the
  desktop scrollbar, which `100vw` would otherwise turn into a new overflow
  source.

## R0b — Two canonical forms, and which to use

| | Form | Use when | Why desktop is safe |
|---|---|---|---|
| **Form C (clamp)** | `<original> max-w-[calc(100%-2rem)]` | The original fixed width already fits at 375px (≤343px) | Inert by construction above 343px — no breakpoint, no media query, provably identical |
| **Form B (breakpoint)** | `w-full max-w-[calc(100%-2rem)] sm:<original>` | The original does not fit at 375px | `sm:` restores the exact original ≥640px by winning on stylesheet order |

**Prefer Form C** — it is the stronger guarantee and the smaller diff.
Example: `popover.rs`'s `w-72` (288px) fits at 375px, so it gets
`w-72 max-w-[calc(100%-2rem)]`, not `w-full sm:w-72`.

**The discriminator that matters more than width is layout context.**
`w-full max-w-[X]` is **not** equivalent to `w-[X]` inside a flex or grid
container: `w-[X]` resolves `flex-basis: auto` to a definite width, while
`w-full` resolves to 100% of the flex container and competes with siblings
before being capped. Inside a flex/grid parent, use `w-full sm:w-[X]` instead
— it emits the byte-identical `width: X` at ≥640px regardless of siblings.
`CalendarView` is the concrete case: it's a flex child in
`date_time_picker.rs`'s `sm:flex-row` and a block child in `date_picker.rs`'s
`w-auto` popover, so the same component needs the flex-aware form to render
identically in both places.

## R1 — Fixed-position overlays need a gutter clamp

**Trigger:** an element whose base classes contain `fixed` and any width class.
**Rule:** it MUST carry `max-w-[calc(100%-2rem)]` at base; a larger base
`max-w-<size>` moves behind `sm:`.
**Corollary (directional pinning):** a `fixed` element pinned on one side
(`right-4`) must not also be `w-full` — that combination overflows off the
unpinned side. Use `w-[calc(100%-2rem)] sm:w-full` instead.
**Members:** `dialog.rs`, `alert_dialog.rs`, `toast.rs` (the directional case
— see the confirmed bug in the proposal). `sheet.rs`/`drawer.rs` already
follow this shape (`w-3/4 sm:max-w-sm`) — the model to copy.

## R2 — Anchored/portalled surfaces get a width-axis clamp

**Trigger:** an element using `max-h-[var(--adico-positioner-available-size)]`
(the positioner's height marker) or a `min-w-[…]`.
**Rule:** `packages/adico-primitives/src/positioner.rs` publishes only a
height variable — there is no width analog — so apply Form C
(`max-w-[calc(100%-2rem)]`) directly. Any `min-w-[…]` must be paired with a
`max-w` clamp, since `min-w` alone can force overflow.
**Members:** `popover.rs`, `hover_card.rs`, `tooltip.rs`, `dropdown_menu.rs`,
`context_menu.rs`, `select.rs`, `combobox.rs`, and the `min-w-[12rem]`
contents in `navigation_menu.rs`/`menubar.rs`.

## R3 — Hard-pinned intrinsic dimensions get a min/max escape

**Trigger:** `w-[<rem>]`, `h-[<rem>]`, or `size-<n≥20>` with no accompanying
`min-`/`max-`.
**Rule:** width → `w-full max-w-[<original>]` outside a flex/grid parent, or
`w-full sm:w-[<original>]` inside one (see R0b). Height → `min-h-[<X>]` where
the box is a scroll container, otherwise pair `h-[<X>]` with
`max-h-[calc(100svh-<gutter>)]`. Percentage siblings that are `shrink-0
grow-0` (e.g. calendar's `w-[60%]`/`w-[40%]`) lose `grow-0` so they can absorb
slack.
**Members:** `calendar.rs`, `time_picker.rs`'s dial, `carousel.rs`'s vertical
height, `command.rs`'s list cap, `color_picker.rs`.

## R4 — Horizontal flex must wrap or scroll

**Trigger:** a `flex`/`inline-flex` row-direction container with ≥2 children
and neither `flex-wrap` nor `overflow-x-auto`.
**Rule, in preference order:**
1. If an orientation variant already exists, switch orientation at a
   breakpoint (`button_group.rs` already ships a vertical variant).
2. If items are equal-weight and reorderable, use `flex-wrap`.
3. If order is semantic and items must stay on one line (tabs, menubar), use
   `overflow-x-auto` and drop `flex-1` from items below `sm` so they size to
   content instead of being crushed.
**Never** apply `flex-wrap` to a container whose children carry side-specific
rounded joins without also handling the corner rewrite — a wrapped join looks
broken.
**Members:** `tabs.rs`, `navigation_menu.rs`, `menubar.rs`, `button_group.rs`,
`toolbar.rs`, `toggle_group.rs`, `pagination.rs`, `data_table.rs`'s toolbar
and pagination footer, `input_group.rs`, `input_otp.rs`, `tag_group.rs`.
`breadcrumb.rs` already does this correctly (`flex-wrap break-words`).

## R5 — Fixed heights are viewport-relative

**Trigger:** `h-screen`, `max-h-screen`, or a bare `vh` unit.
**Rule:** use `svh`/`dvh`, never `vh` — mobile browser chrome makes `100vh`
taller than the visible area. `dialog.rs`'s existing height-axis treatment
(`max-h-[calc(100svh-2rem)] min-h-0` plus a scrolling body) is the model to
replicate on the width axis and elsewhere, not new invention.
**Members:** `toast.rs` (`max-h-screen`), `drawer.rs` (`max-h-[80vh]`),
`command.rs`'s list cap, and any of `virtual_list.rs`/`message_scroller.rs`/
`scroll_area.rs` found to have a bare `vh`.

## R6 — Two-column grids collapse to one

**Trigger:** `grid-cols-2`, `grid-cols-[1fr_auto]`, or a `has-[…]:grid-cols-*`
variant.
**Rule:** base becomes single-column; `sm:` restores the multi-column track.
For a `has-[…]` variant, the whole compound moves behind `sm:` (verify
Tailwind v4 actually emits the `sm:has-[…]` compound in the built stylesheet
before relying on it; fall back to a `@container` query if it doesn't).
**Members:** `card.rs`'s `CardHeader` grid, and any grid found in `alert.rs`
or `theme_builder.rs` during their audit.

## R7 — Touch targets: audit-and-record only, not fixed in this change

Every other rule adds an override that is a no-op above `sm`. Enlarging a tap
target is the opposite — it changes mobile-visible appearance and has zero
precedent in this repo (there is no `max-*` variant anywhere in `registry/`
today). Folding it into this sweep would break "every override restores
today's value or is a named exception," the property that makes the sweep
reviewable. **Decision: this change produces a findings list only** — any
interactive target below 44×44 CSS px is noted per component, with a
follow-up change owning the fix via Tailwind's `pointer-coarse:` variant
(confirmed present in the pinned v4.1.5 toolchain), not a width breakpoint.

## R8 — Composition over another component's base class needs `!`

**Trigger:** component A renders component B and must defeat one of B's
**base** (unprefixed) classes.
**Why:** `registry/lib/cn.rs`'s `cn()` is a plain space-join with no conflict
dedupe — base-vs-base ties resolve by stylesheet order, not argument order.
Mobile-first mostly sidesteps this (overrides land in a different media
block), but not when the composer needs to kill a base class outright.
`date_time_picker.rs` already documents the workaround (`w-auto!`,
`max-h-none!`, `overflow-visible!`).
**Known sites:** `date_picker.rs` (`w-auto` over `PopoverContent`'s `w-72`),
`command.rs` (`CommandDialog` over `DialogContent`).
**Rule:** if a composer must override a base class of its child, use the `!`
important form with a comment citing `cn.rs`'s non-merging behavior. If the
override only needs to apply below `sm`, prefer changing the *child's* base
class instead — cheaper, and needs no `!`. This is why overlays (Wave 1-2)
are fixed before their composers (Wave 3).
