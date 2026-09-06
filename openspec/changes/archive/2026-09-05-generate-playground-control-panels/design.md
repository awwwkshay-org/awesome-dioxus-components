## Context

See `proposal.md` — Why. Builds directly on the archived
`generate-playground-controls-from-props` change's design.md, which
already established and this change does not re-litigate:
- `apps/playground/src/components/ui/*.rs` is a byte-identical,
  CLI-installed copy of `registry/ui/*.rs`, a real compiled workspace
  member.
- Generated output lives in `apps/playground/src/generated/controls/`
  (not `registry/generated/` or `statics/`) specifically so a stale
  generated file fails `cargo check --locked --workspace` via a
  compile-time exhaustiveness guard, not only this tool's own `check`.
- Dioxus RSX component invocations are statically typed — there is no
  supported way to drive a component field from a runtime-typed value, so
  generated *compiled* Rust is the only viable mechanism for a live demo
  control, not a design choice made fresh here.
- `rust_introspect.rs`'s enum extraction (variants + `#[default]`) and
  `playground_controls.rs`'s fixed prop-to-control allowlist/skip-with-reason
  posture stay as-is; this change extends the allowlist, not the posture.

What's measured fresh for this change (the archived change deferred these
exact numbers):
- 7 of 66 items have no playground page (`attachment`, `bubble`,
  `data-table`, `marker`, `message`, `message-scroller`, `theme-builder`);
  6 of those 7 aren't even installed into
  `apps/playground/src/components/ui/` yet (`theme-builder` is installed,
  just has no page).
- Of the 59 existing pages, 23 render zero live controls (listed in
  proposal.md); the other 36 hand-write their own wiring.
- `apps/playground/src/components/controls.rs` has an API asymmetry:
  `BoolControl`/`TextControl` take `Signal<T>` (two-way binding a
  generator can drive uniformly); `SelectControl` takes `value` +
  `on_change` (one-way) — a generator has to emit one calling convention.
- 16 of 66 items expose exactly one public, non-generic root component
  (`aspect-ratio`, `badge`, `button`, `input`, `label`, `mode-toggle`,
  `progress`, `scroll-area`, `skeleton`, `spinner`, `switch`, `textarea`,
  `theme-builder`, `theme-switcher`, `toggle`, `virtual-list`) — measured
  by counting each item's `pub fn <Component>(...)` declarations plus
  single-identifier `pub use module::Name;` re-exports, verified by hand
  for two ambiguous cases (`collapsible`'s `pub use ...::{Collapsible,
  CollapsibleContent, CollapsibleTrigger}` is genuinely 3 parts;
  `scroll-area`'s `pub use ...::{ScrollArea, ScrollDirection,
  ScrollType}` is 1 real component plus 2 non-component enum re-exports).

## Goals / Non-Goals

**Goals:**
- Reach 66/66 pages with a generated `DemoState`/`Controls` pair, so a
  future prop change (Change B's waves, or any later one) can't silently
  leave a page's controls stale the way `Badge`/`Item` already did once.
- Generate a live `Preview` wherever the composition is genuinely
  inferable from a single props struct (16 items); never guess a
  multi-part or generic composition.
- Keep every generated artifact inside a real, compiled workspace member,
  so drift is a compile failure, not only a `check` failure.
- Give the same introspected prop metadata a second, generic consumer
  (`apps/docs`'s props table), proving it isn't playground-specific data.

**Non-Goals:**
- No runtime/JSON-driven control renderer (restated from the archived
  change; still true, for the same static-typing reason).
- No generated page/route — `pages/`'s file-naming-doesn't-auto-route
  rule stays; every page still hand-wires its generated panel in, the
  same way it would hand-write it today, just without inventing the
  wiring itself.
- No generated `Preview` for any of the 50 multi-part/generic items —
  their previews stay hand-written, same discipline as an unsupported
  prop shape: excluded with a recorded reason, not silently skipped.
- Not a redesign of `playground_controls.rs`'s classification posture
  (fixed allowlist, skip-with-reason) — this change adds shapes to the
  allowlist, it doesn't change the mechanism.

## Decisions

### D1: Two new control primitives; unify `SelectControl` onto `Signal<T>`

`NumberControl(label, value: Signal<f64>, min: Option<f64>, max: Option<f64>,
step: Option<f64>)` — a bound `<input type="number">`, the shape
`Slider`/`Progress`/`Resizable`'s numeric props need and the reason they
have no live controls today (`playground_controls.rs`'s own
`classify_prop_type` already special-cases numeric types as `Skipped`).

`OptionalBoolControl(label, value: Signal<Option<bool>>)` — a 3-way
`<select>` (`Uncontrolled` / `On` / `Off`) for the
`Option<ReadSignal<Option<bool>>>`-shaped optional-controlled idiom
`pages/sidebar.rs`/`pages/select.rs` already hand-write; generalizing the
existing hand-rolled pattern into a reusable control rather than
reinventing it per page.

`SelectControl<T>`'s signature changes from `(label, value: T, options,
on_change: EventHandler<T>)` to `(label, value: Signal<T>, options: &'static
[(&'static str, T)])`, matching `BoolControl`/`TextControl`'s calling
convention. Every existing hand-written call site using the old signature
is updated in the same change (Task 4), since a generator can only emit
one convention.

### D2: Generated `DemoState`/`Controls`/`Preview`, following the exhaustiveness-guard precedent

Per component with at least one controllable prop (per the extended
allowlist), `playground_controls.rs` emits into
`apps/playground/src/generated/controls/<item>.rs`:

```rust
pub struct ButtonDemoState { pub variant: ButtonVariant, pub size: ButtonSize, pub disabled: bool }
impl Default for ButtonDemoState { fn default() -> Self { Self { variant: ButtonVariant::Default, size: ButtonSize::Default, disabled: false } } }

#[component]
pub fn ButtonControls(state: Signal<ButtonDemoState>) -> Element {
    rsx! {
        SelectControl { label: "Variant", value: /* bound sub-field signal */, options: BUTTON_VARIANT_OPTIONS }
        SelectControl { label: "Size", value: /* ... */, options: BUTTON_SIZE_OPTIONS }
        BoolControl { label: "Disabled", value: /* ... */ }
    }
}
```

`Default` for an enum-typed field uses the enum's own `#[default]`
variant (already extracted); for `bool` it's `false`; for `Option<String>`
it's `None`. A per-field bound sub-signal is generated via `Signal::map`/a
small generated setter, following whichever pattern
`Signal<T>` composition in this Dioxus version actually supports cleanly
— resolved as an implementation detail of Task 2, not a proposal-level
decision, since it doesn't change the generated *public* shape above.

For the 16 single-root, non-generic items, also emit:

```rust
#[component]
pub fn ButtonPreview(state: ButtonDemoState, children: Element) -> Element {
    rsx! { Button { variant: state.variant, size: state.size, disabled: state.disabled, {children} } }
}
```

Keeps the existing `const _: () = { fn _exhaustive(...) }` guard per
enum-typed field and `format_rust_source()` (rustfmt-canonicalized output)
unchanged.

### D3: `Preview` generation stops at exactly the 16 measured single-root, non-generic items — never a heuristic guess for the rest

Rejected: attempting to infer a multi-part composition (e.g. guessing that
`Dialog` = `DialogTrigger` + `DialogContent` in some default arrangement)
from the props structs alone. `pages/dialog.rs`'s paired `open`/
`on_open_change` state and `pages/button.rs`'s page-local `ButtonContent`
enum (varies `children`, not a prop) are exactly the kind of bespoke
per-page composition knowledge the archived change's design.md already
identified as not inferable from a Props struct — that reasoning is
unchanged by anything in this change. The 50 multi-part/generic items get
a generated `DemoState`/`Controls` (still real signal) but keep their
`Preview` hand-written, with the specific reason ("multi-part composition"
or "generic type parameter") recorded in this change's task list per
item, mirroring `classify_prop_type`'s existing `Skipped(reason)`
discipline at the whole-component level instead of the per-prop level.

### D4: Page conversion order — install missing items first, uncontrolled pages next, then hand-written-control pages

1. Install the 6 not-yet-installed items via `adico add` (real CLI path,
   never a direct copy).
2. Write all 7 missing pages using the generated panel from day one (no
   hand-written wiring to later replace).
3. Wire the generated panel into the 23 currently-uncontrolled pages —
   pure addition, lowest regression risk.
4. Convert the 36 pages with hand-written controls to the generated
   panel, one at a time, diffing each page's live behavior before/after
   in the browser (`dx serve`) since this is the step most likely to
   silently change which props a page exposes.

### D5: `apps/docs` props table is a second consumer, not a copy

Emits the same introspected prop data (`FileIntrospection`/`PropField`,
already `Serialize`) as JSON consumed by `apps/docs` at build time,
proving the metadata is genuinely reusable rather than playground-coupled
— the same principle `catalog::schema`'s shared shape across four fetch
axes already established for upstream data.

## Risks / Trade-offs

- [Converting 36 already-working hand-written pages risks silently
  dropping a prop a page currently exposes but the generator's allowlist
  doesn't yet support] -> Mitigated by D4's ordering (uncontrolled pages
  first, established lowest-risk) and a per-page `dx serve` visual diff
  before considering that page converted, not a blanket batch conversion.
- [16-of-66 for generated `Preview` is a smaller fraction than the
  proposal's overall "66/66" framing might suggest] -> Accepted and
  stated plainly: 66/66 is `DemoState`/`Controls` coverage (real,
  generator-driven signal state for every item); `Preview` coverage is
  16/66, with the other 50 keeping a hand-written preview by design (D3),
  not a shortfall.
- [`SelectControl`'s signature change (D1) is a breaking change to every
  existing call site] -> Accepted, scoped entirely within
  `apps/playground` (not a registry item, not shipped to consumers) and
  fixed in the same change that makes the change, per Task 4.

## Open Questions

None — the control-primitive additions (D1), the generated-artifact shape
(D2), the `Preview` scoping rule (D3), the conversion order (D4), and the
docs-table mechanism (D5) are the load-bearing decisions and are resolved
above. The exact `Signal<T>`-sub-field-binding implementation detail noted
in D2 is left to Task 2's implementation, not blocking this proposal.
