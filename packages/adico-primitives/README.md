# adico-primitives

Owned, headless runtime behavior for [Awesome Dioxus Components](../../README.md) (`adico`) —
a shadcn-style, source-installed component ecosystem for [Dioxus](https://dioxuslabs.com).

This crate is the one thing in the ecosystem that consumers depend on as an ordinary compiled
dependency rather than copying into their own source tree. Everything else — the styled
components under [`registry/`](../../registry/) — is installed *source*, and that installed
source calls into this crate for the behavior it doesn't want to re-implement per component:
open/close state, keyboard interaction, focus management, positioning, roving-tabindex
collections, and so on.

```
Dioxus → adico-primitives → registry source + metadata → adico CLI/registry-core → consumer app
```

If you're building or customizing an installed component, you'll mostly be reading this
crate's docs, not this crate's source — `cargo doc -p adico-primitives --open` is the fastest
way to browse every primitive's props, ARIA contract, and example.

## Not a styled component library

Every primitive here is **headless**: it owns state, keyboard/pointer interaction, ARIA
attributes, and (where relevant) positioning — but ships no CSS and no visual opinion beyond
`data-*`/`aria-*` attributes for a consumer's own stylesheet to hook into. Styling lives one
layer up, in the registry components under `registry/ui/` that compose these primitives and
that consumers install (and then own) via the `adico` CLI.

## Quick example

```rust
use dioxus::prelude::*;
use adico_primitives::{tooltip::{Tooltip, TooltipContent, TooltipTrigger}, ContentSide};

#[component]
fn Demo() -> Element {
    rsx! {
        Tooltip {
            TooltipTrigger { "Hover me" }
            TooltipContent {
                side: ContentSide::Left,
                "Rich, positioned, keyboard-dismissible content."
            }
        }
    }
}
```

Every component follows the same controlled/uncontrolled shape: an optional `open`/`value`
(`ReadSignal<Option<T>>`), a `default_open`/`default_value` for the uncontrolled case, and an
`on_open_change`/`on_value_change` callback — see [`use_controlled`](#hooks) below.

## Feature flags

| Flag | Default | Enables |
| --- | --- | --- |
| *(none)* | ✔ | Pure logic and an SSR-safe fallback for anything that would otherwise need a browser/webview — the path this crate's own test suite runs against. |
| `web` | | Browser DOM interop (via `dioxus-document`/`document::eval`) for WebAssembly consumers — real focus trapping, clipboard writes, animated-open timing, OS theme detection. |
| `native` | | The same interop surface as `web`, targeting Dioxus desktop/mobile's embedded WebView (`wry`/`tao`) instead of a browser. Named `native`, not `desktop`, because desktop and mobile share one WebView-based interop path. |
| `test-utils` | | Narrowly-scoped test adapters consumed by `adico-test-utils`; not needed by application consumers. |

A consumer app enables exactly one of `web`/`native` to match its Dioxus renderer — never
both, and never assume one without checking, since browser-only interop must stay behind a
target-aware adapter rather than leaking into a primitive's public API.

## What's here

### Component primitives (`src/*.rs`, one file per component)

Each of these is a complete, independently re-authored, tested surface — not a thin wrapper
over shared internals. A handful are pure facades over another primitive where that's a
better fit than duplicating it (`dropdown_menu` re-exports `menu`; `autocomplete` re-exports
most of `combobox`; `preview_card` supplies its own delay/positioning defaults over
`hover_card`) — see [Conventions](#conventions). "Exposed components" lists every `#[component]`
the module exports (its full public surface, not just the root).

| Module | Exposed components |
| --- | --- |
| `accordion` | `Accordion`, `AccordionMulti`, `AccordionItem`, `AccordionTrigger`, `AccordionContent` |
| `alert_dialog` | `AlertDialogRoot`, `AlertDialogContent`, `AlertDialogTitle`, `AlertDialogDescription`, `AlertDialogActions`, `AlertDialogAction`, `AlertDialogCancel` |
| `aspect_ratio` | `AspectRatio` |
| `autocomplete` | `AutocompleteStatus`, `AutocompleteClear`, plus `combobox`'s own components re-exported as `AutocompleteRoot`/`AutocompleteInput`/`AutocompleteList`/`AutocompleteItem`/`AutocompleteEmpty` |
| `avatar` | `Avatar`, `AvatarImage`, `AvatarFallback` |
| `calendar` | `Calendar`, `RangeCalendar`, `CalendarHeader`, `CalendarNavigation`, `CalendarPreviousMonthButton`, `CalendarNextMonthButton`, `CalendarMonthTitle`, `CalendarSelectMonth(Select\|Option\|Value)`, `CalendarSelectYear(Select\|Option\|Value)`, `CalendarView`, `CalendarGridRoot`, `CalendarGrid`, `CalendarGridHead`, `CalendarGridHeaderRow`, `CalendarGridDayHeader`, `CalendarGridBody`, `CalendarGridWeek`, `CalendarGridCell`, `CalendarDay` |
| `checkbox` | `Checkbox`, `CheckboxIndicator` |
| `checkbox_group` | `CheckboxGroupRoot` |
| `collapsible` | `Collapsible`, `CollapsibleTrigger`, `CollapsibleContent` |
| `color_picker` | `ColorPicker`, `ColorArea`, `AreaTrack`, `AreaThumb`, `AreaThumbSaturationInput`, `AreaThumbValueInput` |
| `combobox` | `Combobox`, `ComboboxMulti`, `ComboboxInput`, `ComboboxList`, `ComboboxOption`, `ComboboxItemIndicator`, `ComboboxEmpty` |
| `command` | `CommandRoot`, `CommandInput`, `CommandList`, `CommandGroup`, `CommandItem`, `CommandSeparator`, `CommandEmpty` |
| `context_menu` | `ContextMenu`, `ContextMenuTrigger`, `ContextMenuContent`, `ContextMenuItem` |
| `date_picker` | `DatePicker`, `DateRangePicker`, `DatePickerInput`, `DatePickerInputValue`, `DatePickerMonthSegment`, `DatePickerDaySegment`, `DatePickerYearSegment`, `DatePickerSeparator`, `DatePickerPopover`, `DatePickerCalendar`, `DateRangePickerInput`, `DateRangePickerInputValue`, `DateRangePickerStartValue`, `DateRangePickerEndValue`, `DateRangePickerCalendar` |
| `dialog` | `DialogRoot`, `DialogContent`, `DialogTitle`, `DialogDescription` |
| `direction` | `DirectionProvider` |
| `drag_and_drop_list` | `DragAndDropList`, `DragAndDropListItems`, `DragAndDropListItem`, `DragAndDropDropIndicator`, `DragAndDropInstructions`, `DragAndDropLiveRegion` |
| `dropdown_menu` | Re-exports `menu`'s components as `DropdownMenu`, `DropdownMenuTrigger`, `DropdownMenuContent`, `DropdownMenuItem` |
| `field` | `FieldRoot`, `FieldLabel`, `FieldDescription`, `FieldError` |
| `fieldset` | `FieldsetRoot`, `FieldsetLegend` |
| `form` | `FormRoot` |
| `gesture` | *(no components — `use_long_press` hook plus the pure `moved_past_threshold` helper)* |
| `hover_card` | `HoverCard`, `HoverCardTrigger`, `HoverCardContent` |
| `hover_intent` | *(no components — `use_hover_intent` hook and its `HoverIntent<T>` return type)* |
| `label` | `Label` |
| `menu` | `Menu`, `MenuTrigger`, `MenuContent`, `MenuItem`, `MenuCheckboxItem`, `MenuRadioGroup`, `MenuRadioItem`, `MenuGroup`, `MenuGroupLabel`, `MenuSeparator`, `MenuSubmenuRoot`, `MenuSubmenuTrigger` |
| `menubar` | `Menubar`, `MenubarMenu`, `MenubarTrigger`, `MenubarContent`, `MenubarItem` |
| `message_scroller` | `MessageScroller`, `MessageScrollerViewport`, `MessageScrollerContent` |
| `meter` | `MeterRoot`, `MeterLabel`, `MeterValue`, `MeterTrack`, `MeterIndicator` |
| `navigation_menu` | `NavigationMenuRoot`, `NavigationMenuList`, `NavigationMenuItem`, `NavigationMenuTrigger`, `NavigationMenuContent`, `NavigationMenuLink` |
| `number_field` | `NumberFieldRoot`, `NumberFieldGroup`, `NumberFieldInput`, `NumberFieldIncrement`, `NumberFieldDecrement` |
| `otp_field` | `OtpFieldRoot`, `OtpFieldInput`, `OtpFieldSeparator` |
| `popover` | `PopoverRoot`, `PopoverTrigger`, `PopoverContent`, `PopoverContentRendered` |
| `preview_card` | `PreviewCard`, `PreviewCardTrigger`, `PreviewCardContent` |
| `radio_group` | `RadioGroup`, `RadioItem` |
| `scroll_area` | `ScrollArea` |
| `select` | `Select`, `SelectMulti`, `SelectTrigger`, `SelectValue`, `SelectList`, `SelectGroup`, `SelectGroupLabel`, `SelectOption`, `SelectItemIndicator` |
| `separator` | `Separator` |
| `slider` | `Slider`, `RangeSlider`, `SliderTrack`, `SliderRange`, `SliderThumb` |
| `switch` | `Switch`, `SwitchThumb` |
| `tabs` | `Tabs`, `TabList`, `TabTrigger`, `TabContent` |
| `tag_group` | `TagGroup`, `TagGroupMulti`, `TagList`, `TagOption`, `TagRemoveButton`, `TagGroupLabel`, `TagGroupEmpty` |
| `theme_mode` | *(no components — `use_theme_mode`/`use_persisted_theme_mode` hooks)* |
| `time_picker` | `TimePicker`, `TimePickerInput`, `TimePickerInputValue`, `TimePickerHourSegment`, `TimePickerMinuteSegment`, `TimePickerSecondSegment`, `TimePickerMeridiemSegment` |
| `toast` | `ToastProvider`, `ToastList`, `ToastListItem`, `Toast`, `ToastTitle`, `ToastDescription`, `ToastContent`, `ToastCloseButton` |
| `toggle` | `Toggle` |
| `toggle_group` | `ToggleGroup`, `ToggleItem` |
| `toolbar` | `Toolbar`, `ToolbarButton`, `ToolbarSeparator` |
| `tooltip` | `Tooltip`, `TooltipTrigger`, `TooltipContent` |
| `typeahead` | *(no components — `use_typeahead` hook and its matching helpers)* |
| `virtual_list` | `VirtualList` |

### Shared support primitives

Behavior used by more than one component primitive lives in its own top-level module here,
never inlined into every consumer — see [Conventions](#conventions). Most of these expose
hooks rather than components; see [Hooks](#hooks) for signatures.

| Module | Exposed surface | What it's for |
| --- | --- | --- |
| `clipboard` | `use_clipboard` hook | Target-gated clipboard-copy support. |
| `collection` | `use_collection_provider`, `use_collection_provider_with`, `use_item`, `use_deferred_collection_focus` hooks | Ordered interactive collection state for roving-focus components. |
| `layer` | `use_layer`, `use_layer_member` hooks | Shared open/close overlay stacking, so nested overlays (dialog → tooltip) know which one is topmost for Escape/outside-dismiss. |
| `listbox` | `use_listbox_id`, `use_listbox_render`, `use_listbox_container`, `use_listbox_option` hooks; `ListboxItemIndicator` component | Shared listbox popup hooks. |
| `move_interaction` | `use_move_interaction` hook | Shared pointer-drag and arrow-key movement for track-style controls (slider, color picker). |
| `persisted_state` | `use_persisted_global` hook | Generic persisted-`GlobalSignal` primitive (backs `theme_mode`). |
| `pointer` | `track_pointer_down`, `pointer_position`, `upsert_pointer` functions; `Pointer` struct | Global pointer-position registry backing `gesture`/`move_interaction`'s tracking. |
| `portal` | `use_portal` hook; `PortalIn`/`PortalOut` components | Logical, VDOM-only content relay. |
| `positioner` | `Positioner`, `Arrow` components | Collision-aware anchored-placement engine (Base UI's floating-content model) backing every popup/tooltip/menu/select. |
| `scroll_lock` | `use_scroll_lock` hook | Reference-counted page scroll lock. |
| `segment` | `use_segment_field_provider` hook; `NumericSegment`/`MeridiemSegment` components | Generic numeric/meridiem segmented-field input, backing `time_picker`/`date_picker`. |
| `selectable` | `use_single_selectable_value`, `use_selectable_root`, `use_selectable_option` hooks | Shared state for select-like listbox components. |
| `selection` | `OptionState`/`RcPartialEqValue` structs; `selected_text`/`selected_texts`/`sync_option`/`remove_option` functions | Shared option-selection state and text helpers used by `select`/`combobox`/`tag_group`. |
| `time` | `sleep` (target-aware) | Cancelable delay used wherever a primitive needs a timeout (`wasm` vs. native). |

## Hooks

### Crate-root (`use adico_primitives::{...}`)

The load-bearing patterns every primitive above is built from:

| Hook | Signature | Purpose |
| --- | --- | --- |
| `use_controlled` | `<T>(value: ReadSignal<Option<T>>, default: ReadSignal<T>, on_change: Callback<T>) -> (Memo<T>, Callback<T>)` | The standard controlled/uncontrolled state hook. |
| `use_optionally_controlled` | `<T>(controlled: Option<ReadSignal<Option<T>>>, default: Option<T>, on_change: Callback<Option<T>>) -> (Memo<Option<T>>, Callback<Option<T>>)` | The same shape for state that is itself optional (an accordion that can have *nothing* selected while still being controlled) — `use_controlled`'s single-level `Option` can't distinguish "controlled, currently empty" from "uncontrolled" the way this can. |
| `use_unique_id` | `() -> Signal<String>` | Runtime-unique ARIA-relationship id generator. |
| `use_id_or` | `<T>(generated_id: Signal<T>, user_id: ReadSignal<Option<T>>) -> Memo<T>` | Resolves to a caller-supplied id when given, `use_unique_id`-generated otherwise. |
| `use_escape_key` | `(open: impl Readable<Target = bool>, on_escape: impl FnMut() + Clone) -> impl FnMut(Event<KeyboardData>) + Clone` | Escape-key dismissal gated on `layer`'s shared stack, so only the topmost overlay reacts. |
| `use_outside_dismiss` | `(id: impl Readable<Target = String>, on_dismiss: impl FnMut() + Clone)` | Click-outside dismissal, scoped to the element with this id. |
| `use_animated_open` | `(id: impl Readable<Target = String>, open: impl Readable<Target = bool>) -> impl Fn() -> bool + Copy` | Target-aware open/close mount timing for exit animations. |
| `use_focus_trap` | `(id: Memo<String>, open: Memo<bool>, is_modal: ReadSignal<bool>)` | Modal focus containment (paired with the `FocusTrapScript` component). |

Also at the crate root: the `ContentSide`/`ContentAlign` placement enums every `positioner`-backed
primitive's content prop uses, and the `Controlled<T>` struct some hooks accept in place of three
separate arguments.

### Per-module hooks

| Hook | Module | Returns |
| --- | --- | --- |
| `use_calendar_grid` | `calendar` | `CalendarGridData` |
| `use_clipboard` | `clipboard` | `(Memo<ClipboardStatus>, Callback<String>)` |
| `use_collection_provider` / `use_collection_provider_with` | `collection` | `CollectionState` |
| `use_item` | `collection` | `CollectionItem` |
| `use_deferred_collection_focus` | `collection` | — |
| `use_checkbox_group_item` | `checkbox_group` | `(Memo<bool>, Callback<bool>)` |
| `use_checkbox_group_parent` | `checkbox_group` | `(Memo<CheckboxState>, Callback<()>)` |
| `use_checkbox_group_disabled` | `checkbox_group` | `ReadSignal<bool>` |
| `use_drag_and_drop_list_items` | `drag_and_drop_list` | `Vec<DragAndDropListRenderItem>` |
| `use_direction` | `direction` | `Direction` |
| `use_field_control` | `field` | `FieldControlBinding` |
| `use_fieldset_disabled` | `fieldset` | `ReadSignal<bool>` |
| `use_hover_intent` | `hover_intent` | `HoverIntent<T>` (generic; see the crate's [`hover_intent`] docs) |
| `use_listbox_id` / `use_listbox_render` / `use_listbox_container` / `use_listbox_option` | `listbox` | — |
| `use_message_scroller_pinned_to_bottom` | `message_scroller` | `Memo<bool>` |
| `use_message_scroller_scroll_to_bottom` | `message_scroller` | `Callback<()>` |
| `use_move_interaction` | `move_interaction` | `MoveInteraction` |
| `use_long_press` | `gesture` | `LongPress` |
| `use_persisted_global` | `persisted_state` | generic, `<T>(...) -> (Memo<T>, Callback<T>)`-shaped |
| `use_layer` | `layer` | `Layer` |
| `use_layer_member` | `layer` | `Layer` |
| `use_portal` | `portal` | `PortalId` |
| `use_segment_field_provider` | `segment` | — (context provider) |
| `use_single_selectable_value` | `selectable` | generic |
| `use_selectable_root` | `selectable` | — |
| `use_selectable_option` | `selectable` | generic |
| `use_toast` | `toast` | `Toasts` |
| `use_scroll_lock` | `scroll_lock` | — |
| `use_theme_mode` | `theme_mode` | — |
| `use_persisted_theme_mode` | `theme_mode` | `(Memo<ThemeMode>, Callback<ThemeMode>)` |
| `use_clock_dial` | `time_picker` | `ClockDial` |
| `use_typeahead` | `typeahead` | `Typeahead` |

`cargo doc -p adico-primitives --open` is the authoritative reference for every hook's full
signature and doc comment — the tables above are a map of what exists and where, not a
substitute for the generated docs.

## Conventions

This crate follows a small set of rules recorded in
[`openspec/specs/adico-primitives-authorship`](../../openspec/specs/adico-primitives-authorship/spec.md)
and [`openspec/specs/adico-primitives`](../../openspec/specs/adico-primitives/spec.md):

- **One primitive, one file.** A primitive's public module is a single `.rs` file under
  `src/`, never a directory of private sub-modules. Behavior genuinely shared across more than
  one primitive gets its own top-level file instead of being copied into every consumer.
- **Independently specified, not ported.** Each primitive's observable behavior (roles,
  keyboard interaction, states) is derived from the relevant WAI-ARIA APG pattern (or the
  closest applicable guidance) and this repo's own compatibility inventories
  (`statics/catalogs/*.json`, `statics/primitive_compatibility.json`) — not copied from an
  upstream implementation.
- **Test-before-rewrite.** A primitive is never rewritten without test coverage pinning its
  specified behavior first.
- **Tests live under `tests/`, never inline.** Every test is a black-box integration test
  against the crate's public API in `packages/adico-primitives/tests/*.rs` — there is no
  `#[cfg(test)] mod tests` inside `src/*.rs` (a couple of older files that still have one are
  known, tracked exceptions, not the pattern to follow).
- **A named exception is a decision, not an oversight.** `context_menu` and `menubar` remain
  independent of the shared `menu`/`positioner` machinery for recorded reasons (documented in
  each file's own module doc and in `openspec/specs/adico-primitives`), re-evaluated rather
  than silently carried forward whenever a capability gap they depend on changes.

## Testing

```sh
cargo test --locked -p adico-primitives                                    # default (SSR-fallback) build
cargo test --locked -p adico-primitives --features web                     # browser-interop paths
cargo test --locked -p adico-primitives --features native                  # WebView-interop paths
cargo check -p adico-primitives --target wasm32-unknown-unknown --features web
```

The default feature set is what CI and local development normally run against: no browser or
WebView is involved, so every test is a deterministic `VirtualDom` render or a real dispatched
event (`dom.runtime().handle_event(..)` + `dom.render_immediate_to_vec()`) checked against
server-rendered HTML. A handful of behaviors genuinely can't be pinned this way — real-timer
delays and true async cancellation chief among them — and are documented as such at the test
site rather than silently skipped; see `tests/test_toast.rs` and `tests/test_preview_card.rs`
for two worked examples of what that documentation looks like.

## Where to go next

- [`../../CLAUDE.md`](../../CLAUDE.md) — full repository conventions and validation commands.
- [`../../docs/architecture.md`](../../docs/architecture.md) — how this crate fits the rest of
  the ecosystem.
- `cargo doc -p adico-primitives --open` — every primitive's full props/ARIA/example
  documentation, generated from the source itself.
- `apps/playground` (`dx serve` from that directory) — a live, interactive demo of every
  installed component built on these primitives.
