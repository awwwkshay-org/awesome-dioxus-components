# Existing component hardening audit

This record is the implementation ledger for `improve-existing-components`.
It covers the 21 `registry:ui` entries in `registry/registry.json`. Registry
source is the authority; every listed playground copy is refreshed with
`adico add --replace`, never edited as a second source of truth.

## Evidence and disposition keys

- **R** — registry-source remediation.
- **P** — `adico-primitives` remediation or retained primitive behavior.
- **D** — intentional Dioxus composition difference, documented rather than
  imitating a React-only renderer/slot API.
- **B** — named dependency block.

The comparison baseline is the current shadcn component documentation and the
owned Dioxus Components provenance recorded under `provenance/records/`.
Current primary references: [shadcn Button](https://ui.shadcn.com/docs/components/base/button),
[Pagination](https://ui.shadcn.com/docs/components/base/pagination),
[Select](https://ui.shadcn.com/docs/components/base/select),
[Combobox](https://ui.shadcn.com/docs/components/base/combobox),
[Calendar](https://ui.shadcn.com/docs/components/base/calendar),
[Date Picker](https://ui.shadcn.com/docs/components/date-picker), and
[Sidebar](https://ui.shadcn.com/docs/components/sidebar).

## Matrix

| Component | Source/API + variants/states | Theme / interaction / a11y | Responsive / docs / CLI / platforms | Disposition |
| --- | --- | --- | --- | --- |
| Button | Typed variants/sizes, composed children, native/global props | semantic hover/focus/disabled/link states | live variants/content controls; CLI/refreshed web+server copy | R |
| Badge | Default, secondary, destructive, outline, verified variants | semantic token/dark-mode-compatible status surfaces | variant/content controls; CLI/refreshed | R |
| Card | Attribute-forwarding structural header/content/footer parts | card semantic surfaces and composed Button actions | live title/description/action controls; CLI/refreshed | R |
| Input | typed value/type/placeholder/native invalid/disabled/read-only props | semantic invalid/focus/disabled/read-only states | live field controls; CLI/refreshed | R |
| Textarea | typed value/rows/placeholder/native invalid/disabled/read-only props | semantic invalid/focus/disabled/read-only states | live field controls; CLI/refreshed | R |
| Skeleton | default/circle shapes and reduced-motion treatment | decorative by default; opt out explicitly | shape/decorative controls; CLI/refreshed | R |
| Item | composed slots and default/muted/interactive/disabled states | semantic focus/disabled states | state controls; CLI/refreshed | R |
| Pagination | semantic landmark/current link, native link attributes, labels/compact mode | keyboard-native anchors and current-page ARIA | active/labels/compact controls; CLI/refreshed | R |
| Dialog | installed Button trigger, overlay/content/header parts | primitive focus, Escape, dismissal, ARIA; opaque layered overlay | controlled-open workbench route | P/R |
| Sheet | installed Button trigger and four-side content | primitive focus, Escape, dismissal, ARIA; opaque layered overlay | live side control | P/R |
| Select | styled parts; controlled single/multi values and open/invalid/disabled | primitive keyboard/typeahead/ARIA, opaque absolute popup | live single/multi/open controls; CLI/refreshed | P/R |
| Combobox | styled parts; controlled single/multi/query/open/disabled values | primitive filter/keyboard/ARIA, opaque absolute popup | live single/multi/open controls; CLI/refreshed | P/R |
| Tooltip | styled trigger/content placement surface | primitive hover/focus/Escape/ARIA | controlled-open/disabled route | P/R |
| Popover | styled trigger/content placement surface | primitive focus, outside dismissal, ARIA | open/alignment route | P/R |
| Hover Card | styled trigger/content placement surface | primitive pointer/focus/ARIA | controlled-open/disabled route | P/R |
| Dropdown Menu | styled root/trigger/content/item parts | primitive roving focus/Escape/dismissal/ARIA; opaque absolute popup | open/disabled route; CLI/refreshed | P/R |
| Context Menu | styled trigger/content/item parts | primitive pointer/keyboard/focus/dismissal/ARIA | open/disabled route | P/R |
| Menubar | styled bar/trigger/content/item parts | primitive roving-focus/keyboard/ARIA | canonical composition route | P/R |
| Calendar | styled single/range roots with all primitive parts retained | primitive keyboard/focus/ARIA; tokenised selected/unavailable states | selection/disabled/first-day controls; CLI/refreshed | R/P |
| Date Picker | styled typed single/range roots retaining primitive inputs/popovers | primitive constraints/focus/ARIA; tokenised selected, layered popup | disabled/read-only controls; CLI/refreshed | R/P |
| Sidebar | controlled provider/side/collapse, structural/menu parts, active/disabled menu item | semantic focus/pointer states | live state controls; CSS desktop disposition documented | R/D |

## First-wave control contract

Every route owns its state and declares the controls it deliberately exposes.
The shared controls render `ControlDefinition<T>` values, where `T` is the
route's concrete prop type. The renderer supports boolean, text, number, and
closed-option fields. A route may attach an unavailable reason for a supported
public prop that should not be demonstrated (for example, a callback whose
side effect would not be observable in the preview).

| Route | Live controls | Intentional Dioxus differences |
| --- | --- | --- |
| Button | variant, size, disabled, native type, text/icon composition | icons remain caller-composed children |
| Pagination | active page, previous/next labels, compact presentation | normal anchors/events replace a React renderer prop |
| Select | value, open, disabled, invalid presentation | primitive composition replaces item-array APIs |
| Combobox | value, open, disabled, query/default example | primitive composition replaces command/render APIs |
| Calendar | selection, disabled, first day of week, view month | typed `time::Date` state replaces JavaScript Date |
| Date Picker | selection, disabled, read-only, popover alignment | typed `time::Date` state replaces JavaScript Date |
| Sidebar | open, side, collapsible mode, active item | mobile Sheet behavior is unavailable pending viewport support |

Badge, Card, Input, Textarea, Skeleton, and Item have dedicated variation
controls. Dialog/Sheet and every popup/menu route exposes the safe live state
needed to inspect its primitive behavior. A menu’s check/radio/submenu API is
not fabricated in registry source where the owned primitive has no equivalent
part; that remains a documented primitive extension, rather than a misleading
visual-only prop.

## Completion evidence

Each first-wave row requires source API review, its declared live playground
controls, CLI-refreshed copied source and checksums, focused Rust/consumer
compile evidence, applicable keyboard/a11y checks, web and server feature
checks, plus a recorded reason for each unavailable browser, desktop, or
hydration check.

The following log carries a row only for a component whose ledger closure
required its own dedicated evidence write-up (sections 2, 4, and 6). Badge,
Card, Input, Textarea, Skeleton, Item, and Pagination (section 5 and 3) and
Select, Combobox, and Sidebar (section 4) are equally closed; their evidence
lives in the Matrix and control-contract tables above plus their task
records, not as a separate row here.

| Component | Rust | Consumer compile | Browser / keyboard / a11y | Notes |
| --- | --- | --- | --- | --- |
| Button | `every_public_variant_has_a_distinct_semantic_class`, `icon_sizes_remain_square` unit tests in `registry/ui/button.rs` | `adico-playground` `web`/`server` features and `wasm32-unknown-unknown` all pass | Not warranted as a dedicated Playwright spec: Button is a styled native `<button>` with no custom JS/ARIA behavior (same disposition as Badge/Card/Input/Textarea/Skeleton/Item, none of which carry a spec either); keyboard activation (Space/Enter) and `disabled`/`aria-invalid` semantics are native HTML, and its trigger composition is exercised live by `dialog.spec.ts` (`Open dialog` / `Open nested dialog` buttons), which passes with zero critical axe violations | Desktop/hydration: unavailable, no desktop or SSR-hydration fixture exists for Button in isolation; covered transitively by the workspace-wide `server` feature check |
| Calendar | `adico-primitives` calendar doctests (`Calendar`, `CalendarGrid`, `RangeCalendar`, `CalendarSelectMonth`/`CalendarSelectYear`, etc.) | `adico-playground` `web`/`server` features and `wasm32-unknown-unknown` pass; `tests/installation/wave4-consumer` `cargo build`/`cargo check --target wasm32-unknown-unknown` pass | `tests/playwright/wave4.spec.ts` (`installed Calendar navigates and selects dates with arrow keys`) passed live against `wave4-consumer` via `dx serve`; zero critical axe violations in the same run | Desktop/hydration: unavailable, no desktop/SSR-hydration fixture; registry-source drift found during closure (playground copy had hand-authored month/year select pills not present in `registry/ui/calendar.rs`) was reconciled by porting the fix into registry source and refreshing every installed copy through `adico add --replace` |
| Date Picker | `adico-primitives` date_picker doctests (`DatePicker`, `DatePickerPopover`, `DatePickerCalendar`, `DateRangePicker`, etc.) | `adico-playground` `web`/`server` features and `wasm32-unknown-unknown` pass; `tests/installation/wave4-consumer` `cargo build`/`cargo check --target wasm32-unknown-unknown` pass | `tests/playwright/wave4.spec.ts` (`installed DatePicker opens a Calendar inside its popover`) passed live against `wave4-consumer` via `dx serve`; zero critical axe violations in the same run | Desktop/hydration: unavailable, no desktop/SSR-hydration fixture; no registry-source drift found (`date_picker.rs` copies matched registry source) |
| Dialog | Shared `adico-primitives::dialog` doctests (`DialogRoot`, `DialogContent`, `DialogTitle`, `DialogDescription`) | `adico-playground` `web`/`server`/`wasm32-unknown-unknown` pass; `tests/installation/dialog-consumer` `cargo build`/`cargo check --target wasm32-unknown-unknown` pass | `tests/playwright/dialog.spec.ts` (3 tests: open/ARIA/focus-restore/Escape, outside-interaction + axe, nested-dialog layered Escape) passed live against a freshly reinstalled `dialog-consumer` via `dx serve`; zero critical axe violations | Desktop/hydration: unavailable; no registry-source drift found |
| Sheet | Reuses the `adico-primitives::dialog` primitive (`DialogContent`/`DialogCtx`) directly, so its layer/focus/Escape/dismissal/ARIA behavior is the same code path `dialog.spec.ts` exercises | `adico-playground` `web`/`server`/`wasm32-unknown-unknown` pass | No dedicated Playwright spec: Sheet is a styled four-side variant of the same primitive Dialog uses, not a separate behavior surface; side/trigger/overlay composition is verified through the playground's live "Side" control | Desktop/hydration: unavailable; registry-source formatting drift (playground had reformatted a manual `impl Default for SheetSide` into a derive) reconciled by copying the current, correctly-formatted source back into `registry/ui/sheet.rs` |
| Tooltip | `adico-primitives` tooltip doctests (`Tooltip`, `TooltipContent`, `TooltipTrigger`) | `adico-playground` `web`/`server`/`wasm32-unknown-unknown` pass; `tests/installation/wave3-consumer` `cargo build`/`cargo check --target wasm32-unknown-unknown` pass | `tests/playwright/wave3.spec.ts` (`installed Tooltip shows on hover with ARIA association and hides on mouse leave`) passed live against a freshly reinstalled `wave3-consumer`; zero critical axe violations in the suite-wide scan | Desktop/hydration: unavailable |
| Popover | `adico-primitives` popover doctests (`PopoverRoot`) | same as Tooltip | `tests/playwright/wave3.spec.ts` (`installed Popover opens on click, exposes dialog semantics, and closes with Escape`) passed live | Desktop/hydration: unavailable |
| Hover Card | `adico-primitives` hover_card doctests (`HoverCard`) | same as Tooltip | `tests/playwright/wave3.spec.ts` (`installed HoverCard shows on hover and hides on mouse leave`) passed live | Desktop/hydration: unavailable |
| Dropdown Menu | `adico-primitives` dropdown_menu doctests (`DropdownMenu`) | same as Tooltip | `tests/playwright/wave3.spec.ts` (`installed DropdownMenu opens with roving-focus keyboard navigation and closes on selection`) passed live | Desktop/hydration: unavailable; registry-source drift (formatting only) and the missing `cn` registry dependency (see below) both fixed |
| Context Menu | `adico-primitives` context_menu doctests (`ContextMenu`) | same as Tooltip | `tests/playwright/wave3.spec.ts` (`installed ContextMenu opens on right-click, navigates by keyboard, and closes with Escape`) passed live | Desktop/hydration: unavailable; no registry-source drift found |
| Menubar | `adico-primitives` menubar doctests (`Menubar`) | same as Tooltip | `tests/playwright/wave3.spec.ts` (`installed Menubar opens a menu on click and selects an item`) passed live | Desktop/hydration: unavailable; the registry façade's public `Menubar` component silently dropped the primitive's `disabled` prop (no way to render a disabled menubar at all despite the primitive supporting it) — fixed by forwarding it, and the playground route, which previously had no `controls:` block at all, gained a live "Disabled" toggle |

Check/radio/submenu item variants are not fabricated for Dropdown Menu,
Context Menu, or Menubar: `packages/adico-primitives/src/{dropdown_menu,
context_menu,menubar}.rs` expose no check/radio/submenu-flavored parts today,
so there is nothing in the owned primitive layer for a styled façade to wrap.
This remains a documented primitive-extension gap (see "First-wave control
contract" above), not a registry-source omission — closing it would mean
extending `adico-primitives` first, which is out of scope for a styled-façade
hardening pass. All eight overlay/menu components were also grepped for
hardcoded colors (`bg-[#`, `rgb(`, `hsl(`, hex literals) and use semantic
tokens exclusively (`bg-popover`, `text-popover-foreground`, `bg-accent`,
etc.), so they inherit theme-reactivity structurally rather than needing
individual theme wiring: `apps/playground/src/theme.rs`'s customization tray
writes the same `--popover`/`--popover-foreground`/`--accent`/etc. CSS custom
properties these Tailwind classes resolve through (see its `CssVariable`
mapping and the generated inline `style` covering the full semantic set), so
any live palette or appearance change reaches every overlay/menu surface
without per-component changes.

Closing Calendar's ledger also surfaced two pre-existing registry metadata
defects, fixed alongside it since they broke the same standalone-install path
being verified: `select` and `combobox` (both closed under tasks 4.1/4.2) and
`dropdown-menu` (closed above, part of this overlay/menu batch) used
the `cn` helper in their copied source but omitted `cn` from
`registryDependencies`, so `adico add select` (or `combobox`, `dropdown-menu`)
alone — without another item that happens to declare `cn` — installed a
project that failed to compile. Fixed in `registry/registry.json`, covered by
the new `standalone_add_of_items_using_cn_resolves_the_cn_dependency`
regression test in `packages/adico-cli/tests/cli_integration.rs`, and verified
live via `select.spec.ts` against a freshly reinstalled `select-consumer`.

Lucide icons (used by Calendar, Select, Combobox, and Date Picker) were also
centralized behind `adico_primitives::icons` instead of each registry
component and consumer depending on `dioxus-icons` directly, since every one
of those items already requires `adico-primitives`.

## Hardening report

All 21 `registry:ui` components in the Matrix above are closed: Button
(section 2), Pagination (section 3), Select/Combobox/Sidebar (section 4),
Badge/Card/Input/Textarea/Skeleton/Item (section 5), Calendar/Date Picker
(section 4), and Dialog/Sheet/Tooltip/Popover/Hover Card/Dropdown
Menu/Context Menu/Menubar (section 6). No component is blocked. The one deliberate, documented gap is check/radio/submenu menu item
variants (Dropdown Menu, Context Menu, Menubar): the owned primitives expose
no such parts, so a styled façade has nothing to wrap without first extending
`adico-primitives` — out of scope for this styled-façade hardening change.

Unavailable checks, applied uniformly rather than per component:

- **Desktop**: no desktop-targeted consumer fixture exists for any of the 21
  components; `adico-primitives --features desktop` compiles (verified below)
  but no fixture exercises desktop-specific interaction.
- **SSR hydration**: no hydration-specific fixture exists; the `server`
  feature check verifies the server-rendering code path compiles, not a live
  hydration run.
- **Visual regression**: no visual-regression tooling exists yet in this repo
  (`tests/visual` is `build-adico-component-ecosystem` M4 scope, tracked
  separately in `parity.json`, out of scope for this change).

Final validation, run against the complete working tree after section 6:

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | passed |
| `cargo xtask registry build` / `registry validate` | 22 item payload(s), passed |
| `cargo xtask provenance check` | 3 imported record(s), 34 source unit(s), passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed, all suites, default parallel execution. A pre-existing flake was found and fixed along the way: `adico-cli`'s `css::tests` module derived its temp directory from a nanosecond timestamp alone, which could collide across threads on platforms with coarser clock resolution; `temporary_project_root()` now also mixes in an atomic counter so every call gets a distinct path |
| `adico-primitives` `web`/wasm32 and `desktop` feature checks | both passed |
| `adico-playground` `web`/wasm32 and `server` feature checks | both passed |
| `dialog.spec.ts` (3 tests) against a freshly reinstalled `dialog-consumer` via `dx serve` | passed, zero critical axe violations |
| `select.spec.ts` (2 tests) against a freshly reinstalled `select-consumer` via `dx serve` | passed |
| `wave3.spec.ts` (7 tests) against a freshly reinstalled `wave3-consumer` via `dx serve` | passed, zero critical axe violations |
| `wave4.spec.ts` (5 tests) against a freshly reinstalled `wave4-consumer` via `dx serve` | passed, zero critical axe violations |
| `git diff --check` | passed |
| `openspec validate improve-existing-components --strict` | passed |

New component work on `build-adico-component-ecosystem` (Wave 2 migration,
paused at task 4.8e for this hardening pass) may resume.
