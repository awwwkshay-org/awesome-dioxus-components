## Context

See `proposal.md` - Why. Relevant existing mechanics:

- `packages/adico-xtask/src/playground_controls.rs` already introspects each
  playground UI file and emits one independent `<Component>DemoState` +
  `<Component>Controls` panel per component that has qualifying props
  (`render_controls_component`, `render_component_file`). There is no
  merging step anywhere in this pipeline — `generated/controls/select.rs`
  already ships separate `SelectDemoState`, `SelectMultiDemoState`, and
  `SelectTriggerDemoState`. The generator today skips a component with zero
  qualifying fields entirely (`if fields.is_empty() { continue; }`) —
  **this is the behavior being changed**: instead of skipping, it now emits
  an empty-state panel for that component.
- `rust_introspect.rs` today only knows about a component **defined locally**
  in the file it parses (`pub struct *Props` + paired fn, or an inline-arg
  `#[component] pub fn`). It has no knowledge of a `pub use
  adico_primitives::some_module::{...};` re-export line — its own doc
  comments name this an accepted gap ("never follows a re-export across the
  crate boundary"). This is a real, common pattern, not an edge case: a
  file's own root component is frequently a bare re-export with no local
  wrapper at all. Confirmed concretely:
  - `registry/ui/accordion.rs:6` — `pub use adico_primitives::accordion::{Accordion, AccordionMulti};`
    — `Accordion` itself has never been introspectable; it gets no group
    today except via a page's own hand-written wrapper.
  - `registry/ui/dialog.rs:8-10` — `pub use adico_primitives::dialog::{DialogContent as DialogPrimitiveContent, DialogDescription, DialogRoot as Dialog, DialogTitle};`
    — a **renamed** re-export (`DialogRoot as Dialog`): the local, public name
    (`Dialog`) differs from the name in its defining module (`DialogRoot`).
    `DialogContent as DialogPrimitiveContent` is part of the *same* `pub use`
    block (confirmed by reading the file directly), so it genuinely is part
    of `dialog.rs`'s public surface — its own `DialogContent` (the styled
    facade) is a separate, locally-defined component, non-conflicting only
    because the raw primitive is imported under this alias. It correctly
    gets its own `DialogPrimitiveContentControls` group. `registry/ui/drawer.rs`
    is the file with the *private* `use adico_primitives::dialog::{DialogContent
    as DialogPrimitiveContent, DialogCtx};` (no `pub`) — that one is correctly
    excluded by construction, since a `pub`-aware scan never sees it at all.
  - `registry/ui/checkbox.rs`, `registry/ui/slider.rs`, `registry/ui/toast.rs`
    re-export `CheckboxState` (an enum), `SliderProps`/`RangeSliderProps`
    (Props structs), and `use_toast`/`consume_toast`/`ToastOptions`/`ToastType`
    (hooks and types) — real `pub use` re-exports, but none are a renderable
    component.
  - `packages/adico-primitives/src/` is a flat module layout: every
    `pub mod <name>;` in its `lib.rs` maps directly to
    `packages/adico-primitives/src/<name>.rs` (confirmed — e.g.
    `accordion.rs`, `aspect_ratio.rs` both exist as plain files; no nested
    `mod.rs` subdirectories exist except an unrelated `js` dir). Every
    registry re-export path found across the registry
    (`grep -rhoE 'adico_primitives::[a-z_]+::' registry/ui/*.rs`, 42 distinct
    module prefixes) uses this direct submodule form, so resolving one is a
    reliable single-hop lookup: take the module segment right after
    `adico_primitives::`, load that one file, and look up the (possibly
    renamed) item there. The sole exception found is a bare crate-root path
    with no module segment, `adico_primitives::ContentAlign`/`ContentSide`
    (re-exported identically by `popover.rs`, `hover_card.rs`,
    `navigation_menu.rs`, `tooltip.rs`) — these have no submodule to resolve
    through, so the lookup needs a documented fallback: check
    `packages/adico-primitives/src/lib.rs` itself. This particular case
    resolves to an enum either way, so it is excluded regardless — the
    fallback exists so the resolver never mis-treats a bare-path name as
    "unresolvable" (and thus a hard error) when it is simply one hop shorter.
- `apps/playground/src/components/demo.rs`'s `Demo` component renders its
  entire `controls: Option<Element>` prop into one anonymous
  `div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" }`.
  This is the only place flattening happens, and it has no notion of
  grouping.
- `apps/playground/src/components/controls.rs`'s module doc declares its five
  control components' signatures frozen, because generated code emits calls
  against them exactly (`cargo xtask playground-controls sync`).
- A playground page composes root props, generated subcomponent panels, and
  hand-written demo-scenario toggles into that single flat `controls: rsx!{}`
  block today (`pages/select.rs`, `pages/accordion.rs`, `pages/sidebar.rs`).
- Naming conventions cannot be used to infer a subcomponent's parent: `Tabs`
  owns `TabList`/`TabTrigger`; `AccordionMulti`/`SelectMulti` are sibling
  root variants, not parts of `Accordion`/`Select`; `Drawer`/`Sheet` are
  aliased re-exports of the dialog primitive defined in a different file.
  Registry metadata (`registry/schema.json`) records no per-item component
  list. Group labels are therefore always the literal, already-known
  component name at the call site — never inferred.

## Goals / Non-Goals

**Goals:**
- Every control panel — generated or hand-written — visibly attributes each
  control to the exact component that declares the underlying prop.
- Generated panels become self-labeling with zero page-level changes.
- Every real component a page actually renders is represented in the control
  panel, even one with zero controllable props — nothing in the demoed
  composition is invisible.
- A component that is only reachable through a re-export from
  `adico_primitives` is discovered and treated the same as one defined
  locally, provided it resolves to a real component.
- The rollout to the remaining, unconverted pages introduces no visual
  regression while it is pending.

**Non-Goals:**
- No change to `Demo`'s own props or its outer grid — grouping composes
  around it, not through it.
- No change to `classify_prop_type`'s allowlist (e.g. recognizing
  `adico_primitives::ContentAlign`) — that is a prop-coverage gap, orthogonal
  to discovering and labeling components.
- No attempt to infer parent/child relationships from naming or file layout —
  a label always names the exact component already available at each call
  site, never a different, inferred one. It is rendered as space-separated
  Title Case for readability (e.g. `AccordionItem` → "Accordion Item"), but
  that is a display transform of the same identifier, not a substitution for
  it — see the new "Component group labels are humanized" decision below.
- No requirement that a page render a group for a component the registry
  file exposes but the page's own composition never renders (e.g.
  `select.rs`'s re-exported `SelectGroup`/`SelectGroupLabel`/
  `SelectItemIndicator`, which the `/select` page's rsx never composes). This
  keeps the existing, unmodified `adico-playground-structure` requirement
  that a demo page renders only its real composition, and that a generated
  panel with no realistic binding target is omitted rather than forcing an
  invented element — this change makes *more* panels available to wire in,
  it does not change which ones a page is obligated to use.
- Converting all ~53 pages is out of scope for this change; 5 pilot pages
  prove the pattern, the rest are an explicit follow-up.

## Decisions

**Add one new component, `ControlGroup`, rather than changing the five
existing control components or `Demo` itself.** `controls.rs`'s five
signatures are declared frozen because generated code depends on them
exactly; changing any of them would force touching all 46 generated files
and every page unnecessarily. `Demo` itself needs no change: `ControlGroup`
renders `col-span-full`, occupying a full row of `Demo`'s existing 1/2/3-column
outer grid, with its own identical inner grid for the controls it wraps.
Alternative considered: change `Demo`'s `controls` prop to accept a list of
`(part, Element)` groups instead of one opaque `Element`. Rejected — it forces
every one of the ~53 pages that pass `controls:` to change shape in this one
change, which is exactly the large blast radius the phased pilot rollout is
meant to avoid. The `col-span-full` approach lets converted and unconverted
pages coexist with no visual difference for the unconverted ones.

**The generator wraps every emitted panel in `ControlGroup { part: "<Humanized Component Name>" }`.**
The component name is already the loop variable the generator iterates over
(`render_component_file`'s `for component_name in &introspection.components`),
so no new lookup or metadata is needed beyond humanizing it for display (see
next decision). Regenerating (`playground-controls sync`) immediately labels
all 46 generated files.

**Component group labels render as space-separated Title Case, reusing the
existing enum-variant-label humanizer — renamed, not duplicated, for its
broadened role.** `playground_controls.rs` already has
`humanize_variant_label(ident: &str) -> String`, used today only to turn an
enum variant identifier into an option label (`IconXs` → `Icon Xs`). Its
algorithm has no variant-specific logic at all — it mechanically inserts a
space before an uppercase letter that follows a lowercase one (or follows an
uppercase-then-lowercase run) — so it is exactly the right transform for a
component name too, verified by tracing it by hand: `AccordionItem` →
`Accordion Item`, `SelectItemIndicator` → `Select Item Indicator`,
`DialogPrimitiveContent` → `Dialog Primitive Content`. Rather than duplicate
this logic under a second name, the function is renamed to
`humanize_pascal_case_label` (its actual, general contract) and reused at
both call sites: the existing enum-options loop, and the two new
`ControlGroup { part: "..." }` emission points in `render_controls_component`
and `render_empty_controls_component`. Alternative considered: keep the name
`humanize_variant_label` and just apply it to component names too. Rejected
— a future reader hitting a call like `humanize_variant_label(component_name)`
would reasonably wonder why a component name is being treated as a variant;
the rename is small (one definition, one pre-existing call site, five
pre-existing unit tests) and removes that confusion at its source. Hand-written
`ControlGroup { part: "..." }` calls on playground pages cannot call this
function themselves — it is a build-time-only helper in a different crate
(`adico-xtask`), never a runtime dependency of `adico-playground` — so a page
author writes the already-humanized string literally (e.g. `"Select List"`,
not `"SelectList"`), matching the same convention by hand.

**Component discovery is widened to include resolvable re-exports, not just
local definitions.** `rust_introspect` gains a step that walks each file's
`pub use adico_primitives::<module>::{...}` items (via `syn`, handling a
`UseRename` — `X as Y` — the same way `Dialog`'s `DialogRoot as Dialog`
requires) and, for each named item, loads
`packages/adico-primitives/src/<module>.rs` and classifies the *original*
name there using the exact same recognizer `rust_introspect` already applies
to local files: a `pub struct <Name>Props` with a paired `pub fn <Name>`, or
an inline-arg `pub fn <Name>(...) -> Element`, counts as a component; a plain
`pub struct`/`pub enum`, or a `pub fn` that isn't in this shape (a hook),
does not. Reusing the existing recognizer means no new, separate notion of
"what counts as a component" is introduced — a re-exported component is
judged by literally the same rule as a locally-defined one. A bare
crate-root path (no module segment, e.g. `adico_primitives::ContentAlign`)
falls back to checking `packages/adico-primitives/src/lib.rs` directly.
Alternative considered: a naming-convention heuristic (skip snake_case names
and names ending in `State`/`Props`/`Options`/`Type`/`Context`). Rejected —
it would silently misclassify a future export that doesn't happen to follow
today's naming pattern, where the resolution-based approach stays correct by
construction.

**An unresolvable re-exported name is a hard `sync`/`check` failure, never a
silent skip.** If a `pub use adico_primitives::<module>::{Name}` doesn't
correspond to anything found in the resolved module (renamed, moved, or
deleted upstream), that is treated the same way the generator already treats
every other never-silently-drop case (`PropShape::Skipped` reasons are always
printed, never swallowed): `sync` and `check` both fail loudly, naming the
unresolved item, rather than proceeding as if the component didn't exist.

**Every discovered, resolvable component gets a `ControlGroup`, including one
with no controllable props.** This reverses the previous decision. Instead of
skipping a component with zero qualifying fields, the generator now still
emits `ControlGroup { part: "<ComponentName>" }`, with a plain
`"No adjustable props."` line as its body — this is a literal string in the
generated output, not a new component signature; `ControlGroup`'s own
`(part, children)` shape is unchanged. Because every call site decides for
itself what to put inside a `ControlGroup`, there is no separate "does this
group have anything in it" check to get wrong. Hand-written pilot-page groups
follow the same rule directly: a page never wraps a truly empty
`ControlGroup` in hand-written code (there is nothing to auto-detect at
runtime — see Risks), it explicitly writes the same empty-state line when
that is the accurate statement to make.

**A component that gets both a generated panel and page-level hand-written
extra controls under the same name renders as two adjacent, identically
labeled groups — accepted, not merged.** `pages/accordion.rs`'s "Allow
multiple open" toggle (a demo-scenario switch, not a real prop) is
hand-written and grouped under `ControlGroup { part: "Accordion" }` per the
existing decision below. Once `Accordion` becomes introspectable via
re-export resolution, the generator *also* emits its own
`ControlGroup { part: "Accordion" }` (real props if any exist, otherwise the
empty-state line). These are two separate `Element`s composed by the page,
not one merged group — merging them would mean threading extra children
through the generated panel's own function signature, which is out of scope
here and unnecessary: two boxes both correctly labeled "Accordion" is
redundant-looking but not incorrect, since every control under either box
genuinely is Accordion's. No new mechanism is built to avoid this; it is
named explicitly here so it isn't mistaken for a bug during review.

**Non-prop demo-scenario toggles are grouped under the root component's own
group**, per the proposal's decision — for example Accordion's "Allow multiple
open" switch (between `Accordion` and `AccordionMulti`) renders inside
`ControlGroup { part: "Accordion" }`, not in its own group and not left
ungrouped. This keeps every control attributed to *some* real, named
component, even when the control itself isn't literally one of that
component's props.

**Import-set safety for the generator's `use crate::components::controls::{...}`
line.** That emitter already always writes the braced form regardless of how
many names it contains, and the entire generated file is piped through
`rustfmt` before being written — the doc comment on that step states rustfmt
is the deliberate authority on import brace style, not the generator itself.
Growing the imported set from one control (e.g. `NumberControl` alone in
`accordion.rs`) to two (adding `ControlGroup`) requires no special-casing.
This is verified against the current source, not assumed; a test case with a
single-leaf-control file is added to lock it in.

**This does not weaken the existing "generated output cannot silently go
stale" guarantee.** That guarantee is enforced today by
`const _: () = { fn _exhaustive(...) }` exhaustiveness guards over enum
variants. `ControlGroup`'s `part` argument is a string literal with no
equivalent compile-time exhaustiveness check, so renaming a component doesn't
trip a guard of that shape. It still cannot silently go stale, for a
different, sufficient reason: renaming a component also renames its
generated `<Component>Controls` function, which breaks every page importing
the old name at compile time. No new machinery is introduced to preserve
this; it's a byproduct of names already being load-bearing. This same
guarantee now additionally covers re-exported components: an unresolvable
re-export is its own hard failure (see Decisions above), and a *resolved*
re-export that later gets renamed upstream (in `adico_primitives`) becomes
unresolvable in exactly the same way, so it cannot silently go stale either.

## Risks / Trade-offs

- **[Partial rollout leaves 21 pages' hand-written controls unlabeled]** →
  Mitigated two ways: the `col-span-full` layout choice means every
  unconverted page keeps rendering in the current flat grid with no visual
  regression, and — confirmed live during this change's own verification —
  a page with *no* hand-written controls of its own already gets its
  generated panel's label for free on regeneration (e.g. `/switch`), so the
  real remaining gap is smaller than "every unconverted page." The proposal
  names the follow-up explicitly rather than silently dropping it.
- **[A future page could hand-write a `ControlGroup` with no children, e.g.
  after removing its last control but forgetting to write the empty-state
  line]** → Not compile-time preventable — a rendered `Element` cannot be
  inspected for emptiness at runtime, so there is no way to make
  `ControlGroup` self-detect this. Caught the same way any dead or misleading
  markup is caught today: code review, and the live-render verification step
  in `tasks.md`.
- **[Group labels are free-text strings, not validated against real component
  names]** → A typo'd label wouldn't fail any build. Mitigated in the
  generator's case because the label is mechanically humanized from the same
  `introspection.components` value already used to name the generated
  function and struct (via `humanize_pascal_case_label`), so it cannot drift
  from them. Pilot pages are hand-checked against the actual composed
  component names — and against the same humanization convention — during
  review and live verification.
- **[Cross-crate resolution could misclassify a component authored in a
  shape `rust_introspect` doesn't yet recognize]** → Mitigated by reusing the
  *exact same* recognizer already exercised against every local registry
  file, rather than inventing a second, re-export-specific one — any gap in
  it is a pre-existing gap, not new surface area from this change. Covered
  by the new resolution unit tests in `tasks.md` (component via inline fn,
  component via Props-struct pattern, struct/enum rejection, hook rejection,
  renamed re-export, bare crate-root fallback).
- **[Widening discovery could surface many more per-file components than
  before, inflating unrelated generated files with propless groups that no
  page ever wires in]** → Accepted, not mitigated further: an unused
  generated group costs nothing at runtime (Dioxus components are
  effectively free until instantiated) and the existing, unmodified
  "demo pages render only their real composition" requirement already stops
  a page from being *forced* to display one. This is why that requirement is
  called out as unchanged in Non-Goals rather than revisited here.
