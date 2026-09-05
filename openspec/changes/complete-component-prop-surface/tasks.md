## 1. Shared prop vocabulary

- [x] 1.1 Add `registry/lib/variants.rs` (new `registry:lib` item,
      alongside `cn`): `pub enum Radius { None, Sm, Default, Lg, Xl, Full }`
      with `#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]`,
      `#[default]` on `Default`, and `impl Radius { pub fn class(self) ->
      &'static str }` mapping to the corresponding `rounded-*` Tailwind
      utility (`Default` maps to the bare `rounded-lg` class tracking the
      `--radius` token, matching `theme_builder.rs`'s existing default).
      Register it in `registry/registry.json`, with its `files[0].checksum`
      set to the file's real `sha256` hex digest (e.g. `shasum -a 256
      registry/lib/variants.rs`) — checksums in `registry.json` are
      hand-maintained, not computed by `registry build`, and a stale one
      fails install/`adico add` even though `registry validate` stays
      silent about it (no `registryDependencies` of its own, matching
      `cn`'s entry shape). Add a matching `("lib/variants.rs",
      RegistryLocation::Embedded)` arm to `ConfiguredRegistryReader::read`
      in `packages/adico-cli/src/main.rs` (the embedded-registry file
      reader is an explicit per-file `include_bytes!` allowlist, not a
      directory scan — every other `registry:lib`/`registry:ui` item
      already has its own arm there) so `adico add`/`--replace` can
      actually fetch it. Verify with a unit test asserting each variant's
      `class()` output and that `Default::default()` is `Radius::Default`,
      and by installing `variants` into a scratch consumer via the real
      `adico add` CLI path (not a direct file copy) to confirm the new
      reader arm and checksum are both correct.
- [x] 1.2 Add `variants` to the `registryDependencies` of every item
      identified in task 3.1's target list (do not add it speculatively to
      items that don't get a `radius` prop). Verify
      `cargo run -p adico-xtask -- registry build` and `registry validate`
      pass.

**Checksum discipline for every remaining task in this change:** any task
below that edits a `registry/ui/*.rs` or `registry/lib/*.rs` file's
content must also recompute and update that file's `checksum` in
`registry/registry.json` (`shasum -a 256 <path>`) in the same task —
`registry build`/`registry validate` do not do this for you, and a stale
checksum only surfaces later, at install time.

## 2. Convention normalization

- [x] 2.1 **`disabled` representation.** Categorize every styled
      component as native-leaf (`Option<bool>`) or primitive-backed
      (`ReadSignal<bool>`); fix every component whose current
      representation doesn't match its category, documenting `Button`'s
      native-only routing as the one intentional exception in its own doc
      comment. Verify by grepping every `disabled` field's type across
      `registry/ui/*.rs` and confirming each matches its category or is
      `Button` with its exception comment present.
- [x] 2.2 **`class: Option<String>` everywhere.** Add to `Slider`,
      `RangeSlider`, `Toast` (fix in
      `packages/adico-primitives/src/{slider,toast}.rs`, since their
      registry facades re-export the primitive's props type verbatim —
      keep the facade a bare re-export, do not introduce a
      registry-owned wrapper struct), and `AreaThumb`/`HueSlider` (fix in
      `registry/ui/color_picker.rs`, since these two take no props struct
      at all today). **Correction found during implementation:**
      `styling-usage check` does not actually assert a `class` field
      exists on every styled item (verified by reading
      `packages/adico-xtask/src/styling_usage.rs` — its conditions (a)–(f)
      cover Tailwind-only/token-compliance, not prop-surface shape); verify
      instead by grepping each of the four files for a real `class:
      Option<String>` field/parameter reaching the rendered root element,
      plus a compile check that
      existing `with_class()`-style workarounds in `slider.rs` can be
      simplified now that a real `class` field exists (simplify them in
      this task, don't leave the workaround dangling unused).
- [x] 2.3 **`attributes: Vec<Attribute>` extends coverage.** Using
      `statics/prop_parity/*.json`'s `dioxus-components`/
      `dioxus-primitives` axes (the ones that enumerate native events
      individually) as the evidence source, identify every styled
      component that renders a real element but has no `#[props(extends =
      GlobalAttributes)]` `attributes` field, and add one.
      **Correction found during implementation:** Change A's earlier
      `looks_like_native_event_name`/`has_attributes_extend` fix already
      closed nearly all of this gap — the actual evidence-driven residue
      was exactly 3 items, not the large remainder design.md's Context
      implied: `toggle` (missing `onmounted`/`onfocus`/`onkeydown` — the
      registry facade wasn't forwarding these three individually-named
      callback fields the primitive already exposes, a documented
      Dioxus-issue workaround distinct from the generic `attributes`
      mechanism — plus a plain missing `attributes` field), `sidebar`'s
      `SidebarTrigger` (missing `attributes`; added after the trigger's
      own fixed `onclick` since Dioxus requires an attribute spread to be
      an element's last attribute, so a caller's own `onclick` replaces
      rather than composes with the toggle, matching every other
      `attributes`-accepting component's own precedent), and
      `pagination`'s `PaginationPrevious`/`PaginationNext` (missing
      `attributes`, added and forwarded to the underlying
      `PaginationLink`, which already had it). Verified
      `cargo run -p adico-xtask -- prop-parity sync` then reports zero
      native-event-shaped `missing` entries for the fixed items (per
      `prop_parity.rs`'s `looks_like_native_event_name`/
      `has_attributes_extend` coverage rule).
- [x] 2.4 **Restore dropped props.** `Checkbox`: restore the primitive's
      form `value` and `attributes` fields the registry facade currently
      narrows away. `ToastProvider`: un-narrow `default_duration`/
      `max_toasts` back to `ReadSignal<...>`, matching the primitive's own
      type. Verify with a unit test per component asserting the facade's
      prop type matches the primitive's, and update
      `docs/adico/component-hardening-audit.md` to record both as closed
      (matching its existing `Menubar`-dropped-`disabled` entry's format).
- [x] 2.5 **Handler naming.** Rename `AlertDialogAction`/
      `AlertDialogCancel`'s and `ToolbarButton`'s `on_click` to a
      component-appropriate semantic name (e.g. `on_select`), matching the
      primitive-facing convention every other primitive-backed component
      in the registry already uses. Verify by grepping for `on_click` in
      `registry/ui/*.rs` and confirming no remaining primitive-backed
      component uses it (a native-leaf component's plain `onclick`
      passthrough via `attributes` is unaffected — this only renames
      hand-declared `on_click` fields).
- [x] 2.6 **`**BREAKING**` controlled-trio.** Migrate `Select`'s,
      `Combobox`'s, and `TagGroup`'s singular-select function `value`
      parameter from `Option<ReadSignal<Option<T>>>` to
      `ReadSignal<Option<T>>`, matching their own `SelectMulti`/
      `ComboboxMulti`/`TagGroupMulti` functions' already-compliant
      `values` shape. The outer `Option` served no behavioral purpose —
      `use_controlled` already treats "no signal" and "a signal
      permanently holding `None`" identically — so no case needed the
      double-`Option` kept; the registry facade now defaults to
      `ReadSignal::new(Signal::new(None))` and wraps in `Some(..)` only
      when forwarding to the primitive, which keeps its own
      double-`Option` field unchanged (out of this task's file scope).
      Verified `cargo check --locked --workspace` and, standalone, real
      `adico add --replace` refreshes + `cargo check` in
      `tests/installation/{select-consumer,wave4-consumer,
      wave5-tag-group-consumer}` (the three real fixtures using these
      components); updated the one call site each that used them
      (`apps/playground/src/pages/{select,combobox,tag_group}.rs`) — no
      `examples/*`/`tests/installation/*` call site passes this `value`
      prop today, so none needed updating. Playwright re-run deferred to
      task 6.3 rather than duplicated here: `select`/`combobox` are both
      touched again in Wave 1 (task 5.1) and `tag-group` in Wave 5 (task
      5.5), so a live browser pass now would be re-done anyway once those
      waves land.

## 3. `radius` rollout

- [x] 3.1 Using `grep -lE "rounded-(none|sm|md|lg|xl|2xl|3xl|full)\b"
      registry/ui/*.rs` (49 files as measured) as the starting candidate
      list, hand-surveyed every file to produce the final target list
      below: one `radius: Radius` per registry item (occasionally on more
      than one exported component when they're genuinely one visual unit
      rendered together, e.g. a trigger + its own popup), each with the
      `#[props(default = Radius::<X>)]` matching that item's *current*
      literal class (no visual regression), and every excluded sub-part
      named with a reason.

      **Target list** (item — component(s) — default — excluded sub-parts):
      - accordion — `AccordionTrigger` — `Md`
      - alert — `Alert` — `Default` (already `rounded-lg`)
      - alert-dialog — `AlertDialogContent` — `Default`; excluded: `AlertDialogAction`/`AlertDialogCancel` (mirror `Button`'s own fixed corners, not the dialog panel's surface)
      - attachment — `Attachment` — `Default`; excluded: `AttachmentTrigger`, `AttachmentMedia` (internal sub-regions of the card)
      - avatar — `Avatar` + `AvatarFallback` (independent props, same default, since a caller who reshapes one should reshape both to keep the clip consistent) — `Full`
      - badge — `Badge` — `Md`
      - bubble — `BubbleContent` — `Xl` (`rounded-2xl`); excluded: `BubbleReactions` (separate decorative reaction pill)
      - button — `Button` — `Md`; strip base string **and** every non-`Default`/`Icon` `ButtonSize::class()` arm (`Xs`/`Sm`/`Lg`/`IconXs`/`IconSm`/`IconLg`)
      - button-group — `ButtonGroupText` — `Md`; excluded: `ButtonGroup`'s `has-[...]:[&>...]:rounded-r-md` (arbitrary-variant descendant selector, targets a nested select-trigger), `ButtonGroupOrientation::class()`'s `rounded-{l,r,t,b}-none` (side-specific joins)
      - calendar — `CalendarView` — `Md`; excluded: nav buttons, month/year select triggers (internal chrome), grid's `[&_button]:rounded-md` (arbitrary-variant descendant selector)
      - card — `Card` — `Xl`
      - carousel — **excluded entirely**: no item-level bounded surface: the only `rounded-full` is `CarouselPrevious`/`CarouselNext`'s shared nav-button class, not the carousel's own surface
      - color-picker — `ColorArea` — `Md`; excluded: `AreaThumb` (`rounded-full` drag handle); `AreaTrack`'s `rounded-[inherit]` needs no change (already inherits `ColorArea`'s corner automatically)
      - combobox — `ComboboxInput` + `ComboboxList` (shared value — visually one paired control) — `Md`; excluded: `ComboboxOption` (per-row highlight)
      - command — `Command` — `Md`; excluded: `CommandInput`, `CommandItem` (internal chrome/per-row highlight)
      - context-menu — `ContextMenuContent` — `Md`; excluded: `ContextMenuItem` (per-row highlight)
      - date-picker — `DatePickerInput` — `Md`; excluded: `DatePickerTrigger` (small disclosure icon), `[&_[role=spinbutton]]:rounded-sm` (arbitrary-variant descendant selector)
      - dialog — `DialogContent` — `Default`; excluded: `DialogClose` (`rounded-xs`, not in the `Radius` scale at all)
      - drag-and-drop-list — `DragAndDropListItem` — `Md`; excluded: `DragAndDropDropIndicator` (`rounded-full` drop-position line, not a content surface)
      - drawer — **excluded entirely**: every real corner is either `DrawerDirection::class()`'s side-specific, direction-dependent value (`rounded-{t,b,l,r}-[10px]`, none matching the bare-`rounded-*` regex) or decorative-internal (`DrawerClose`, the grab-handle bar)
      - dropdown-menu — `DropdownMenuContent` + `DropdownMenuTrigger` (shared value) — `Md`; excluded: `DropdownMenuItem` (per-row highlight)
      - empty — `Empty` — `Default`; excluded: `EmptyMediaVariant::Icon`'s own `rounded-lg` (variant-gated chrome, not the root's own class)
      - hover-card — `HoverCardContent` — `Md`
      - input — `Input` — `Md`
      - input-group — `InputGroup` — `Md`; excluded: `InputGroupInput`/`InputGroupTextarea`'s `rounded-none` (deliberately flush with the group border — changing it independently breaks the flush-field look, per the module's own doc comment)
      - item — `Item` — `Md`
      - kbd — `Kbd` — `Sm`
      - marker — `Marker` — `Full`
      - menubar — `Menubar` + `MenubarContent` (shared value) — `Md`; excluded: `MenubarTrigger`, `MenubarItem` (internal chrome/per-row highlight)
      - native-select — `NativeSelect` — `Md`
      - navigation-menu — `NavigationMenuTrigger` + `NavigationMenuContent` + `NavigationMenuLink` (shared value — one visual family) — `Md`
      - pagination — `PaginationLink` — `Md` (`PaginationPrevious`/`PaginationNext` compose `PaginationLink` and inherit automatically; no separate prop)
      - popover — `PopoverContent` — `Md`
      - progress — `Progress` — `Full`
      - radio-group — **excluded entirely**: a radio button's circular shape is its semantic identity (matching `skeleton`'s `Circle` variant reasoning), not an independent cosmetic dial; the outer ring and the `before:rounded-full` inner dot both stay hardcoded
      - select — `SelectTrigger` + `SelectList` (shared value) — `Md`; excluded: `SelectOption` (per-row highlight)
      - sidebar — `SidebarTrigger` and `SidebarMenuButton` (two **independent** props — not shared, since they're rendered in unrelated locations, not one visual unit) — `Md` each; excluded: `SidebarVariant::Floating`/`Inset`'s own conditional radius (switched by the variant itself, not a free-standing dial), `SidebarGroupLabel`-shaped internal chrome
      - skeleton — **excluded entirely**: `SkeletonVariant::class()`'s `Default`/`Circle` distinction *is* the variant's whole job (rectangle vs. circle placeholder); an independent `radius` would allow a nonsensical `Circle` + `radius: None` combination
      - slider — `SliderTrack`, `SliderRange`, `SliderThumb` (three **independent** props, matching their existing independent `class` params — no shared context exists to thread one value automatically) — `Full` each
      - switch — `Switch` only (one prop; `SwitchThumb` is composed inline inside `Switch`'s own body, not a separately-invoked component, so `Switch` threads the same value to both its own track class and the inline thumb's class) — `Full`
      - tabs — `TabList` — `Default` (already `rounded-lg`); excluded: `TabTrigger`'s `rounded-md`/`rounded-none` pair (encodes `TabsVariant`'s segmented-vs-underline visual identity — an independent radius would allow a broken hybrid)
      - tag-group — `TagOption` — `Md`; excluded: `TagRemoveButton` (`rounded-full` icon control inside the chip)
      - textarea — `Textarea` — `Md`
      - theme-builder — **excluded entirely**: every `rounded-md` is internal preview-swatch/mockup chrome inside the editor panel, not the panel's own surface (its pre-existing `radius: String` field is the unrelated `--radius` CSS-value editor)
      - theme-switcher — **excluded entirely**: the one `rounded-full` is a per-theme color-swatch dot rendered in a loop — circular is the established convention for a selectable color dot (same reasoning as `skeleton`'s `Circle` variant), not cosmetic
      - toast — **excluded entirely, architectural conflict found during
        implementation**: `Toast`'s facade takes `props: ToastProps` — the
        primitive's own struct, re-exported verbatim, not a registry-owned
        wrapper — because `ToastProvider`'s default `render_toast` callback
        spreads `Toast { ..props }` from a `ToastPropsWithOwner`, which only
        works if `Toast`'s param type structurally matches. Adding
        `radius: Radius` would require either the primitive depending on
        the registry-owned `Radius` type (violates the primitives → registry
        one-directional dependency rule) or replacing `Toast`'s props type
        with a registry-owned wrapper (breaks the `..props` spread). Neither
        is worth forcing for one component's corner radius; `ToastCloseButton`
        stays hardcoded too (it was only ever going to be excluded, per the
        original survey).
      - toggle — `Toggle` — `Md`
      - toggle-group — `ToggleItem` — `Md`
      - toolbar — `ToolbarButton` — `Md`
      - tooltip — `TooltipContent` — `Md`

      42 items get `radius`; 7 are excluded entirely (`carousel`, `drawer`,
      `radio-group`, `skeleton`, `theme-builder`, `theme-switcher`,
      `toast`), each with the reason recorded above (and to be repeated in
      that component's own doc comment during 3.2, matching this file's
      existing exception-documentation discipline).
- [x] 3.2 Strip every `rounded-*` literal from each target component's
      base class string(s) **and** every `impl FooVariant/FooSize { fn
      class(self) }` match arm that currently hard-codes one (confirmed
      present in at least `button.rs`'s `ButtonSize::class()` — check all
      14 files with this inherent-impl pattern, not just `Button`).
      Compose `radius.class()` into the final `cn()` call instead. Verify
      per component that its Tailwind classes still produce the same
      visual rounding at the `Default` variant (no visual regression at
      the default).
- [x] 3.2b Add one `(item, part, "radius")` entry per task 3.1 target item
      to `packages/adico-xtask/src/prop_parity.rs`'s
      `ADICO_EXTENSION_REASONS` table (design.md's D3 addendum), naming
      the part `radius` actually lands on for that item, with reason
      "adico extension: consistent corner-radius control not present
      upstream" (or the component-appropriate equivalent). 53 entries
      added (some items have 2-3 radius-bearing parts); each part id
      verified against the real id `statics/prop_parity/<item>.json`
      already records, not hand-derived from the naming rule alone —
      `TagOption`/`ToggleItem`/`TabList`/`ColorArea` don't share their
      item's own name as a literal prefix, so `part_id_for` falls through
      to kebab-casing the whole component name (`tag-option`/
      `toggle-item`/`tab-list`/`color-area`) instead of stripping one, and
      hand-guessing would have gotten these four wrong. Verified `cargo
      run -p adico-xtask -- prop-parity sync` classifies `radius` as
      `adico_extension` on 38 of the 42 target items (98 axis-level
      entries) and `prop-parity check` passes. **Correction found during
      implementation:** the remaining 4 items (`command`, `empty`, `kbd`,
      `input-group`) plus `color-picker`'s specific `color-area` part
      never fire — not a table defect, but a structural limit: their part
      is `unresolved` on every axis (`cmdk` is a third-party dependency
      with no catalog axis; "no prop data available"; no matching part id
      on the axis at all), and `AdicoExtension` classification only runs
      inside `resolve_full_part_props`'s `Resolved` branch. The table
      entries stay in place as inert, future-proof documentation — exactly
      the table's own stated purpose, if evidence ever becomes available
      for these five, per the module's own doc comment.
- [x] 3.3 Add condition (g) to `packages/adico-xtask/src/styling_usage.rs`:
      for every item in the task 3.1 target list, assert no `rounded-`
      literal appears in its `registry/ui/*.rs` file's radius-bearing
      component(s) outside `Radius::class()`'s own match arms. This is
      per-*component*, not whole-file — an item's explicitly excluded
      sub-parts (task 3.1's list) keep their own hard-coded `rounded-*`
      and are not asserted against; record each item's excluded sub-part
      names in a new hand-maintained `radiusException: Vec<{ part,
      reason }>`-shaped field on `StylingUsageRecord` (preserved across
      `sync`, matching `styleException`/`colorException`'s existing
      preserve-on-sync behavior — this is the one condition in this file
      that needs a new exception list, not a rule change to an existing
      one). Verify the new condition fails against a deliberately
      reintroduced `rounded-md` literal in a radius-bearing component with
      no matching exception, then passes against the real, fixed source
      with every excluded sub-part named in `radiusException`.
      **Corrections found during implementation:** the naive whole-source
      substring check produced two classes of false positive, both fixed
      rather than accepted — a registry file's own `#[cfg(test)]` module
      re-asserts a `class()` fn's literal output for testing (not live
      production code), so the detector now splits the source at
      `#[cfg(test)]` and only scans what precedes it; and two files
      (`slider.rs`, `toggle_group.rs`) mentioned a literal `rounded-full`/
      `rounded-md` in doc-comment *prose* explaining a design decision,
      which the substring check couldn't distinguish from real code —
      reworded both sentences to describe the shape without the literal
      class name rather than teaching the checker to parse doc comments.
      18 of the 42 target items needed a populated `radiusException`
      (`alert-dialog`, `attachment`, `bubble`, `calendar`, `color-picker`,
      `combobox`, `command`, `context-menu`, `date-picker`,
      `drag-and-drop-list`, `dropdown-menu`, `empty`, `input-group`,
      `menubar`, `select`, `sidebar`, `tabs`, `tag-group`); the other 24
      target items' `radius`-bearing part(s) leave no stray literal at
      all, so their exception list stays empty.

## 4. `loading` rollout

- [x] 4.1 Add `loading: bool` (`#[props(default)]`) and `loading_text:
      Option<String>` to `Button`, `InputGroupButton`, `PaginationLink`,
      `AlertDialogAction`, `ToolbarButton`, `SidebarMenuButton`, and
      `DataTable`. Each composes the existing `Spinner` registry item
      (declared in `registryDependencies`, not duplicated) when `loading`
      is true, sets `aria-busy="true"` and native `disabled`, and renders
      `loading_text` in place of/alongside the normal content per that
      component's own composition shape. `InputGroupButton` simply
      forwards to `Button` (composes it already). `PaginationLink` renders
      an `<a>`, which has no native `disabled` — uses `aria-disabled` plus
      an onclick guard instead. `AlertDialogActionPrimitive`'s `attributes`
      only extends `GlobalAttributes` (no `button`-specific extend), so
      `disabled` isn't available via the usual shorthand keyword there —
      built by hand via `Attribute::new(...)`, matching `slider.rs`'s own
      precedent for this exact limitation. `ToolbarButton`/
      `SidebarMenuButton` combine `loading` with their existing `disabled`
      field via `use_memo`/a plain `||`. `DataTable` (a whole-table
      pattern, not a single button) renders a full-width row with the
      spinner + text in place of the row data while loading; found
      `TableCell` has no `attributes` passthrough at all, so `aria-busy`
      moved to the outer wrapper `div` instead (a native element, no
      extends limitation there). **Also found and fixed a real
      methodological gotcha**, not a task defect: `adico`'s registry
      source is embedded via `include_bytes!` at *compile* time, so
      invoking the pre-built `target/debug/adico` binary directly (instead
      of through `cargo run -p adico-cli --`) silently serves stale
      registry content after a source edit — caught when
      `examples/{basic-spa,basic-ssr}` kept installing an already-fixed
      `data_table.rs`'s *previous* broken revision.
- [x] 4.2 Add each of the seven `(item, part, prop)` entries to
      `packages/adico-xtask/src/prop_parity.rs`'s `ADICO_EXTENSION_REASONS`
      table with the reason "adico extension: shadcn composes `<Button
      disabled><Spinner /></Button>` by hand; adico exposes it as a first-class
      prop instead" (or the component-appropriate equivalent). 14 entries
      added (`loading` + `loading_text` × 7 items). Verified `cargo run -p
      adico-xtask -- prop-parity sync` classifies both as `adico_extension`
      on 6 of the 7 items (`button`, `input-group`, `pagination`,
      `alert-dialog`, `toolbar`, `sidebar`); `data-table`'s entries stay
      inert (zero resolvable parts on any axis — a Dioxus-only pattern
      with no shadcn/dioxus-components catalog match at all), the same
      documented structural limit task 3.2b already recorded for
      `command`/`empty`/`kbd`/`input-group`'s `radius`. `prop-parity check`
      passes.

## 5. Parity-prop waves

Each wave: for every item listed, resolve every `missing` entry
`statics/prop_parity/<item>.json` currently records against its
best-matching axis — implement it, or record it `intentional_difference`
with a written reason (design.md's D6). Consult the generated JSON file
directly for the exact prop list; it is not duplicated here.

- [ ] 5.1 **Wave 1 — composite/collection controls** (96 missing entries):
      `combobox` (29), `slider` (26), `select` (16), `calendar` (14),
      `navigation-menu` (11). Verify `cargo run -p adico-xtask --
      prop-parity diff` reports no drift for these five items against
      their updated source, and that re-running `prop-parity sync`
      produces records with zero remaining `missing` status for them.

      **Progress: `combobox`, `slider`, `select` done** (3 of 5).
      `prop-parity diff` reports zero remaining `missing` status for all
      three. `select`'s own notes:
      - Real fixes: `attributes: Vec<Attribute>` forwarding on `Select`,
        `SelectMulti`, `SelectTrigger`, `SelectOption`, `SelectValue`,
        `SelectList` (all six facades; the primitives already supported
        it), and a new `SelectTriggerSize { Sm, Default }` enum matching
        shadcn's own real `"sm" | "default"` cva axis on `select.trigger`
        (verified against `statics/catalogs/shadcn.json`, not guessed) —
        the hardcoded `h-9` moved out of the base class string into
        `SelectTriggerSize::class()`, same discipline as the B3 radius
        rollout.
      - Reused reasons from `combobox`: `COLLECTION_MANAGEMENT_REASON`
        (`highlightItemOnHover`, `autoComplete`, `isItemEqualToValue`,
        `itemToStringLabel`/`Value`, `items`, `onOpenChangeComplete`),
        `FORM_PARTICIPATION_REASON` (`form`, `readOnly`, `required`),
        `ATTRIBUTES_COVERAGE_REASON` (`id`), `SEPARATE_COMPONENT_REASON`
        (`multiple` → `SelectMulti`), `CASCADING_DISABLED_REASON`
        (`trigger.disabled`).
      - Two new reasons, both reused going forward: `MODAL_POPUP_REASON`
        (`root.modal` — no modal-vs-non-modal focus-trap toggle) and
        `CUSTOM_VALUE_RENDER_REASON` (`value.children` — no
        caller-supplied value-renderer callback; needs the primitive to
        thread the typed selected value out to one, deferred). Also
        retroactively reclassified `combobox`'s own `root.modal` entry
        from `COLLECTION_MANAGEMENT_REASON` to the new, more accurate
        `MODAL_POPUP_REASON` — a one-line correction to the prior
        commit's table, not a re-litigation of the entry itself.
      - **Discovered defect, not fixed in this change:** `select`'s own
        `name: ReadSignal<String>` field (present on both `Select` and
        `SelectMulti`, both registry and primitive layers) is completely
        dead — grepped for every use in `packages/adico-primitives/src/
        select.rs` and found none; no hidden `<input name=... value=...>`
        or any other consumer of it exists. Setting it silently does
        nothing today. Invisible to `prop-parity` (the classifier only
        checks field *presence*, not whether a field is wired to real
        behavior), found only by reading source while checking whether
        `name` explained why `form`/`readOnly`/`required` weren't also
        already present. Left as `FORM_PARTICIPATION_REASON`'s deferred
        follow-up work would fix this as a side effect (a real hidden
        mirror `<input>` needs to consume `name` for the first time);
        not fixed standalone here, since it's a pre-existing correctness
        defect outside this wave's prop-surface-evidence scope, not a
        `prop-parity` gap.

      Corrections found during `combobox`, load-bearing for the rest of
      this wave:
      - `prop_parity.rs` had no per-`(item, part, prop)` mechanism to
        classify a genuine *upstream* gap `intentional_difference` — only
        `ADICO_EXTENSION_REASONS` existed, for the opposite direction (an
        adico field with no upstream counterpart). Added
        `INTENTIONAL_DIFFERENCE_REASONS`, keyed and looked up identically
        (reusing `adico_extension_reason`), consulted by
        `classify_upstream_prop` (now threaded `item`/`part`) after the
        presence check and before falling through to `missing`. This is
        what D6's "record it `intentional_difference` with a written
        reason" was always going to need; see design.md's D6 addendum for
        the four reason categories this introduces
        (`COLLECTION_MANAGEMENT_REASON`, `SEPARATE_COMPONENT_REASON`,
        `PART_DECOMPOSITION_REASON`, `ATTRIBUTES_COVERAGE_REASON`,
        `CASCADING_DISABLED_REASON`, `FORM_PARTICIPATION_REASON`).
      - `combobox`'s `name`/`form`/`required`/`readOnly` (base-ui `root`)
        need native browser form participation (`FormData` inclusion,
        constraint validation) via a hidden mirror `<input>` synced to the
        controlled value — a new `adico-primitives` mechanism, not a prop
        rename. Per D6's third terminal state ("named as an explicit
        follow-up ... a genuine primitive-behavior gap requiring new
        `adico-primitives` work"), recorded `intentional_difference` with
        `FORM_PARTICIPATION_REASON` rather than built here — Change A's
        already-approved 4-status schema (`present | missing |
        intentional_difference | adico_extension`) has no fifth status,
        so the reason text itself names it a deferred follow-up. **This
        pattern will very likely recur** for `select`/`calendar` later in
        this wave and for `switch`/`checkbox`/`progress`/`radio-group` in
        Wave 3 — resolve it the same way there rather than re-litigating,
        and track the accumulated list of components needing this as a
        candidate for a future change.
      - Two genuine gaps got real fixes instead of a reason: `aria_label`
        on `ComboboxInput` and `ComboboxList` (adico had no accessible-name
        prop at either level) and `attributes: Vec<Attribute>` forwarding
        on all five combobox registry facades (`Combobox`, `ComboboxMulti`,
        `ComboboxOption`, `ComboboxEmpty`, `ComboboxInput`, `ComboboxList`)
        — the primitives already had `attributes` fields; only the
        registry layer wasn't forwarding them (same defect class as task
        2.3, but for a literal upstream prop named `attributes`, which
        2.3's native-event-name heuristic doesn't cover).
      - Also fixed in this pass, discovered starting `slider` (the wave's
        next item): `rust_introspect.rs`'s `inline_component_props`
        deliberately returns `None` for the `props: FooProps` shape,
        deferring to an `Item::Struct` match — but that only fires for a
        `FooProps` struct *defined in the same file*. `Slider`/
        `RangeSlider`/`Toast` wholesale-reuse the primitive's own Props
        struct (`pub fn Slider(props: SliderProps)` + `pub use
        adico_primitives::slider::SliderProps` — no local definition), so
        `prop_parity.rs` saw zero adico fields for them and reported every
        single upstream prop `missing` (26 for `slider`, matching
        design.md's count, but for the wrong reason — a tooling gap, not a
        real 26-prop surface gap). `introspect_item` now falls back to the
        primitive module's own struct definitions (via the existing
        `find_primitive_modules`) for a name this item's own registry
        source doesn't already provide, never overriding a registry-owned
        definition that legitimately narrows/widens the primitive's own
        surface (two new tests cover both directions). Before the fix
        every upstream prop on `slider`'s `root`/`range-slider` parts
        showed `missing` (the false-positive inflation design.md's 26
        count was measured against); after it, `prop-parity sync` reports
        31 genuine `missing` entries (`root`'s real native form-integration
        gaps, `thumb`'s per-thumb accessibility/behavior props, and three
        `attributes`-forwarding gaps on `track`/`range`/`thumb`) — a higher
        number than 26, since the count is now measuring the real surface
        instead of an undercounted false positive; the same fallback
        also corrected two *other* shapes of the identical underlying
        problem ("no entry in `introspection.props` under the name
        `adico_field_names` looks up"), each in a wave-relevant item:
        `toast`'s facade reuses `ToastProps` wholesale, exactly like
        `Slider`; `calendar`'s registry facade deliberately *consolidates*
        several primitive components onto one shared, registry-owned
        struct (`CalendarNavigationButtonProps` for both
        `CalendarPreviousMonthButton`/`CalendarNextMonthButton`,
        `CalendarSelectFieldProps` for four `CalendarSelect*` components),
        so the naive `{component}Props` lookup misses even though a
        same-shaped struct exists on the *primitive* side per component
        (`CalendarNextMonthButtonProps`, etc.) — the fallback finds that
        one instead, correctly, since the registry's own map has neither
        key for these.
      - Also fixed in this pass (found via the full baseline run, unrelated
        to combobox itself but blocking a green baseline): `adico-cli`'s
        `plan_cargo_dependency_edits` rejected a *second*, separate
        `adico add` that needed an additional Cargo feature on a
        dependency an *earlier* `adico add` had already written without
        it (e.g. `spinner` alone writes a plain `adico-primitives =
        "=0.1.0"`; a later `adico add dialog` needs `features = ["web"]`
        on the same crate) — surfaced by Section 4's `loading` rollout
        giving `button` a new `spinner` registry dependency, which put
        `dialog` (`registryDependencies: [button, ...]`) and `spinner` in
        the same merged Cargo dependency for the first time.
        `packages/adico-cli/src/cargo.rs`'s existing-dependency branch now
        widens (rewrites to add the missing feature/`default-features`)
        instead of erroring, and only still hard-errors on a genuine
        version/`package` mismatch. Added
        `a_later_add_widens_an_existing_dependency_to_add_a_missing_feature`
        covering it.
      - **Correction to the fix above, found immediately while verifying
        `slider`'s own consumer-fixture reinstall:** the first version of
        `widen_existing_dependency` also OR'd `default_features` (`shape
        .default_features || requested.default_features`), reasoning it
        the same way as the feature-set union. That's wrong in the
        opposite direction from the original bug: this repo's own browser
        fixtures (`tests/installation/wave2-risk-consumer`,
        `wave5-color-picker-consumer`) hand-write `dioxus = { ...,
        default-features = false, features = [...] }` specifically so a
        runtime crate doesn't inherit fullstack/development-only default
        features, and `UnifiedCargoDependency::default_features` has no
        way to express "doesn't care either way" distinct from "wants
        them on" — so the very next `adico add` needing one more named
        feature on that crate silently flipped `default-features = false`
        back to enabled, reproduced live on both fixtures re-installing
        `slider`. Fixed by never touching `default_features` in the
        widened entry at all (always keep the existing value) — only the
        named `features` set is ever unioned. Added
        `widening_a_dependency_never_touches_an_explicit_default_features_false`
        covering it; reverted the two fixtures' incorrectly-rewritten
        `Cargo.toml` and reinstalled `slider` again to confirm the fixed
        version leaves them byte-identical.
- [ ] 5.2 **Wave 2 — menu/overlay family** (34 missing entries):
      `dropdown-menu` (5), `hover-card` (5), `popover` (6), `tooltip` (6),
      `toggle-group` (6), `context-menu` (3), `menubar` (3). Verify same
      as 5.1 for these seven items.
- [ ] 5.3 **Wave 3 — native leaf/form controls** (32 missing entries):
      `checkbox` (7), `switch` (6), `progress` (6), `input` (2),
      `textarea` (2), `toolbar` (2), `avatar` (2), `toggle` (1),
      `accordion` (1), `button` (1), `label` (1), `radio-group` (1).
      Verify same as 5.1 for these twelve items.
- [ ] 5.4 **Wave 4 — dialog/overlay content** (18 missing entries):
      `sheet` (6), `dialog` (3), `alert-dialog` (3), `drawer` (3),
      `command` (3). Verify same as 5.1 for these five items.
- [ ] 5.5 **Wave 5 — Dioxus-only/composite misc** (48 missing entries):
      `pagination` (12), `tag-group` (8), `color-picker` (4),
      `drag-and-drop-list` (4), `tabs` (4), `attachment` (3), `item` (2),
      `sidebar` (2), `bubble` (1), `input-otp` (1), `marker` (1), `toast`
      (6). Verify same as 5.1 for these twelve items.

**Per-wave fixture refresh:** end each of sections 1–5 by refreshing
`apps/playground`'s installed copy via `adico add --replace` (real CLI
path, never a direct file edit) for every item touched in that section,
then `cargo check --locked --workspace`. Waiting until task 6.2 to refresh
the playground copy means every intermediate check in sections 1–5
validates a stale installed copy — a compile break would be attributed to
the wrong section. `examples/*`/`tests/installation/*` still refresh once,
at task 6.2, since they aren't touched incrementally during a wave.

## 6. Propagate and validate

- [ ] 6.1 Re-run `cargo run -p adico-xtask -- registry build`,
      `primitive-usage sync`, `styling-usage sync`, `component-compat
      sync`, and `prop-parity sync` in that order, and commit the
      regenerated `statics/**`/`registry/generated/**` output. Verify
      every corresponding `check` command passes with zero drift.
- [ ] 6.2 Refresh every installed consumer fixture
      (`examples/basic-spa`, `examples/basic-ssr`,
      `tests/installation/*`) through `adico add --replace`, never by
      hand-editing copied source. Verify each fixture's `cargo check`/
      `cargo test` passes, and `cargo check --target wasm32-unknown-unknown`
      for the web-targeting fixtures.
- [ ] 6.3 Run `cd tests/playwright && npm test` for keyboard/axe coverage
      on every component whose interactive props changed in Waves 1–5 or
      the `**BREAKING**` controlled-trio migration (task 2.6). Verify all
      pass; report any surface with no existing fixture (per
      `docs/validation.md`'s recorded desktop/mobile gap) rather than
      claiming it passed.
- [ ] 6.4 Run the full baseline: `cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings` (the documented baseline;
      also run the wider `--workspace` form and report any failure
      outside this change's own files separately, per the precedent set
      in `extend-upstream-prop-evidence`), `cargo test --locked -p
      adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask`, and `openspec validate
      complete-component-prop-surface --strict`. Verify all pass; report
      any that don't and why.
