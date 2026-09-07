## Why

On a playground demo page for a multi-part component, the control panel renders
every prop control as one flat, unlabeled list. A viewer cannot tell which
component actually declares a given prop. On `/select`, "Disabled",
"Multi-select", "Size", "Aria Invalid", "Align", "Value", and "Open state" all
sit side by side with nothing indicating that `Size`/`Aria Invalid` belong to
`SelectTrigger`, `Align` belongs to `SelectList`, and `Multi-select` isn't a
real prop at all — it's a demo-scenario switch between `Select` and
`SelectMulti`. This makes it look like the root component accepts subcomponent
props it doesn't actually have, which is actively misleading about the real
API surface the playground exists to demonstrate.

The underlying prop data is already correctly separated per component —
`packages/adico-xtask/src/playground_controls.rs` already generates an
independent `<Part>DemoState` + `<Part>Controls` panel per component in a
registry file (e.g. `generated/controls/select.rs` already has separate
`SelectDemoState`, `SelectMultiDemoState`, `SelectTriggerDemoState`, with no
merging step anywhere in the generator). The flattening happens only at
render time: `apps/playground/src/components/demo.rs` renders the whole
`controls` element into one anonymous grid with no grouping concept, and pages
hand-assemble root props, generated subcomponent panels, and non-prop demo
toggles into that single flat list. This is a presentation fix, not a data
model change.

## What Changes

- Add a new `ControlGroup(part, children)` component to
  `apps/playground/src/components/controls.rs` that wraps a set of controls
  under a header naming the exact component that declares them, rendered as
  space-separated Title Case (e.g. `AccordionItem` → "Accordion Item",
  `SelectItemIndicator` → "Select Item Indicator") rather than the raw
  PascalCase Rust identifier. The five existing control components
  (`BoolControl`, `TextControl`, `SelectControl`, `NumberControl`,
  `OptionalBoolControl`) are unchanged — their signatures are declared frozen
  because generated code emits calls against them exactly.
- `packages/adico-xtask/src/playground_controls.rs`'s generator wraps every
  generated `<Part>Controls` panel's body in
  `ControlGroup { part: "<Humanized Component Name>" }` (humanized the same
  way an enum variant's option label already is) and includes `ControlGroup`
  in that file's generated imports. The generated Rust identifiers themselves
  (`<ComponentName>Controls`, `<ComponentName>DemoState`) are untouched —
  only the display string changes. Regenerating (`cargo xtask
  playground-controls sync`) makes every one of the 46 generated control
  files self-labeling with no page-level changes required.
- **The generator now also discovers components a registry file exposes only
  through a `pub use adico_primitives::...` re-export**, not just ones it
  defines locally. This closes a real, documented gap: today `Accordion` and
  `AccordionMulti` (bare re-exports in `registry/ui/accordion.rs`) are
  invisible to introspection and never get a panel of their own at all. A
  re-exported name is resolved by parsing its actual definition in
  `packages/adico-primitives/src/<module>.rs` and is included only if it
  resolves to a real renderable component — a non-component re-export (a
  type, enum, or hook, e.g. `checkbox.rs`'s `CheckboxState`, `slider.rs`'s
  `SliderProps`/`RangeSliderProps`, `toast.rs`'s `use_toast`/`consume_toast`/
  `ToastOptions`/`ToastType`) is excluded, not shown as a "component." A
  re-exported name that cannot be resolved at all (renamed, moved, deleted
  upstream) is a hard `sync`/`check` failure, never a silent skip.
- A part with zero controllable props (e.g. `AccordionTrigger`,
  `AccordionContent`, or a resolved-but-propless re-export like `Accordion`
  itself) now **still gets its own group**, rendering a plain "No adjustable
  props" line instead of being omitted — so a page's control panel always
  shows every real part of the composition it renders, not just the ones
  with something to adjust.
- Five pilot pages (`accordion`, `select`, `dialog`, `sidebar`, `button`) are
  hand-converted to wrap their non-generated, hand-written controls in
  `ControlGroup` too, labeled by the component that actually owns each
  control. A demo-scenario toggle that isn't a real prop (e.g. Accordion's
  "Allow multiple open", Select's "Multi-select") is grouped under the root
  component's own group rather than left unlabeled or given a group of its
  own.
- **Known limitation, explicit follow-up**: only 5 pages have their
  hand-written controls explicitly grouped in this change. Of the 69 total
  playground pages, 21 others (beyond the 5 pilot pages) call a hand-written
  leaf control — `BoolControl`/`TextControl`/`NumberControl`/`SelectControl`/
  `OptionalBoolControl` — directly, and those calls stay unlabeled until a
  follow-up change wraps them in `ControlGroup` by hand, the same way this
  change did for the 5 pilot pages. This is a materially smaller remaining
  gap than it might first appear: the labeling itself lives in the
  *generated* `<Component>Controls` panels, so every page that only wires in
  generated panels (with no hand-written controls of its own) already gets
  its group label for free the moment `playground-controls sync` regenerates
  — confirmed live: `/switch`, never touched by this change, now shows a
  labeled "Switch" group with no page edit. Every currently-unconverted page
  keeps rendering with no visual regression in the meantime (see design.md's
  `col-span-full` layout note).
- Out of scope: widening `classify_prop_type` to recognize prop types declared
  outside the same file (e.g. `adico_primitives::ContentAlign`) is a separate
  prop-coverage gap; grouping works the same around existing hand-rolled
  controls for such props.
- No breaking changes: nothing in `registry/ui/` changes, no registry item's
  public prop surface changes, and no installed-consumer file is touched. This
  is playground-app-only presentation and dev-tooling work.

## Capabilities

### New Capabilities
(none — this change adds requirements to existing capabilities rather than
introducing a new one)

### Modified Capabilities
- `adico-playground-demo-controls`: a generated `<Part>Controls` panel must
  render its controls under a header naming the component that declares
  them; every component a registry file exposes — defined locally or
  resolved from a `pub use adico_primitives::...` re-export — is represented,
  including with an empty-state message when it has no controllable props; a
  re-export that does not resolve to a real component is excluded, and one
  that cannot be resolved at all is a hard build/check failure.
- `adico-playground-structure`: a page's hand-written controls must be
  grouped by the component that declares them, and a control that is a demo
  scenario switch rather than a real prop must be grouped under the root
  component rather than left ungrouped.

## Impact

- `apps/playground/src/components/controls.rs` (new `ControlGroup` component,
  appended)
- `packages/adico-xtask/src/rust_introspect.rs` (new capability: enumerate
  `pub use` re-exports, including renamed ones, and resolve each into its
  defining module under `packages/adico-primitives/src/` to classify it as a
  component or not)
- `packages/adico-xtask/src/playground_controls.rs` (generator: wraps emitted
  panels, extends generated imports, emits an empty-state panel instead of
  skipping, extended test coverage)
- `apps/playground/src/generated/controls/*.rs` (all 46 files, regenerated via
  `cargo xtask playground-controls sync`)
- `apps/playground/src/pages/{accordion,select,dialog,sidebar,button}.rs`
  (pilot conversion)
- `packages/adico-primitives/src/*.rs` is **read** by the generator (to
  resolve re-exports) but never written — no primitive source changes.
- No change to `apps/playground/src/components/demo.rs`, `registry/ui/**`,
  installed consumer fixtures, the CLI, or any database/schema/config surface.
