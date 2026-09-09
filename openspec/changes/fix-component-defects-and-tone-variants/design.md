## Context

See `proposal.md` — Why, for motivation. This document covers how each of the
nine problems is fixed and the shared decisions that tie several of them
together.

Relevant current state:

- `registry/lib/variants.rs` currently exports only `Radius`; no shared trait
  or macro exists for variant enums — each component hand-rolls
  `enum XxxVariant` plus a private `fn class(self) -> &'static str`.
- `registry/lib/cn.rs:4-10` is a plain space-join, not a tailwind-merge — it
  performs no conflict dedupe, so class overrides depend on Tailwind's
  compiled source order.
- The managed theme-token region is a hardcoded string in
  `packages/adico-cli/src/css.rs`'s `theme_region()` (`:538-675`), bounded by
  `THEME_REGION_START`/`THEME_REGION_END` markers so `adico init` rewrites
  only that region without touching a consumer's other CSS.
- `registry/ui/theme_builder.rs`'s `ThemeToken` enum (28 variants today, per
  its own doc comment at `:37`) has match/field sites at `:365`, `:495`,
  `:528`, `:561`, `:594`, `:627`, `:673`, `:715`, `:1328`.
- `apps/playground/src/pages/drag_and_drop_list.rs:30` calls
  `use_drag_and_drop_list_items()` inside a children block passed to
  `DragAndDropListItems`. Dioxus builds a component's children eagerly in the
  *parent's* render scope — here, `DragAndDropListPage`'s scope — while
  `DragAndDropContext` is provided inside `DragAndDropList`'s own body
  (`packages/adico-primitives/src/drag_and_drop_list.rs:387-392`), a child
  scope. `consume_context` walks upward from where it's called, so it cannot
  find a context provided by a descendant.
- `apps/playground/src/pages/calendar.rs:99` wraps `CalendarView` (already
  `border bg-popover p-3`, `registry/ui/calendar.rs:54-58`) in `PopoverContent`
  (also `border bg-popover p-4`, `registry/ui/popover.rs:48`), with only
  `class: "w-auto p-0"` overriding the padding, not the border.
- `apps/playground/src/pages/color_picker.rs:29-40` renders `ColorArea`,
  `HueSlider`, and `ColorPickerFields` as direct children of a raw
  `PopoverContent`, not of `ColorPicker`, so `ColorPicker`'s own `gap-2`
  (`registry/ui/color_picker.rs:41-44`) never applies to them.
  `openspec/specs/adico-existing-components/spec.md:327-330` already mandates
  a `ColorPickerContent` export; it does not exist in the repo today.
- `apps/playground/src/pages/date_time_picker.rs:113-125` measures a height
  from the calendar once and applies it as an inline style to the time panel,
  then passes `fill_height: true` to `TimePickerBody`, whose `Analog` arm
  (`registry/ui/time_picker.rs:795-804`) drops `fill_height` — its own doc
  (`:783-786`) states it has no effect for `Analog`, since `TimePickerClock`
  is a fixed-size dial. `DateTimePickerContent` sets `overflow-visible!`
  deliberately (`registry/ui/date_time_picker.rs:254-262`), so the oversized
  dial is never clipped — it visibly spills out instead.
- `registry/ui/avatar.rs:31-38`'s `AvatarSize` (`Sm`/`Default`/`Lg` →
  `size-8`/`size-10`/`size-12`) already scales the root correctly.
  `AvatarFallback` (`:155-159`) carries no `text-*` class. `Avatar` and
  `AvatarFallback` deliberately share no context (`:49-52`, `:138-139`,
  documented so each can clip independently), which is why threading `size`
  into the fallback is new design work, not a one-line class addition.
- `apps/playground/src/pages/input.rs:30-32` documents the current stance:
  password reveal is a consumer-side composition, not an `Input` prop.
- `registry/ui/input_otp.rs:39-44`'s `mask: ReadSignal<bool>` already exists,
  defaulting to `false`; `apps/playground/src/pages/input_otp.rs:15,61-72`
  hand-rolls a reveal toggle at the page level.

## Goals / Non-Goals

**Goals:**
- Fix all nine reported problems with root-cause changes, not workarounds.
- Introduce exactly one new shared style vocabulary (`Tone`), reused by every
  component that has a `color` prop.
- Keep every `registry/ui/*.rs` change flowing through the full
  registry/statics/installed-copy/generated-controls pipeline so no surface
  goes stale.

**Non-Goals:**
- Extending the shape/`color` split to any presentational component beyond
  `Button`/`Badge`/`TagOption` (see proposal's "Explicitly out of scope") —
  `Kbd` is explicitly excluded per the user's direction; `Alert` and `Toast`
  keep the single-enum tone model this change already gave them (see the
  revised Decisions entry below for why this is no longer "extend `Tone` to
  every presentational component").
- Redesigning the popover/positioner primitive itself — the calendar,
  color-picker, and date-time-picker fixes work within the existing
  `Positioner`/popover contract.
- Adding new `AvatarSize` steps.
- Any desktop/mobile-native validation — no fixture exists for that surface
  in this repo (tracked gap per `docs/validation.md`).

## Decisions

**REVISED mid-implementation, superseding the paragraph below it.** The first
implementation pass built the "delegation" design directly below (`Tone`
values folded into each component's own variant enum, `Destructive` kept as
a name). After that was implemented and working, the user reviewed it and
gave an explicit, different design for `Button`, `Badge`, and `TagOption`
specifically: a **shape** enum (`Primary`/`Secondary`/`Outline`/`Ghost`/`Link`
— renamed from `Default`/`Secondary`/`Outline`/`Ghost`/`Link`, with
`Destructive`/`Verified` removed) crossed with a separate **`color: Tone`**
prop (`Default`/`Success`/`Warning`/`Error`/`Info`). This is the orthogonal
design the paragraph below explicitly rejected — superseded here because the
user's own product decision overrides the earlier judgment call, and the
"inconsistent API" cost that judgment call weighed against is smaller than
originally estimated once the shape enum drops `Destructive` entirely (there
is no orphaned variant to strand). `Alert` and `Toast` were **not** covered by
this revision — the user scoped it to `Button`/`Badge`/`TagOption` (and
explicitly excluded `Kbd`), so `AlertVariant` keeps the original delegation
model (`Default`/`Destructive`/`Success`/`Warning`/`Info` folded into one
enum) and `Toast` keeps its literal `data-[type=…]` classes, both already
shipped and left alone.

Concrete shape:
- `Tone` keeps its Rust name (not `Color`) to avoid colliding with
  `color_picker.rs`'s already-public `Color` type (an RGB value, unrelated
  concept). `Tone::Error` still resolves to the existing `--destructive` CSS
  custom property — **only the Rust variant name changes** (`Destructive` →
  `Error`); nothing in `packages/adico-cli/src/css.rs` or
  `registry/ui/theme_builder.rs` needed to change again, since the token
  itself keeps its original name.
- `Tone` grows five *shape-family* methods —
  `solid_class`/`soft_class`/`outline_class`/`ghost_class`/`link_class` —
  each a 5-arm match (one per `Tone` value) returning the **color-driven**
  portion of that shape's class. `Tone::Default` reproduces each shape's
  exact pre-existing look (e.g. `outline_class`'s `Default` arm is
  `border border-input bg-background hover:bg-accent
  hover:text-accent-foreground`, byte-for-byte what `ButtonVariant::Outline`
  already rendered), so choosing `color: Tone::Default` changes nothing
  visually for any existing shape. A non-`Default` color recolors that same
  structural shape (e.g. `outline_class`'s `Success` arm is
  `border border-success text-success hover:bg-success/10`). This keeps the
  56 literal Tailwind-scanner-visible strings (5 shapes × 5 colors, minus
  reuse) in one place instead of duplicated per component.
- Each component's own shape enum keeps only its *structural* differences
  (padding, shadow, focus-ring extras, border-transparent bookkeeping) and
  composes them with the relevant `Tone` method — e.g. `ButtonVariant` still
  owns `shadow-xs` and its `h-auto px-0 py-0` `Link` sizing; `BadgeVariant`
  still owns whether `border-transparent` is needed (every shape except
  `Outline`, since `Badge`'s base class always sets `border`).
- **One narrow, deliberate exception to "`Default` reproduces the exact
  pre-existing look":** `Button`'s and `Badge`'s existing `Outline` shapes
  structurally diverge (`border-input`/`bg-background` vs. `border-border`/
  `bg-transparent`/`text-foreground`) even though every other shape (solid,
  soft, ghost, link) turns out to be byte-identical between the two once each
  component's own structural extras — `shadow-xs`, `border-transparent`,
  `dark:hover:bg-accent/50`, `h-auto px-0 py-0` — are factored out into that
  component's own code rather than `Tone`'s shared methods. Sharing one
  `Tone::outline_class()` across both means picking one shape; `Button`'s
  (`border border-input bg-background hover:bg-accent
  hover:text-accent-foreground`) is kept as the canonical `Default` arm, so
  `Badge`'s `Outline` shape's un-selected look changes slightly. This is
  accepted as being in the spirit of the user's actual ask — one canonical
  variant *look*, not just one canonical variant *name list* — rather than
  re-litigated as a blocking question, since it is the only such divergence
  found and is visually minor (both read as "a bordered, unfilled surface").
- `TagOption` is the one place this crosses into pre-existing per-instance
  state: its `data-[selected=true]:` fill already existed
  (`tone_selected_class`, from the delegation pass) and only ever styled the
  *selected* state, leaving the resting state a fixed neutral
  `bg-secondary`/`text-secondary-foreground` regardless of tone. That split is
  kept: `TagOption` gains `variant: TagOptionVariant` (a new enum, since no
  shape enum existed before) and `color: Tone`, both of which affect only the
  selected-state look; the resting look is unchanged. Because Tailwind's
  scanner needs the literal `data-[selected=true]:` prefix on each class, this
  needed its own 25-arm table (5 shape-family functions × 5 `Tone` arms) local
  to `tag_group.rs`, mirroring `Tone`'s own methods rather than reusing them
  directly (their un-prefixed output isn't reusable through a
  `data-[selected=true]:` wrapper without becoming a runtime-formatted string,
  which Tailwind's static scanner cannot see).

*(Superseded) original decision, kept for the record:* `Tone` is adopted by
delegation, not as an orthogonal `tone:` prop. `Tone::Success.surface_class()`
(and its siblings) live in one place in `registry/lib/variants.rs`;
`BadgeVariant::class()`, `AlertVariant::class()`, etc. delegate to it for the
tone-shaped variants while keeping their own variant names — critically,
keeping `Destructive` as the name for the error tone rather than introducing a
same-meaning `Tone::Error` that would either duplicate `Destructive` or force
it out of the existing enums (a breaking rename). This mirrors how `Radius`
already works: one shared mapping, consumed positionally by every component's
own `cn(&[...])` call. *Alternative considered and rejected (at the time)*: an
orthogonal `tone: Tone` prop alongside each component's existing `variant:`
prop. Rejected because it either strands `Destructive` as a "variant" with no
matching "tone" (an inconsistent API), or requires migrating every
`Destructive` call site to the new prop — a breaking rename with no
behavioral benefit. This is the exact design the revision above adopts for
`Button`/`Badge`/`TagOption` after the user's explicit direction; `Alert` was
never revisited, so this original reasoning still stands as its rationale.

**Tone tokens are real theme tokens, not fixed palette colors.** Consistent
with every existing role token (`--primary`, `--destructive`, etc.), so the
new tones follow a consumer's chosen palette and light/dark theme instead of
being hardcoded Tailwind colors like `Badge::Verified`'s current
`emerald-600`. *Alternative considered and rejected*: fixed palette colors
(what `Badge::Verified` and `Toast` already do). Rejected per the user's
explicit choice — fixed colors would perpetuate the exact defect (theme-blind
presentational components) this change exists to fix.

**`Badge::Verified` and `Toast`'s hardcoded tone colors are rerouted onto the
new tokens, reversing a previously recorded design decision.**
`statics/styling_usage/badge.json` records an explicit rationale for
`Verified`'s fixed emerald: "a fixed, palette-independent success green so
the checkmark badge reads consistently across every theme." This change
reverses that call — once `--success` exists as a real theme token, a
"verified" look SHOULD track the consumer's theme like every other semantic
surface, and the inconsistency of one badge variant refusing to theme while
all others do is worse than the green shifting slightly across palettes.
**Superseded detail**: with the shape/`color` revision above, `Verified` is
not merely *rerouted* onto `--success` under its old name — it is removed
from `BadgeVariant` entirely (per the user's explicit choice), and the same
visual is now reached via `variant: Primary, color: Tone::Success`. The
`styling_usage` record's reason text is updated to describe this, not just
its pass/fail boolean. Separately, `statics/styling_usage/toast.json`
already claims `tokenCompliant: true` with an empty `colorException`, despite
`registry/ui/toast.rs:45` containing `border-emerald-500/50` and
`border-amber-500/50` today — that record is already wrong; implementation
must investigate why `styling-usage check` didn't catch it (task 7.11) before
trusting a regenerated pass.

**The drag-and-drop hang is fixed by giving the playground page its own
descendant-scope component, not by changing the primitive's context
resolution.** The primitive's context boundary (provider inside
`DragAndDropList`, consumed by descendants) is correct and is exactly what
the default-children path already relies on
(`packages/adico-primitives/src/drag_and_drop_list.rs:456-476`) and what the
passing installation fixture uses. The bug is the playground page calling the
hook from its own (ancestor) scope. *Alternative considered and rejected*: add
a `try_use_drag_and_drop_list_items()` that returns `None` instead of
panicking outside the provider. Rejected as the default choice because it
would hide the real defect (calling a context hook from the wrong scope)
behind a silently-empty list rather than fixing the call site; the primitive
API changes only if task 2.2's reproduction shows the panic is unavoidable
from any page-level composition Dioxus permits.

**The Calendar and ColorPicker popup-frame fixes strip the outer
`PopoverContent`'s own border/background rather than only its padding,
following `DatePickerContent`'s existing plain-utility pattern
(`registry/ui/date_picker.rs:241`: `w-auto border-0 bg-transparent p-0
shadow-none`) rather than `DateTimePickerContent`'s `!`-override pattern
(`registry/ui/date_time_picker.rs:254-262`).** Both patterns exist in the repo
today for the same underlying problem (`cn`'s lack of conflict dedupe,
`registry/lib/cn.rs:4-10`). The plain-utility form is preferred here because
neither Calendar nor ColorPicker's inner content needs to *override* a
class that would otherwise win by source order — it needs the popover frame's
classes simply not applied, which `date_picker.rs` already demonstrates
working correctly. `date_time_picker.rs`'s `!` overrides remain as they are
(they solve a different problem: overriding `w-72` with `w-auto` on the same
element). *Alternative considered and rejected*: unify all three components
onto the `!`-override pattern for consistency. Rejected as unnecessary
churn — `date_time_picker.rs`'s use of `!` is justified by its own comment for
a reason that doesn't apply to Calendar or ColorPicker's simpler "don't
double the frame" fix.

**Avatar's fallback-text scaling requires an `AvatarContext`, introduced as
new shared state between `Avatar` and `AvatarFallback`.** The two currently
share no context by design, each clipping independently
(`registry/ui/avatar.rs:49-52`, `:138-139`). Scaling the fallback's text with
the root's `size` needs the fallback to know the root's size, which requires
either a new prop threaded by the consumer on both `Avatar` and
`AvatarFallback` (mirroring how `radius` is already threaded twice), or a
context. A context is chosen because it does not require every existing
`AvatarFallback` call site to be touched to opt in — the scaling should be
automatic. *Alternative considered and rejected*: a second `size` prop on
`AvatarFallback`, mirroring `radius`'s existing pattern. Rejected because
`radius`'s dual-prop pattern exists so each part can clip *independently*
(a real, load-bearing design choice); text scaling has no such reason to be
settable independently of the root's size, so defaulting it via context is
strictly better ergonomics with no loss of flexibility (a consumer can still
override via `class`).

**Input's password reveal is added directly to `Input`, changing its DOM
shape only when `r#type == "password"`.** `Input` currently renders a bare
`<input>` (`registry/ui/input.rs:56-66`); a reveal toggle needs a wrapper
element. Making the wrapper conditional on `r#type` keeps every non-password
`Input` byte-identical to today's output, satisfying the "Input reveals a
password value… For every other `r#type` value… unchanged" requirement.
*Alternative considered and rejected*: always wrap `Input` in the reveal
structure regardless of type, hiding the toggle via CSS when not a password
field. Rejected because it changes the DOM shape (and thus any consumer CSS
targeting `Input`'s root as a bare `<input>`) for every consumer, not just
password fields — a larger, unnecessary compatibility break.

## Risks / Trade-offs

- **[`cn` has no conflict dedupe]** → every class override in the Calendar,
  ColorPicker, and DateTimePicker fixes depends on Tailwind's compiled source
  order, not attribute order. Mitigation: verify each fix by rendering in the
  browser, not by reasoning about class order in isolation; prefer the
  simplest form (omit the conflicting class rather than override it) per the
  Decisions above.
- **[Token rollout touches 23 checked-in `tailwind.css` files and 9
  `theme_builder.rs` sites]** → high mechanical surface area for a small
  logical change. Mitigation: implement `css.rs`'s `theme_region()` first,
  then refresh every `tailwind.css` file through the real `adico init` /
  fixture-refresh tooling rather than hand-editing each one.
- **[The Input OTP default flip is behavior-breaking]** → any existing
  consumer's installed `InputOTP` that relies on values being visible by
  default changes behavior once they reinstall via `adico add --replace`.
  Mitigation: name it explicitly in `proposal.md`'s Impact section and in the
  MODIFIED requirement's restated text; this registry ships source, so
  already-installed copies are unaffected until a consumer opts into the
  update.
- **[The Input password-reveal change reverses a documented design
  decision]** → `apps/playground/src/pages/input.rs:30-32`'s stance
  ("composition, not a prop") was deliberate, and moving it into the registry
  brushes `adico-playground-structure:131` ("Registry components are never
  modified solely for playground's convenience"). Mitigation: the change is
  justified on consumer merit (a consumer wants `type="password"` to just
  work) independent of the playground demo; the demo simplification is a
  downstream consequence, not the reason for the registry change.
- **[The drag-and-drop root cause is confirmed by static analysis, not yet by
  a live reproduction]** → tasks.md's group 2 opens with a reproduction step
  before any fix lands, so the chosen fix is validated against the actual
  failure mode (wasm panic vs. render loop) rather than assumed.
- **[The revised shape/`color` split is a breaking rename for `Button` and
  `Badge`, reversing this document's own original "no renames, no removed
  variants" goal]** → `ButtonVariant`/`BadgeVariant::Default` becomes
  `Primary`; `Destructive` is removed from both (reachable via
  `color: Tone::Error` on any shape instead); `BadgeVariant::Verified` is
  removed outright. Every existing call site of these four identifiers
  breaks at compile time for any consumer who reinstalls. Mitigation: this is
  an explicit, informed user decision (not a default this change chose on its
  own), it is confined to three components (`Button`/`Badge`/`TagOption`,
  not `Alert`/`Toast`/`Kbd`), and — as with every other change in this
  registry — it only reaches a consumer's installed copy when they explicitly
  run `adico add <item> --replace`; nothing changes for an already-installed
  consumer otherwise. Named explicitly in `proposal.md`'s Impact section.
- **[`TagOption` needs a 25-arm literal table it cannot share with `Tone`'s
  own methods]** → Tailwind's static scanner requires the literal
  `data-[selected=true]:` prefix on each class string; `Tone`'s own
  `solid_class`/etc. return unprefixed strings, and wrapping them at runtime
  (`format!("data-[selected=true]:{...}")`) would be invisible to the
  scanner. Mitigation: a small local table in `tag_group.rs`
  (`selected_solid_class`/`selected_soft_class`/etc.), structurally mirroring
  `Tone`'s methods one-for-one so the two stay easy to keep in sync by eye,
  covered by a test asserting every `(shape, color)` pair resolves to a
  string containing that color's token name.

## Open Questions

- Whether the drag-and-drop fix stays confined to
  `apps/playground/src/pages/drag_and_drop_list.rs` or also requires a change
  to `packages/adico-primitives/src/drag_and_drop_list.rs` (e.g. a
  `try_`-context variant) depends on task 2.2's reproduction. If the primitive
  changes, a delta for `adico-primitives`'s existing drag-and-drop
  requirements is added at that point; this does not change the chosen
  approach (fix the call site) or any other task, only whether one additional
  spec file is needed.
- Whether `InputOTP`'s `mask` default belongs at the registry facade
  (`registry/ui/input_otp.rs`) or should also flip at the primitive layer
  (`packages/adico-primitives/src/otp_field.rs`, spec'd at
  `adico-primitives:385`) is a task-2.10-time investigation; either way the
  observable default behavior specified in this change's MODIFIED
  requirement is the same.
