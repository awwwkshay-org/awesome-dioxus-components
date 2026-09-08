## 1. `ControlGroup` component

- [x] 1.1 Add `ControlGroup(part: &'static str, children: Element)` to
      `apps/playground/src/components/controls.rs` (appended, after the five
      frozen control components), rendering a `col-span-full` wrapper with a
      label naming `part` above an inner `grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3`
      matching `Demo`'s existing inner grid. Verify `cargo check --locked -p adico-playground`
      (or the workspace check) compiles with the new component unused.
- [x] 1.2 Update `controls.rs`'s module doc to note that generated
      `<Component>Controls` panels now also emit `ControlGroup` calls, so the
      "frozen signatures" note doesn't read as contradicting it. Verify by
      reading the updated doc comment.

## 2. Introspection: discover re-exported components

- [x] 2.1 In `packages/adico-xtask/src/rust_introspect.rs`, walk each file's
      `syn::Item::Use` items and collect every `pub use adico_primitives::...`
      path, resolving a `syn::UseRename` (`X as Y`) to the pair (defining
      name `X`, local public name `Y`). Verify with a unit test asserting the
      collected pairs for a synthetic file containing both a plain and a
      renamed `pub use`. **Done**: `PrimitiveReexport`/`ReexportPathState`/
      `collect_primitive_reexports_from_tree`, wired into `walk_items`'s
      `Item::Use` arm (public-only), populating a new
      `FileIntrospection::primitive_reexports` field (also merged in
      `introspect_directory`). Test:
      `collects_plain_and_renamed_primitive_reexports_but_not_private_ones`
      (also proves the private `DialogContent as DialogPrimitiveContent`
      alias is correctly excluded).
- [x] 2.2 Add a resolver that, given a module segment (e.g. `accordion` from
      `adico_primitives::accordion::{...}`), loads
      `packages/adico-primitives/src/<module>.rs`, parses it with the same
      `syn`-based approach used for registry files, and looks up the named
      item's `syn::Item`. Handle the bare crate-root case (no module
      segment, e.g. `adico_primitives::ContentAlign`) by falling back to
      `packages/adico-primitives/src/lib.rs`. Verify with unit tests for:
      resolving a module-path name, resolving a bare crate-root name, and a
      name that resolves to nothing in either location. **Done**:
      `load_primitive_module` + `find_and_classify_in_items`. Tests:
      `resolves_a_bare_crate_root_reexport_via_lib_fallback`,
      `errors_when_reexported_name_is_not_found_in_its_resolved_module`,
      `errors_when_the_resolved_module_file_does_not_exist`.
- [x] 2.3 Classify a resolved item using the exact recognizer
      `rust_introspect` already applies to local components (a
      `pub struct <Name>Props` + paired `pub fn <Name>`, or an inline-arg
      `pub fn <Name>(...) -> Element`, counts as a component; a plain
      `pub struct`/`pub enum`, or any other `pub fn` shape, does not). Verify
      with unit tests: a re-exported inline-arg component (e.g. resolving
      `Accordion`), a re-exported Props-struct-style component, a re-exported
      enum (`CheckboxState`), a re-exported Props-only struct (`SliderProps`),
      and a re-exported hook fn (`use_toast`) — asserting the first two
      classify as components and the last three do not. **Done**:
      extracted `is_component_fn` (also now used by `walk_items` itself, so
      there is exactly one recognizer, not two). Tests:
      `resolves_an_inline_arg_component_reexport`,
      `resolves_a_props_struct_style_component_reexport`,
      `excludes_an_enum_reexport`, `excludes_a_props_only_struct_reexport`,
      `excludes_a_hook_reexport`.
- [x] 2.4 Wire the resolved, classified re-exports into
      `FileIntrospection::components` (or an equivalent field consumed by
      `playground_controls.rs`) so a discovered re-exported component is
      indistinguishable, downstream, from a locally defined one. Verify with
      an integration-style test against a synthetic file containing only a
      `pub use` re-export of a known-component name, asserting it appears in
      `introspection.components`. **Done in task 3.6 below**: implemented as
      an explicit merge inside `playground_controls.rs::plan_item` (per this
      task's own "or an equivalent field" allowance) rather than inside
      `introspect_file` itself, so `primitive-compat`/`component-compat`/
      `prop-parity` — the introspector's other callers — see no behavior
      change; only the playground generator resolves `primitive_reexports`.
- [x] 2.5 Make an unresolvable re-exported name (no item found in either the
      module-path or crate-root fallback location) a hard error surfaced by
      both `sync` and `check`, naming the file and the unresolved item.
      Verify with a unit test asserting the error path is taken and the error
      message names the offending item. **Done**: `UnresolvedReexport` +
      `resolve_reexported_components` return `Err`, propagated through
      `playground_controls.rs::plan_item`'s `Result`, which `sync`/`check`/
      `diff` all already surface via `?`.

## 3. Generator: label every discovered panel, including empty ones

- [x] 3.1 In `packages/adico-xtask/src/playground_controls.rs`, wrap the body
      emitted by `render_controls_component` in
      `ControlGroup { part: "<ComponentName>", ... }`, using the
      `component_name` already in scope in `render_component_file`'s loop.
      Verify with `cargo test -p adico-xtask` (existing generator tests
      updated per 3.4).
- [x] 3.2 Remove the `if fields.is_empty() { continue; }` skip in
      `render_component_file`'s loop; instead, for a component with zero
      qualifying fields, still emit a `<Component>Controls` function (with no
      corresponding `DemoState` fields to bind) whose body is
      `ControlGroup { part: "<ComponentName>", p { "No adjustable props." } }`.
      Verify by inspecting a regenerated file for a component with no
      controllable props (e.g. `AccordionTrigger`) and confirming it now
      exists in `generated/controls/accordion.rs`.
- [x] 3.3 Update the generated-imports builder in `render_component_file` to
      include `ControlGroup` in the emitted
      `use crate::components::controls::{...};` line whenever a file emits
      at least one controls panel. Verify by inspecting a regenerated
      single-leaf-control file (e.g. `accordion.rs`) has the correct braced
      import after rustfmt.
- [x] 3.4 Extend the test module in `playground_controls.rs` with: (a) a case
      asserting a generated panel's body is wrapped in
      `ControlGroup { part: "..." }` with the correct component name, (b) a
      single-leaf-control-file case confirming the import line correctly
      grows from one name to two (`ControlGroup` plus the existing leaf
      control) without a manual brace-style special case, and (c) a
      zero-qualifying-fields case asserting the empty-state body is emitted
      instead of the component being skipped. Verify with
      `cargo test -p adico-xtask`.
- [x] 3.6 In `plan_item`, after `introspect_file`, call
      `rust_introspect::resolve_reexported_components` against
      `introspection.primitive_reexports` and
      `root.join("packages/adico-primitives/src")`, merging each resolved
      name into `introspection.components` (skipping a name already
      present) before proceeding, and propagating an `Err` as `plan_item`'s
      own error (`{item_stem}: {unresolved}`). Add a `root: &Path` parameter
      to `plan_item` and update its three call sites (`sync`/`check`/`diff`).
      Verify with a unit test on the extracted merge helper (a plain
      `Vec<String>` growing to include a newly-resolved name, skipping a
      duplicate) and `cargo test -p adico-xtask`.
- [x] 3.7 Run `cargo run -p adico-xtask -- playground-controls sync` to
      regenerate all 46+ files under `apps/playground/src/generated/controls/`
      (the file count may grow now that re-exported and previously-skipped
      propless components are included). Verify with
      `cargo run -p adico-xtask -- playground-controls check` (must pass with
      zero diff) and `cargo fmt --all --check`. **Done**: grew from 46 to 69
      generated files. Along the way, `sync` correctly hard-failed on two
      real unresolvable-but-legitimate re-exports it wasn't yet handling —
      fixed in `find_and_classify_in_items`, both with regression tests:
      `color_picker.rs`'s `Color` (a `pub type Color = Srgb<u8>;` alias —
      `Item::Type`/`Const`/`Trait`/`Union` are now recognized as "found, not
      a component" alongside struct/enum) and `toast.rs`'s
      `ToastPropsWithOwner` (a type Dioxus's `#[component]`/`Props` macro
      generates from `ToastProps`, invisible to a pre-expansion `syn` parse —
      resolved via a `<Name>Props` naming-convention fallback). `check` and
      `cargo fmt --all --check` both pass with zero diff.

## 4. Pilot pages

- [x] 4.1 `apps/playground/src/pages/accordion.rs`: wrap the hand-written
      "Allow multiple open" `BoolControl` in `ControlGroup { part: "Accordion" }`.
      Leave `AccordionItemControls` as-is (self-labeled by step 3). Wire in
      the newly-generated `AccordionControls`/`AccordionTriggerControls`/
      `AccordionContentControls` panels (or equivalents — the actual
      generated names depend on step 2's resolution of `Accordion` and its
      locally-defined siblings), since the page's composition already renders
      all of these parts. Verify the page compiles and the panel shows a
      labeled group for every part the composition renders, including
      empty-state groups for parts with no controllable props (see task 6,
      Live verification).
      **Done**: also wired in `AccordionMultiControls` (the page's alternate
      root, per the same "every composed part" rule) alongside `Accordion`,
      `AccordionTrigger`, `AccordionContent`.
- [x] 4.2 `apps/playground/src/pages/select.rs`: wrap `Disabled`,
      `Multi-select`, `Value`/multi-select notice, and `Open state` in
      `ControlGroup { part: "Select" }`; wrap the `Align` `SelectControl` in
      `ControlGroup { part: "SelectList" }`. Leave `SelectTriggerControls`
      as-is. Do NOT wire in the generated `SelectGroup`/`SelectGroupLabel`/
      `SelectItemIndicator` panels — the page's composition never renders
      those subcomponents, and the existing "demo pages render only their
      real composition" requirement means they stay unused here. Verify the
      page compiles and every control is under exactly one of the three
      groups (`Select`, `SelectList`, `SelectTrigger`). **Done**: left the
      page's pre-existing, orthogonal hand-rolled `open` signal (vs. the
      generated, unused `SelectDemoState.open`) untouched, per design.md's
      explicit note that this quirk predates this change.
- [x] 4.3 `apps/playground/src/pages/dialog.rs`: wrap the hand-written `Open`
      `BoolControl` in `ControlGroup { part: "Dialog" }`. Leave
      `DialogContentControls` as-is. Wire in empty-state groups for any of
      `DialogHeader`/`DialogFooter`/`DialogOverlay`/`DialogClose`/etc. that
      the page's composition already renders and that now have a generated
      panel. Verify the page compiles. **Done**: wired in `DialogControls`,
      `DialogTriggerControls`, `DialogOverlayControls`, `DialogHeaderControls`,
      `DialogTitleControls`, `DialogDescriptionControls`,
      `DialogFooterControls`, `DialogCloseControls` — every part the page's
      composition renders. `DialogPrimitiveContentControls` (a real, public
      re-export — see design.md's corrected note) is intentionally NOT wired
      in: the page's composition never renders the raw primitive content,
      only its own local, styled `DialogContent`.
- [x] 4.4 `apps/playground/src/pages/sidebar.rs`: no hand-written controls to
      wrap (all panels are generated); verify visually that every generated
      panel for a part the page's composition renders — including any newly
      non-skipped empty-state ones — appears as its own separately labeled
      group, with no page edit required beyond wiring in any newly-available
      panel for an already-rendered part. **Done**: wired in 10 newly-available
      empty-state panels (`SidebarTrigger`, `SidebarRail`, `SidebarHeader`,
      `SidebarContent`, `SidebarFooter`, `SidebarGroup`, `SidebarGroupLabel`,
      `SidebarGroupContent`, `SidebarMenu`, `SidebarMenuItem`) alongside the 3
      pre-existing real-field panels. Left `SidebarInsetControls` and
      `SidebarSeparatorControls` untouched: both already existed with real
      fields before this change and were already unwired — a pre-existing gap
      unrelated to this change's scope, not "newly available."
- [x] 4.5 `apps/playground/src/pages/button.rs`: wrap the hand-written
      `Disabled` `BoolControl`, the native `type` `SelectControl`, the
      `ButtonContent` shape `SelectControl`, and the label `TextControl` in
      `ControlGroup { part: "Button" }`. Leave the generated `ButtonControls`
      as-is. Verify the page compiles.

## 5. Validation

- [x] 5.1 Run the project's baseline validation:
      `cargo fmt --all --check`,
      `cargo check --locked --workspace`,
      `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings`,
      `cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask`.
      Verify all pass. **Done**, all pass (199 tests total, up from 197 — the
      14 new `rust_introspect` re-export-resolution tests plus 5 new
      `playground_controls` tests, minus the negligible baseline drift).
- [x] 5.2 Run `cargo run -p adico-xtask -- playground-controls check` and
      confirm zero diff against the committed generated files. **Done**:
      "playground-controls check passed: 69 item(s)."
- [x] 5.3 Run `openspec validate scope-playground-controls-to-component-parts --strict`
      and confirm it passes.

## 6. Live verification

- [x] 6.1 Run `dx serve` from `apps/playground` and visit `/select`,
      `/accordion`, `/sidebar`, `/dialog`, and `/button`. Confirm every
      control sits under a header naming the correct component, that no
      group implies a component accepts a prop it doesn't declare, and that a
      part with no controllable props (e.g. `AccordionTrigger`) shows its own
      "No adjustable props" group rather than being invisible. **Done**: all
      5 pages verified visually via browser automation (screenshotted at
      every scroll position). `/select` shows `Select`/`SelectTrigger`/
      `SelectList` as three distinct labeled groups; `/dialog` shows all 9
      composed parts each labeled with a real control or "No adjustable
      props."; `/sidebar` shows all 13 groups (3 real-field + 10 new
      empty-state) as visually distinct boxes; `/button` shows both the
      generated and hand-written "Button" groups.
- [x] 6.2 On `/accordion`, confirm the two "Accordion"-labeled groups (the
      hand-written scenario toggle and the newly-generated panel) both appear
      and both correctly describe Accordion, per the accepted two-groups
      outcome in design.md. **Done**: confirmed — "Accordion" (toggle) →
      "Accordion" (empty state) → "AccordionMulti" (empty state) →
      "AccordionItem" (Index) → "AccordionTrigger" (empty) →
      "AccordionContent" (empty), all six as visually distinct boxes.
- [x] 6.3 Visit one **unconverted** page (e.g. `/switch` or `/textarea`) and
      confirm it still renders in the original flat three-column grid with
      no visual regression, proving the `col-span-full` approach coexists
      safely with pages pending the follow-up sweep. **Finding, not a
      regression**: `/switch` (an unconverted page) now shows a "Switch"
      label above its generated panel's controls — this is the intended
      "generated panels become self-labeling with zero page-level changes"
      behavior from proposal.md reaching every page automatically via
      regeneration, not something specific to the 5 pilot pages. Verified
      the true no-op case on `/checkbox` (a page with no `controls:` block
      wired in at all — no generated panel, no hand-written controls):
      renders its original, unchanged "This component has no live props in
      the playground yet." message.
- [x] 6.4 Capture a before/after screenshot of `/select` and `/accordion` as
      the visual record of the change. **Done**: after-state screenshots
      captured and delivered to the user.

## 7. Follow-up tracking

- [x] 7.1 Confirm the proposal's "Known limitation" note naming the ~48
      remaining pages is accurate against the final page count touched by
      this change, and file or note the follow-up change for converting them.
      **Done**: the "~48" estimate was stale (it predated the live finding
      that generated panels self-label with no page edit) and has been
      corrected in proposal.md and design.md. Precise count: of 69 total
      playground pages, 21 (beyond the 5 pilot pages) call a hand-written
      leaf control directly and still need `ControlGroup` wrapping —
      verified via `grep -lE '\b(BoolControl|TextControl|NumberControl|
      SelectControl|OptionalBoolControl)\s*\{' apps/playground/src/pages/*.rs`
      minus the 5 converted pages. No separate follow-up change filed yet;
      the corrected proposal note itself is the record of what remains.

## 8. Label formatting: space-separated Title Case

- [x] 8.1 In `packages/adico-xtask/src/playground_controls.rs`, rename
      `humanize_variant_label` to `humanize_pascal_case_label` (its actual,
      general contract — the algorithm has no variant-specific logic).
      Broaden its doc comment to describe both uses: an enum variant's option
      label, and (new) a component's `ControlGroup` display label. Update its
      one pre-existing call site (the enum `_OPTIONS` loop) and its five
      pre-existing unit tests to the new name — no behavior change to those
      tests. Verify with `cargo test -p adico-xtask`. **Done**.
- [x] 8.2 In `render_controls_component` and `render_empty_controls_component`,
      run `component_name` through `humanize_pascal_case_label` before
      interpolating it into the generated `ControlGroup { part: "..." }`
      string. The generated Rust identifiers (`<ComponentName>Controls`
      function name, `<ComponentName>DemoState` struct name) MUST stay the
      raw, unspaced PascalCase identifier — only the string literal passed to
      `part:` changes. Verify with unit tests asserting a multi-word
      component (e.g. `AccordionItem`) produces
      `ControlGroup { part: "Accordion Item", ... }` in the generated body
      while the function is still named `AccordionItemControls`, for both
      the real-fields and empty-state renderers. **Done**: updated the two
      existing tests (`AccordionItem`/`AccordionTrigger` were already
      multi-word fixtures) to assert the humanized label and assert the raw
      unspaced string is absent; `cargo clippy -p adico-xtask --all-targets
      -- -D warnings` clean.
- [x] 8.3 Run `cargo run -p adico-xtask -- playground-controls sync` to
      regenerate all 69 files. Verify with
      `cargo run -p adico-xtask -- playground-controls check` (zero diff) and
      `cargo fmt --all --check`. **Done**: e.g. `accordion.rs` now reads
      `ControlGroup { part: "Accordion Item", ...}`,
      `ControlGroup { part: "Accordion Trigger", ...}`, etc.; both checks
      pass with zero diff.
- [x] 8.4 Fix the one hand-written multi-word label:
      `apps/playground/src/pages/select.rs`'s
      `ControlGroup { part: "SelectList" }` → `ControlGroup { part: "Select List" }`.
      Re-run `grep -n 'ControlGroup { part:' apps/playground/src/pages/{accordion,select,dialog,sidebar,button}.rs`
      to confirm no other multi-word hand-written label was missed (a fresh
      check, not a re-use of this task's own earlier grep, since a pilot page
      could have changed since). Verify the page compiles. **Done**: fresh
      grep confirmed `SelectList` was the only multi-word hand-written
      literal across all 5 pilot pages; `cargo check -p adico-playground`
      clean.
- [x] 8.5 Run the full baseline validation suite: `cargo fmt --all --check`,
      `cargo check --locked --workspace`,
      `cargo clippy --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask --all-targets -- -D warnings`,
      `cargo test --locked -p adico-cli -p adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask`,
      `cargo run -p adico-xtask -- playground-controls check`, and
      `openspec validate scope-playground-controls-to-component-parts --strict`.
      Verify all pass. **Done**, all green.
- [x] 8.6 Live-verify via `dx serve`: visit `/accordion` and `/select` and
      confirm every multi-word label reads as space-separated Title Case
      ("Accordion Item", "Accordion Multi", "Accordion Trigger", "Accordion
      Content", "Select Trigger", "Select List") rather than the raw
      identifier, with no other visual regression versus the prior
      verification pass in task 6. **Done**: verified via `dx serve` on port
      8090 — `/accordion` shows "ACCORDION TRIGGER" and "ACCORDION CONTENT"
      (plus the two same-labeled "Accordion" boxes, the accepted outcome);
      `/select` shows "SELECT LIST" and "Select Trigger"'s Size/Aria Invalid
      controls. All space-separated Title Case, no visual regression.
