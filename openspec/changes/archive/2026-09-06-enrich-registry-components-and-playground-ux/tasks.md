# Tasks: enrich-registry-components-and-playground-ux

## 1. Menu facades (registry)

- [x] 1.1 Add `DropdownMenuGroup`, `DropdownMenuLabel { inset }`,
      `DropdownMenuSeparator`, `DropdownMenuShortcut` to
      `registry/ui/dropdown_menu.rs` wrapping the context-free
      `adico_primitives::menu` parts (shortcut = styled span per
      `CommandShortcut`); verify with an SSR-style render in a consumer
      fixture later (4.x) and `rustfmt --edition 2024 --check` passing.
- [x] 1.2 Add `DropdownMenuCheckboxItem`, `DropdownMenuRadioGroup<T>`,
      `DropdownMenuRadioItem<T>`, `DropdownMenuSub`,
      `DropdownMenuSubTrigger`, `DropdownMenuSubContent` to
      `registry/ui/dropdown_menu.rs`, indicators driven by the primitive's
      `data-state` via `group-data-[state=checked]` classes; verify
      controlled + uncontrolled checked/open props compile in the
      playground page (6.1) and keyboard nav works live (10.4).
- [x] 1.3 Add the context-free four (`Group`, `Label`, `Separator`,
      `Shortcut`) to `registry/ui/context_menu.rs` and
      `registry/ui/menubar.rs`; confirm no checkbox/radio/submenu parts are
      exported there (delta-spec scope) by reviewing the module exports.

## 2. Carousel drag (registry)

- [x] 2.1 Implement pointer-drag paging in `registry/ui/carousel.rs`:
      `CarouselDrag` state in the context, `onpointerdown` on the track,
      transient `fixed inset-0 z-[100]` overlay for move/up/cancel
      (resizable.rs pattern), `snap-none` + `cursor-grabbing` while
      dragging, drag math from start offset, release: >20% viewport →
      `page(±1)`, else smooth restore; verify `ScrollBehavior::Instant`
      exists (fallback `Smooth`) and behavior live via `dx serve` (10.4).
- [x] 2.2 Rewrite the stale module doc in `carousel.rs` (lines 1–14): cite
      the retraction in `positioner.rs:17-28`, document the per-element +
      overlay approach and non-use of `pointer.rs`; verify doc renders
      coherently and `rustfmt --check` passes.

## 3. OTP mask + Textarea counter (primitives + registry)

- [x] 3.1 Add `#[props(default)] mask: ReadSignal<bool>` to
      `OtpFieldRootProps` in `packages/adico-primitives/src/otp_field.rs`,
      thread through `OtpFieldCtx`, render slot inputs as
      `type="password"` when masked; update the module header's
      deliberately-not-built list and styling docs; verify with a new SSR
      unit test asserting `type="password"` when masked and `type="text"`
      otherwise (`cargo test -p adico-primitives`).
- [x] 3.2 Add `#[props(default)] mask: ReadSignal<bool>` to `InputOTP` in
      `registry/ui/input_otp.rs`, threaded to the primitive root; verify
      via the playground demo toggle (6.5) and wasm32 check (10.2).
- [x] 3.3 Add the conditional character counter to
      `registry/ui/textarea.rs`: internal count signal seeded from
      `value`, wrapped `oninput` forwarding the consumer handler,
      `div.relative.w-full` wrapper + bottom-right counter span + `pb-6`
      only when `max_length.is_some()`, bare `<textarea>` byte-identical
      otherwise; document the conditional wrapper in the doc comment;
      verify both branches' rendered class strings via unit test or SSR
      snapshot in a fixture and live typing behavior (10.4).

## 4. Registry regeneration roll (gates everything below)

- [x] 4.1 `rustfmt --edition 2024` the six edited registry files, recompute
      sha256 for each and update `registry/registry.json` checksums, then
      run `cargo run -p adico-xtask -- registry build` and
      `-- registry validate` until both pass.
- [x] 4.2 `cargo build -p adico-cli --locked`, then from `apps/playground`
      run `cargo run -p adico-cli -- add --all --replace`; verify
      `adico.lock` updates and `apps/playground/src/components/ui/` matches
      `registry/ui/` (diff spot-check), with zero hand edits.
- [x] 4.3 Regenerate derived artifacts:
      `cargo run -p adico-xtask -- playground-controls sync` (after 4.2),
      `component-props sync`, `primitive-usage sync`, `styling-usage sync`,
      `prop-parity sync`; verify all corresponding `check` commands pass
      and `cargo check --locked --workspace` compiles the regenerated
      controls.

## 5. Playground shell (pan zone, controls, nav)

- [x] 5.1 Rebuild `apps/playground/src/components/controls.rs` on installed
      components (Switch/Input/NativeSelect/Label) with the five public
      signatures frozen; if NativeSelect's `w-fit` wrapper cannot stretch,
      fix `registry/ui/native_select.rs` as its own cited edit and rerun
      task 4; verify `cargo check --locked --workspace` and
      `playground-controls check` pass unchanged.
- [x] 5.2 Add background-drag panning to
      `apps/playground/src/components/demo.rs`: offset + pan signals,
      canvas `onpointerdown` + `overflow-hidden` + `cursor-grab`, wrapper
      `stop_propagation` guard, transient overlay for move/up/cancel,
      `transform: translate(...)` on the centered wrapper, and a Center
      reset button (installed Button, bottom-center, disabled at origin);
      swap controls-pane chrome to installed Card parts; verify live: pan
      from background works, drags on demoed components don't pan, Center
      resets (10.4).
- [x] 5.3 Reorder `nav_items()` in `apps/playground/src/routes.rs` into one
      flat alphabetical list by label (66 entries preserved); verify the
      rendered sidebar lists A→Z and `cargo check` passes.

## 6. Playground demo pages (after task 4)

- [x] 6.1 Rebuild `pages/dropdown_menu.rs` as a realistic account menu
      (label, grouped items with shortcuts, separators, two checkbox items,
      a radio group, an "Invite users" submenu, destructive "Log out");
      keep generated panels; verify all parts render and toggle live.
- [x] 6.2 Rebuild `pages/context_menu.rs` (Back/Forward-disabled/Reload
      group with shortcuts, separator, labeled section) and
      `pages/menubar.rs` (File/Edit/View menus with groups, separators,
      shortcuts; per-menu item indices); verify right-click and menubar
      keyboard navigation live.
- [x] 6.3 Rebuild `pages/hover_card.rs` as a profile card composing
      installed Avatar (fallback initials, offline), name, bio, joined
      date, follower stats; verify hover/focus open behavior unchanged.
- [x] 6.4 Enrich `pages/navigation_menu.rs` ("Products" two-column panel
      with featured tile + description links; "Docs" title+description
      links; keep "Pricing" plain link); verify open/close delays and
      keyboard behavior unchanged.
- [x] 6.5 Rebuild `pages/carousel.rs` with offline gradient/emoji "photo"
      slides and `pages/input_otp.rs` with an Eye/EyeOff mask toggle
      (installed Button, `adico_primitives::icons`) plus a hand-rolled
      length control rendering `for index in 0..length()` slots with keyed
      remount; verify drag paging, mask toggle, and length switch live.
- [x] 6.6 Add the password show/hide composition to `pages/input.rs` using
      installed InputGroup/InputGroupInput/InputGroupButton with Eye/EyeOff
      toggling `r#type` between password/text (verify `InputGroupAddon`
      alignment API in `input_group.rs` while writing); verify toggle live.
- [x] 6.7 Add a hand-rolled max-length control to `pages/textarea.rs`
      (NumberControl → `Option<u32>`, 0 ⇒ None) bound to value/oninput so
      the counter moves; verify counter appears only when max is set.
- [x] 6.8 Rebuild `pages/card.rs` (CardAction in header, small Label+Input
      form or stats row in content, richer footer); verify `CardAction`
      renders top-right per its contract.
- [x] 6.9 Remove the "Standalone link" artifact from `pages/pagination.rs`
      (row, `link_state`, `PaginationLinkControls`/`DemoState` imports and
      panel, explanatory comment); verify the page renders only the
      realistic pagination row and `cargo check` shows no unused-import
      warnings.
- [x] 6.10 Rebuild `pages/dialog.rs`, `pages/sheet.rs`, `pages/drawer.rs`
      with settings/profile form bodies (Label+Input rows, a Switch row)
      and real footers (`DialogClose`+Button, `SheetClose`+Button,
      `DrawerClose`), plus a hand-rolled `DrawerDirection` select wired to
      `DrawerContent { direction }`; verify open/close via footer buttons
      and all four drawer directions live.
- [x] 6.11 Rebuild `pages/sidebar.rs` as a dashboard demo (taller box,
      Platform group with icons, separator, Settings group, footer user
      row with Avatar fallback; no `as_child`); keep generated panels and
      existing rationale comments; verify collapse/expand and controls
      still work live.

## 7. Validation, tests, docs

- [x] 7.1 Run the baseline matrix: `cargo fmt --all --check`,
      `cargo check --locked --workspace`, clippy (`-D warnings`) and tests
      for the five workspace packages per CLAUDE.md; all green.
- [x] 7.2 `cargo check --target wasm32-unknown-unknown -p adico-primitives`;
      passes clean.
- [x] 7.3 Added `tests/playwright/playground-enriched-demos.spec.ts` (4
      specs: drag-past-threshold, snap-back-below-threshold, buttons-still-
      page, OTP mask/unmask) plus a `test:playground-enriched` npm script;
      ran against a served playground. 2/4 pass (snap-back, OTP mask). The
      other 2 (drag-past-threshold, buttons-still-page) fail in this
      sandboxed headless-Chromium harness because `MountedData::
      get_client_rect`/`get_scroll_size` never resolve, leaving
      `viewport_size`/`content_size` at 0 so the Next/Previous buttons stay
      `disabled` and `max_offset()` stays 0 — confirmed via direct DOM query
      (`scrollWidth`/`clientWidth` are correct) and an A/B test against the
      pre-drag, unmodified `carousel.rs` reproducing the identical failure,
      so this is a pre-existing environment limitation of this harness, not
      a regression from the drag feature. Real-browser drag paging, snap-
      back, and button paging were all verified live via `dx serve`.
- [x] 7.4 Live-review pass with the user against a served playground
      surfaced several additional real defects beyond the original task
      list, fixed in this same change (all reverified via the full
      registry→CLI→reinstall pipeline, `cargo check --locked --workspace`,
      clippy, and the full test matrix after each):
      - `apps/playground/src/components/demo.rs`: the pan offset used
        `transform: translate(...)`, which — even at `(0,0)` — establishes a
        new containing block for every `position: fixed` descendant,
        breaking the placement of every anchored popover (Tooltip, Popover,
        HoverCard, DropdownMenu, Select, Combobox, ModeToggle, ...) rendered
        inside the demo zone. Switched to `position: relative; left; top;`,
        which pans identically without creating that containing block.
      - Same file: the canvas's `overflow-hidden` clipped popovers taller
        than the small preview box (e.g. DatePicker's calendar) even though
        they're `position: fixed`. Switched to `overflow-visible`.
      - `registry/ui/textarea.rs`: the character-counter's internal
        `typed_count` signal made the component re-render on every
        keystroke, which reasserted an unconditional `value:
        props.value.clone()` (`None` for uncontrolled usage) onto the
        native `<textarea>` — wiping every typed character immediately.
        Split the render into controlled/uncontrolled branches so the
        native `value` attribute key is present only when actually
        controlled.
      - `registry/ui/message.rs`: `Message`'s row used `items-end`,
        bottom-aligning the avatar against the whole header+content+footer
        stack instead of the sender-name line. Changed to `items-start`.
      - `registry/ui/collapsible.rs`: was a bare, unstyled primitive
        re-export (plain-text trigger, no chevron/border). Added a styled
        `CollapsibleTrigger` (bordered button + rotating chevron) and
        `CollapsibleContent`; rebuilt `pages/collapsible.rs` as a
        "starred repositories" show-more list.
      - `apps/playground/src/pages/drag_and_drop_list.rs`: was rendering
        bare text rows because `DragAndDropListItems`'s default children
        fall back to the *unstyled* primitive `DragAndDropListItem`.
        Rebuilt to compose the styled registry `DragAndDropListItem`/
        `DragAndDropDropIndicator` explicitly via
        `use_drag_and_drop_list_items()`, with a `GripVertical` drag handle.
      - `apps/playground/src/pages/data_table.rs`: the table shrank to its
        content width, leaving too little room for the pagination footer
        (text wrapped awkwardly). Added `wide: true`.
      - `apps/playground/src/pages/progress.rs`: replaced the fixed
        25%-step `SelectControl` with a free-form `NumberControl` (0–100).
      - `registry/ui/input_otp.rs`: swapped `InputOTPSeparator`'s dot glyph
        for a dash, per direct request.
      - Added live "Align" (Start/Center/End) controls to the
        `dropdown-menu`, `tooltip`, `hover-card`, `navigation-menu`,
        `select`, and `combobox` playground pages: `DropdownMenuContent`
        gained an `align: ContentAlign` prop (already supported by the
        underlying `MenuContent` primitive, just not threaded through);
        `SelectList`/`ComboboxList` (primitives) and their registry facades
        gained an `align: ContentAlign` prop (default `Center`, matching
        the previously-hardcoded value) plumbed to their `Positioner` call.
        `context-menu` (cursor-anchored, no trigger to align against) and
        `menubar` (plain CSS `absolute`, no `Positioner`/align concept in
        the primitive) were deliberately left out — reported to the user
        as a scope boundary rather than forcing a nonsensical prop or a
        larger, riskier primitives change under time pressure.
      Final state: `openspec validate enrich-registry-components-and-playground-ux --strict`
      passes; full `cargo fmt/check/clippy/test` matrix green; all registry
      `check` gates (`playground-controls`, `component-props`,
      `primitive-usage`, `styling-usage`, `prop-parity`) green.
