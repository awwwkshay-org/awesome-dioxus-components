// SPDX-License-Identifier: MIT OR Apache-2.0

//! A unified `Menu` primitive (Base UI's anatomy — see design.md §8a):
//! `Menu`/`MenuTrigger`/`MenuContent`/`MenuItem` for the base case, plus
//! `MenuCheckboxItem`, `MenuRadioGroup`/`MenuRadioItem`, `MenuGroup`/
//! `MenuGroupLabel`, `MenuSeparator`, and `MenuSubmenuRoot`/
//! `MenuSubmenuTrigger` for arbitrarily nested submenus — composed on the
//! roving-focus [`crate::collection`] infrastructure and the [`crate::layer`]
//! dismissable-layer stack already shared by dialog/popover.
//!
//! `dropdown_menu` now delegates to this module directly (task 2.3): its
//! `DropdownMenu`/`DropdownMenuTrigger`/`DropdownMenuContent`/`DropdownMenuItem`
//! are re-exports of `Menu`/`MenuTrigger`/`MenuContent`/`MenuItem`, matching
//! Base UI, which has no separate dropdown-menu component at all — `Menu` *is*
//! the dropdown menu. `context_menu` and `menubar` still have their own
//! independent implementations (click-point/long-press anchoring and
//! multi-menu roving coordination, respectively, neither of which this module
//! has a counterpart for); evaluating how much of their content/item
//! rendering can reuse this module's is separate, remaining task 2.3 scope.
//!
//! `MenuContent` composes [`crate::positioner::Positioner`] (task 7.5b) for
//! anchored placement rather than plain flow layout, matching every other
//! floating-content primitive in this crate. `MenuItem` optionally composes
//! [`crate::typeahead`] (7.3c, via an explicit `text_value` prop, since a
//! generic `MenuItem<T>`'s value isn't necessarily stringifiable on its own)
//! for type-to-select; `MenuCheckboxItem`/`MenuRadioItem` are not wired to
//! typeahead (documented as deferred, not silently dropped — the base
//! `MenuItem` case is overwhelmingly the common one, and wiring the other two
//! is a small, separable follow-up, not additional design work).
//! `MenuSubmenuTrigger` now opens on hover-intent after a configurable delay
//! (`MenuSubmenuRootProps::open_delay_ms`/`close_delay_ms`), using the same
//! generation-counter-debounced-timer technique `preview_card.rs`/
//! `navigation_menu.rs` use for their own hover delays (not shared code
//! between the three files — this crate's own "don't add abstractions beyond
//! what's needed" convention doesn't call three call sites of ~15 lines each
//! a pattern worth extracting yet). Click and `ArrowRight`/`ArrowLeft`
//! keyboard open/close still work exactly as before. Not built: cross-sibling
//! coordination (hovering a *different* top-level submenu trigger closing an
//! already-open sibling) — each `MenuSubmenuRoot` only knows its own local
//! open state, with no shared "currently open submenu" coordinator at the
//! parent `Menu` level; a real, separate architectural addition, not
//! attempted here.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::*;

use crate::collection::{CollectionState, collection_item, use_collection_provider, use_item};
use crate::layer::use_layer;
use crate::positioner::Positioner;
use crate::selection::{OptionState, RcPartialEqValue};
use crate::typeahead::{Typeahead, use_typeahead};
use crate::{
    ContentAlign, ContentSide, use_animated_open, use_controlled, use_effect_cleanup, use_id_or,
    use_unique_id,
};

#[derive(Clone, Copy)]
struct MenuContext {
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    focus: CollectionState,
    trigger_id: Signal<String>,

    /// Buffered, auto-clearing typeahead search over this scope's own
    /// `text_values` (each `Menu`/`MenuSubmenuRoot` scope has its own,
    /// independent typeahead session, matching how each already has its own
    /// independent `focus` collection).
    typeahead: Typeahead,
    /// Index -> display text for every registered [`MenuItem`] in this
    /// scope, kept in sync by each item's own mount/update/unmount.
    text_values: Signal<HashMap<usize, String>>,

    /// `true` for a [`MenuSubmenuRoot`]'s own context, `false` for the root
    /// [`Menu`]'s. Gates hover-intent open/close (`hover_*` below and
    /// [`MenuContent`]'s own mouse-enter/leave wiring) so a root `Menu`'s
    /// content — which only closes on click-away/blur/Escape, matching
    /// Base UI's own top-level trigger having no hover-open of its own —
    /// doesn't also close the moment the pointer leaves it.
    is_submenu: bool,
    hover_open_delay_ms: ReadSignal<u64>,
    hover_close_delay_ms: ReadSignal<u64>,
    hover_generation: Signal<u64>,
}

impl MenuContext {
    /// Feeds one typed character into this scope's typeahead buffer and
    /// moves focus to the best-matching registered item, if any.
    fn handle_typeahead_character(&self, text: &str, code: &str) {
        let mut typeahead = self.typeahead;
        if let Some(ch) = text.chars().next() {
            typeahead.learn_from_keyboard_event(code, ch);
        }

        let options: Vec<OptionState> = self
            .text_values
            .read()
            .iter()
            .map(|(index, text_value)| OptionState {
                id: index.to_string(),
                index: *index,
                value: RcPartialEqValue::new(()),
                text_value: text_value.clone(),
            })
            .collect();

        let focus = self.focus;
        if let Some(best) = typeahead.on_input(text, &options, move |i| focus.is_available(i)) {
            let mut focus = self.focus;
            focus.set_focus(Some(best));
        }
    }

    /// Requests opening (or closing) a submenu after this scope's configured
    /// hover delay; a still-pending request is superseded (not applied) if a
    /// newer request for the same scope arrives before it fires.
    fn request_hover_open(&self, open: bool) {
        let mut generation = self.hover_generation;
        let this_generation = generation() + 1;
        generation.set(this_generation);

        let delay = if open {
            (self.hover_open_delay_ms)()
        } else {
            (self.hover_close_delay_ms)()
        };
        let set_open = self.set_open;
        let hover_generation = self.hover_generation;
        spawn(async move {
            if delay > 0 {
                crate::time::sleep(Duration::from_millis(delay)).await;
            }
            if hover_generation() == this_generation {
                set_open.call(open);
            }
        });
    }
}

/// The props for the [`Menu`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuProps {
    /// Whether the menu is open. Uncontrolled (using `default_open`) if not provided.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: bool,
    /// Called when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Whether the menu is disabled: it will not open and items will not be selectable.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,
    /// How long to wait after the last keystroke before clearing the
    /// typeahead search buffer.
    #[props(default = ReadSignal::new(Signal::new(Duration::from_millis(500))))]
    pub typeahead_timeout: ReadSignal<Duration>,
    /// Additional attributes for the menu root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the menu, which should include a [`MenuTrigger`] and a [`MenuContent`].
    pub children: Element,
}

/// # Menu
///
/// The root of a unified menu: a container for a [`MenuContent`] activated by
/// a [`MenuTrigger`]. Supports [`MenuItem`], [`MenuCheckboxItem`],
/// [`MenuRadioGroup`]/[`MenuRadioItem`], [`MenuGroup`]/[`MenuGroupLabel`],
/// [`MenuSeparator`], and arbitrarily nested [`MenuSubmenuRoot`]s.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::menu::{Menu, MenuContent, MenuItem, MenuTrigger};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Menu { default_open: false,
///             MenuTrigger { "Open" }
///             MenuContent {
///                 MenuItem::<String> {
///                     value: "edit".to_string(),
///                     index: 0usize,
///                     on_select: move |_value| {},
///                     "Edit"
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// Defines `data-state` (`open`/`closed`) and `data-disabled` (`true`/`false`).
#[component]
pub fn Menu(props: MenuProps) -> Element {
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    let disabled = props.disabled;
    let trigger_id = use_unique_id();
    let focus = use_collection_provider(props.roving_loop);
    let typeahead = use_typeahead(props.typeahead_timeout);
    let mut ctx = use_context_provider(|| MenuContext {
        open,
        set_open,
        disabled,
        focus,
        trigger_id,
        typeahead,
        text_values: Signal::new(HashMap::new()),
        is_submenu: false,
        // The root trigger never hover-opens (matching Base UI); these are
        // only read by `MenuSubmenuTrigger`, which always resolves the
        // nearest `MenuSubmenuRoot`'s own context instead.
        hover_open_delay_ms: ReadSignal::new(Signal::new(0)),
        hover_close_delay_ms: ReadSignal::new(Signal::new(0)),
        hover_generation: Signal::new(0),
    });

    use_effect(move || {
        let focused = focus.any_focused();
        if *ctx.open.peek() != focused {
            (ctx.set_open)(focused);
        }
    });

    let handle_keydown = move |event: Event<KeyboardData>| {
        if disabled() {
            return;
        }
        match event.key() {
            Key::Enter => ctx.set_open.call(!(ctx.open)()),
            Key::Escape => ctx.set_open.call(false),
            Key::ArrowDown => ctx.focus.focus_next(),
            Key::ArrowUp => {
                if open() {
                    ctx.focus.focus_prev();
                }
            }
            Key::Home => ctx.focus.focus_first(),
            Key::End => ctx.focus.focus_last(),
            Key::Character(text) if open() && text != " " => {
                let code = event.code().to_string();
                ctx.handle_typeahead_character(&text, &code);
            }
            _ => return,
        }
        event.prevent_default();
    };

    rsx! {
        div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            onkeydown: handle_keydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`MenuTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuTriggerProps {
    /// Additional attributes for the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The trigger's contents.
    pub children: Element,
}

/// # MenuTrigger
///
/// Toggles the parent [`Menu`]'s [`MenuContent`]. Must be used inside a [`Menu`].
#[component]
pub fn MenuTrigger(props: MenuTriggerProps) -> Element {
    let mut ctx: MenuContext = use_context();
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    let open = ctx.open;
    let disabled = ctx.disabled;

    rsx! {
        button {
            id: ctx.trigger_id,
            r#type: "button",
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": disabled,
            disabled,
            aria_expanded: open,
            aria_haspopup: "menu",
            onmounted: move |e: MountedEvent| element.set(Some(e.data())),
            onclick: move |_| {
                if disabled() {
                    return;
                }
                let new_open = !open();
                ctx.set_open.call(new_open);
                if let Some(data) = element() {
                    spawn(async move {
                        _ = data.set_focus(true).await;
                    });
                }
            },
            onblur: move |_| {
                if !ctx.focus.any_focused() {
                    ctx.focus.clear_focus();
                    ctx.set_open.call(false);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`MenuContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuContentProps {
    /// The `id` of the content element. Generated if not provided.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,
    /// Side of the trigger to place the content. Defaults to `Bottom` (a
    /// top-level dropdown opening below its trigger); a
    /// [`MenuSubmenuRoot`]'s own nested `MenuContent` should pass
    /// `ContentSide::Right` (or `Left`, for RTL) instead, matching a flyout
    /// submenu's conventional placement — this component has no way to
    /// distinguish "I'm a submenu's content" from "I'm a root menu's
    /// content" structurally, so the caller composing a submenu is
    /// responsible for passing the right side.
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,
    /// Alignment of the content relative to the trigger.
    #[props(default = ContentAlign::Start)]
    pub align: ContentAlign,
    /// Additional attributes for the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The content, typically [`MenuItem`]/[`MenuCheckboxItem`]/[`MenuRadioGroup`]/[`MenuGroup`]/[`MenuSeparator`]/[`MenuSubmenuRoot`].
    pub children: Element,
}

/// # MenuContent
///
/// The popup content of a [`Menu`], anchored to its [`MenuTrigger`] (or, for
/// a submenu, its [`MenuSubmenuTrigger`]) via [`crate::positioner::Positioner`].
/// Only rendered while the menu is open. Must be used inside a [`Menu`].
#[component]
pub fn MenuContent(props: MenuContentProps) -> Element {
    let ctx: MenuContext = use_context();
    let unique_id = use_unique_id();
    let id = use_id_or(unique_id, props.id);
    let render = use_animated_open(id, ctx.open);

    let mut merged_attributes = vec![
        dioxus_core::Attribute::new("role", "menu", None, false),
        dioxus_core::Attribute::new("aria-labelledby", ctx.trigger_id.cloned(), None, false),
        dioxus_core::Attribute::new(
            "data-state",
            if (ctx.open)() { "open" } else { "closed" },
            None,
            false,
        ),
    ];
    merged_attributes.extend(props.attributes);

    rsx! {
        if render() {
            Positioner {
                id: Some(id()),
                anchor_id: ctx.trigger_id,
                side: props.side,
                align: props.align,
                offset: 4.0,
                on_pointer_down: move |event: Event<PointerData>| {
                    event.prevent_default();
                    event.stop_propagation();
                },
                // Keeps a hover-opened submenu open while the pointer is
                // over its own content, not just its trigger (mirrors
                // `preview_card.rs`/`navigation_menu.rs`'s identical
                // content-level hover handling). Gated on `is_submenu`: the
                // root `Menu`'s own content only closes on click-away/blur/
                // Escape, not on mouse-leave, matching Base UI's top-level
                // trigger having no hover-open of its own.
                on_mouse_enter: move |_| {
                    if ctx.is_submenu {
                        ctx.request_hover_open(true);
                    }
                },
                on_mouse_leave: move |_| {
                    if ctx.is_submenu {
                        ctx.request_hover_open(false);
                    }
                },
                attributes: merged_attributes,

                {props.children}
            }
        }
    }
}

/// The props for the [`MenuItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuItemProps<T: Clone + PartialEq + 'static> {
    /// The value passed to `on_select` when this item is chosen.
    pub value: ReadSignal<T>,
    /// This item's position for keyboard navigation ordering.
    pub index: ReadSignal<usize>,
    /// Whether this item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// This item's display text, registered with the enclosing menu scope's
    /// typeahead search so typing can jump-focus to it. Omit (leave `None`)
    /// to keep this item out of typeahead matching entirely.
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,
    /// Called when this item is selected (click, Enter, or Space).
    #[props(default)]
    pub on_select: Callback<T>,
    /// Additional attributes for the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The item's contents.
    pub children: Element,
}

/// # MenuItem
///
/// A selectable item inside [`MenuContent`]. Selecting it calls `on_select`
/// and closes the menu. Must be used inside a [`Menu`].
#[component]
pub fn MenuItem<T: Clone + PartialEq + 'static>(props: MenuItemProps<T>) -> Element {
    let mut ctx: MenuContext = use_context();
    let disabled = move || (ctx.disabled)() || (props.disabled)();
    let item = use_item(collection_item(ctx.focus, props.index).disabled(disabled));
    let focused = move || item.focused();

    let mut registered_text_value_index: Signal<Option<usize>> = use_signal(|| None);
    use_effect(move || {
        let index = props.index.cloned();
        if let Some(previous) = registered_text_value_index.peek().as_ref()
            && *previous != index
        {
            ctx.text_values.write().remove(previous);
        }
        match (props.text_value)() {
            Some(text) => {
                ctx.text_values.write().insert(index, text);
            }
            None => {
                ctx.text_values.write().remove(&index);
            }
        }
        registered_text_value_index.set(Some(index));
    });
    use_effect_cleanup(move || {
        if let Some(index) = *registered_text_value_index.peek() {
            ctx.text_values.write().remove(&index);
        }
    });

    rsx! {
        div {
            role: "menuitem",
            "data-disabled": disabled(),
            tabindex: if focused() { "0" } else { "-1" },
            onclick: move |e: Event<MouseData>| {
                e.stop_propagation();
                if !disabled() {
                    props.on_select.call((props.value)());
                    ctx.set_open.call(false);
                }
            },
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
                    if !disabled() {
                        props.on_select.call((props.value)());
                        ctx.set_open.call(false);
                    }
                    event.prevent_default();
                    event.stop_propagation();
                }
            },
            onmounted: item.onmounted(),
            onblur: move |_| {
                if focused() {
                    ctx.focus.clear_focus();
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`MenuCheckboxItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuCheckboxItemProps {
    /// This item's position for keyboard navigation ordering.
    pub index: ReadSignal<usize>,
    /// The controlled checked state. Uncontrolled (using `default_checked`) if not provided.
    #[props(default)]
    pub checked: ReadSignal<Option<bool>>,
    /// The initial checked state when uncontrolled.
    #[props(default)]
    pub default_checked: bool,
    /// Called when the checked state changes.
    #[props(default)]
    pub on_checked_change: Callback<bool>,
    /// Whether this item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Additional attributes for the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The item's contents (typically a checked-state indicator plus a label).
    pub children: Element,
}

/// # MenuCheckboxItem
///
/// A menu item with a checked state, toggled on select. Unlike [`MenuItem`],
/// selecting it does **not** close the menu — matching Base UI/Radix, where
/// checking several boxes in one visit to the menu is the common case. Must
/// be used inside a [`Menu`].
///
/// ## Styling
///
/// Defines `data-state` (`checked`/`unchecked`) and `data-disabled`.
#[component]
pub fn MenuCheckboxItem(props: MenuCheckboxItemProps) -> Element {
    let mut ctx: MenuContext = use_context();
    let (checked, set_checked) = use_controlled(
        props.checked,
        props.default_checked,
        props.on_checked_change,
    );
    let disabled = move || (ctx.disabled)() || (props.disabled)();
    let item = use_item(collection_item(ctx.focus, props.index).disabled(disabled));
    let focused = move || item.focused();

    let toggle = move || {
        if !disabled() {
            set_checked.call(!checked());
        }
    };

    rsx! {
        div {
            role: "menuitemcheckbox",
            aria_checked: checked(),
            "data-state": if checked() { "checked" } else { "unchecked" },
            "data-disabled": disabled(),
            tabindex: if focused() { "0" } else { "-1" },
            onclick: move |e: Event<MouseData>| {
                e.stop_propagation();
                toggle();
            },
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
                    toggle();
                    event.prevent_default();
                    event.stop_propagation();
                }
            },
            onmounted: item.onmounted(),
            onblur: move |_| {
                if focused() {
                    ctx.focus.clear_focus();
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

#[derive(Clone, Copy)]
struct MenuRadioGroupContext<T: Clone + PartialEq + 'static> {
    value: Memo<Option<T>>,
    set_value: Callback<T>,
}

/// The props for the [`MenuRadioGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioGroupProps<T: Clone + PartialEq + 'static> {
    /// The controlled selected value (`None` means uncontrolled, using
    /// `default_value` instead — matching
    /// [`crate::selectable::use_single_selectable_value`]'s convention for
    /// an optionally-controlled optional value, since `Option<T>` alone
    /// cannot distinguish "controlled, nothing selected" from
    /// "uncontrolled").
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,
    /// The initial selected value when uncontrolled.
    #[props(default)]
    pub default_value: Option<T>,
    /// Called with the newly selected value when a [`MenuRadioItem`] is chosen.
    #[props(default)]
    pub on_value_change: Callback<T>,
    /// Additional attributes for the group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The group's [`MenuRadioItem`]s.
    pub children: Element,
}

/// # MenuRadioGroup
///
/// Groups [`MenuRadioItem`]s into a single-select set within a [`Menu`].
/// Must be used inside a [`Menu`].
#[component]
pub fn MenuRadioGroup<T: Clone + PartialEq + 'static>(props: MenuRadioGroupProps<T>) -> Element {
    let mut internal_value: Signal<Option<T>> = use_signal(|| props.default_value.clone());
    let value = use_memo(move || match props.value {
        Some(controlled) => controlled.cloned(),
        None => internal_value.cloned(),
    });
    let set_value = use_callback(move |v: T| {
        internal_value.set(Some(v.clone()));
        props.on_value_change.call(v);
    });
    use_context_provider(|| MenuRadioGroupContext { value, set_value });

    rsx! {
        div {
            role: "group",
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`MenuRadioItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuRadioItemProps<T: Clone + PartialEq + 'static> {
    /// This item's value within the enclosing [`MenuRadioGroup`].
    pub value: ReadSignal<T>,
    /// This item's position for keyboard navigation ordering.
    pub index: ReadSignal<usize>,
    /// Whether this item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Additional attributes for the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The item's contents.
    pub children: Element,
}

/// # MenuRadioItem
///
/// A single-select item within a [`MenuRadioGroup`]. Selecting it sets the
/// group's value and closes the menu (unlike [`MenuCheckboxItem`], a radio
/// selection is a single terminal choice). Must be used inside a
/// [`MenuRadioGroup`].
///
/// ## Styling
///
/// Defines `data-state` (`checked`/`unchecked`) and `data-disabled`.
#[component]
pub fn MenuRadioItem<T: Clone + PartialEq + 'static>(props: MenuRadioItemProps<T>) -> Element {
    let mut menu_ctx: MenuContext = use_context();
    let group_ctx: MenuRadioGroupContext<T> = use_context();
    let disabled = move || (menu_ctx.disabled)() || (props.disabled)();
    let item = use_item(collection_item(menu_ctx.focus, props.index).disabled(disabled));
    let focused = move || item.focused();
    let checked = move || (group_ctx.value)().as_ref() == Some(&(props.value)());

    let select = move || {
        if !disabled() {
            group_ctx.set_value.call((props.value)());
            menu_ctx.set_open.call(false);
        }
    };

    rsx! {
        div {
            role: "menuitemradio",
            aria_checked: checked(),
            "data-state": if checked() { "checked" } else { "unchecked" },
            "data-disabled": disabled(),
            tabindex: if focused() { "0" } else { "-1" },
            onclick: move |e: Event<MouseData>| {
                e.stop_propagation();
                select();
            },
            onkeydown: move |event: Event<KeyboardData>| {
                if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
                    select();
                    event.prevent_default();
                    event.stop_propagation();
                }
            },
            onmounted: item.onmounted(),
            onblur: move |_| {
                if focused() {
                    menu_ctx.focus.clear_focus();
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`MenuGroup`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuGroupProps {
    /// Additional attributes for the group element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The group's contents, typically a [`MenuGroupLabel`] followed by [`MenuItem`]s.
    pub children: Element,
}

/// # MenuGroup
///
/// A purely visual/ARIA grouping of related menu items; does not affect
/// keyboard navigation ordering.
#[component]
pub fn MenuGroup(props: MenuGroupProps) -> Element {
    rsx! {
        div {
            role: "group",
            ..props.attributes,
            {props.children}
        }
    }
}

/// # MenuGroupLabel
///
/// A non-interactive label for a [`MenuGroup`].
#[component]
pub fn MenuGroupLabel(
    /// Additional attributes for the label element.
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    /// The label's contents.
    children: Element,
) -> Element {
    rsx! {
        div {
            role: "presentation",
            ..attributes,
            {children}
        }
    }
}

/// # MenuSeparator
///
/// A visual divider between menu items or groups.
#[component]
pub fn MenuSeparator(
    /// Additional attributes for the separator element.
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        div {
            role: "separator",
            "aria-orientation": "horizontal",
            ..attributes,
        }
    }
}

/// The props for the [`MenuSubmenuRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct MenuSubmenuRootProps {
    /// This submenu trigger's position within the parent menu's keyboard
    /// navigation ordering.
    pub index: ReadSignal<usize>,
    /// Whether this submenu is open. Uncontrolled (using `default_open`) if not provided.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: bool,
    /// Called when the submenu's open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Whether this submenu is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Milliseconds to wait after the pointer enters the
    /// [`MenuSubmenuTrigger`] before opening this submenu.
    #[props(default = ReadSignal::new(Signal::new(200)))]
    pub open_delay_ms: ReadSignal<u64>,
    /// Milliseconds to wait after the pointer leaves the trigger (or the
    /// submenu's own content) before closing it.
    #[props(default = ReadSignal::new(Signal::new(200)))]
    pub close_delay_ms: ReadSignal<u64>,
    /// How long to wait after the last keystroke before clearing this
    /// submenu's own typeahead search buffer.
    #[props(default = ReadSignal::new(Signal::new(Duration::from_millis(500))))]
    pub typeahead_timeout: ReadSignal<Duration>,
    /// Additional attributes for the submenu trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Must contain a [`MenuSubmenuTrigger`] and a [`MenuContent`] (this
    /// submenu's own, nested content — [`MenuItem`]s inside it, including
    /// further [`MenuSubmenuRoot`]s, compose a new, independent [`Menu`]
    /// scope).
    pub children: Element,
}

/// # MenuSubmenuRoot
///
/// A submenu nested inside a [`MenuContent`]. Acts as a roving-focus item in
/// the *parent* menu's keyboard navigation (via `index`) while also being its
/// own independent [`Menu`] scope for its own [`MenuContent`]/[`MenuItem`]s —
/// nesting composes to any depth by nesting another `MenuSubmenuRoot` inside
/// a submenu's own content. Opens on click or `ArrowRight` (when focused in
/// the parent, matching the parent's writing direction is the caller's
/// responsibility via `crate::direction`); closes on `ArrowLeft` or `Escape`
/// (gated by the shared [`crate::layer`] stack, so a doubly-nested submenu's
/// Escape closes only the innermost one). Must be used inside a [`Menu`] or
/// another `MenuSubmenuRoot`'s [`MenuContent`].
///
/// Also opens on hover-intent, after [`MenuSubmenuRootProps::open_delay_ms`]
/// (and closes after [`MenuSubmenuRootProps::close_delay_ms`]) — see this
/// module's own doc comment for the technique and its one named limitation
/// (no cross-sibling coordination between different open submenus yet).
#[component]
pub fn MenuSubmenuRoot(props: MenuSubmenuRootProps) -> Element {
    let parent_ctx: MenuContext = use_context();
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    let disabled = move || (parent_ctx.disabled)() || (props.disabled)();
    let trigger_id = use_unique_id();
    let focus = use_collection_provider(ReadSignal::new(Signal::new(true)));
    let layer = use_layer(open);
    let typeahead = use_typeahead(props.typeahead_timeout);

    let ctx = use_context_provider(|| MenuContext {
        open,
        set_open,
        disabled: ReadSignal::new(Signal::new(disabled())),
        focus,
        trigger_id,
        typeahead,
        text_values: Signal::new(HashMap::new()),
        is_submenu: true,
        hover_open_delay_ms: props.open_delay_ms,
        hover_close_delay_ms: props.close_delay_ms,
        hover_generation: Signal::new(0),
    });

    let item = use_item(collection_item(parent_ctx.focus, props.index).disabled(disabled));
    let focused = move || item.focused();

    let handle_keydown = move |event: Event<KeyboardData>| {
        if disabled() {
            return;
        }
        match event.key() {
            Key::ArrowRight => {
                set_open.call(true);
                event.stop_propagation();
            }
            Key::ArrowLeft => {
                if open() {
                    set_open.call(false);
                    event.stop_propagation();
                }
            }
            Key::Escape if open() && layer.is_topmost() => {
                set_open.call(false);
                event.stop_propagation();
            }
            Key::Character(text) if open() && text != " " => {
                let code = event.code().to_string();
                ctx.handle_typeahead_character(&text, &code);
                event.stop_propagation();
            }
            _ => {}
        }
    };

    rsx! {
        div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": disabled(),
            onkeydown: handle_keydown,
            onmounted: item.onmounted(),
            tabindex: if focused() { "0" } else { "-1" },
            ..props.attributes,
            {props.children}
        }
    }
}

/// # MenuSubmenuTrigger
///
/// The trigger for a [`MenuSubmenuRoot`]'s content. Click toggles the
/// submenu open immediately; hovering opens it after
/// [`MenuSubmenuRootProps::open_delay_ms`] (closes after
/// `close_delay_ms`), matching Base UI/Radix hover-intent. Must be used
/// inside a [`MenuSubmenuRoot`].
#[component]
pub fn MenuSubmenuTrigger(
    /// Additional attributes for the trigger element.
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
    /// The trigger's contents.
    children: Element,
) -> Element {
    let ctx: MenuContext = use_context();
    let open = ctx.open;
    let disabled = ctx.disabled;

    rsx! {
        div {
            id: ctx.trigger_id,
            role: "menuitem",
            aria_haspopup: "menu",
            aria_expanded: open,
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": disabled,
            onclick: move |e: Event<MouseData>| {
                e.stop_propagation();
                if !disabled() {
                    (ctx.set_open)(!open());
                }
            },
            onmouseenter: move |_| {
                if !disabled() {
                    ctx.request_hover_open(true);
                }
            },
            onmouseleave: move |_| {
                if !disabled() {
                    ctx.request_hover_open(false);
                }
            },
            ..attributes,
            {children}
        }
    }
}
