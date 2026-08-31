// SPDX-License-Identifier: MIT OR Apache-2.0

//! A unified `Menu` primitive (Base UI's anatomy — see design.md §8a):
//! `Menu`/`MenuTrigger`/`MenuContent`/`MenuItem` for the base case, plus
//! `MenuCheckboxItem`, `MenuRadioGroup`/`MenuRadioItem`, `MenuGroup`/
//! `MenuGroupLabel`, `MenuSeparator`, and `MenuSubmenuRoot`/
//! `MenuSubmenuTrigger` for arbitrarily nested submenus — composed on the
//! roving-focus [`crate::collection`] infrastructure and the [`crate::layer`]
//! dismissable-layer stack already shared by dialog/popover.
//!
//! This is a new, additive primitive: `context_menu`, `dropdown_menu`, and
//! `menubar` each still have their own independent (browser-verified) flat
//! menu implementation. Migrating them onto this module is 7.8 scope, not
//! done here — refactoring already-shipped, Playwright-tested components
//! without a live browser to re-verify against would risk silently
//! regressing tested behavior. `MenuSubmenuTrigger`'s hover-intent-delay
//! (opening a submenu after a brief hover, matching Base UI/Radix) is not
//! implemented — only click and `ArrowRight`/`ArrowLeft` keyboard open/close
//! are, which are the browser-independent-to-reason-about paths; hover
//! timing is left as a named follow-up.

use std::rc::Rc;

use dioxus::prelude::*;

use crate::collection::{CollectionState, collection_item, use_collection_provider, use_item};
use crate::layer::use_layer;
use crate::{use_animated_open, use_controlled, use_id_or, use_unique_id};

#[derive(Clone, Copy)]
struct MenuContext {
    open: Memo<bool>,
    set_open: Callback<bool>,
    disabled: ReadSignal<bool>,
    focus: CollectionState,
    trigger_id: Signal<String>,
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
    let mut ctx = use_context_provider(|| MenuContext {
        open,
        set_open,
        disabled,
        focus,
        trigger_id,
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
    /// Additional attributes for the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The content, typically [`MenuItem`]/[`MenuCheckboxItem`]/[`MenuRadioGroup`]/[`MenuGroup`]/[`MenuSeparator`]/[`MenuSubmenuRoot`].
    pub children: Element,
}

/// # MenuContent
///
/// The popup content of a [`Menu`]. Only rendered while the menu is open.
/// Must be used inside a [`Menu`].
#[component]
pub fn MenuContent(props: MenuContentProps) -> Element {
    let ctx: MenuContext = use_context();
    let unique_id = use_unique_id();
    let id = use_id_or(unique_id, props.id);
    let render = use_animated_open(id, ctx.open);

    rsx! {
        if render() {
            div {
                id,
                role: "menu",
                aria_labelledby: "{ctx.trigger_id}",
                "data-state": if (ctx.open)() { "open" } else { "closed" },
                onpointerdown: move |event| {
                    event.prevent_default();
                    event.stop_propagation();
                },
                ..props.attributes,
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
/// Hover-intent-delay opening (matching Base UI/Radix) is not implemented —
/// see this module's own doc comment.
#[component]
pub fn MenuSubmenuRoot(props: MenuSubmenuRootProps) -> Element {
    let parent_ctx: MenuContext = use_context();
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    let disabled = move || (parent_ctx.disabled)() || (props.disabled)();
    let trigger_id = use_unique_id();
    let focus = use_collection_provider(ReadSignal::new(Signal::new(true)));
    let layer = use_layer();

    use_context_provider(|| MenuContext {
        open,
        set_open,
        disabled: ReadSignal::new(Signal::new(disabled())),
        focus,
        trigger_id,
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
/// submenu open; must be used inside a [`MenuSubmenuRoot`].
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
            ..attributes,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(root: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(root);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[component]
    fn UncheckedCheckboxItem() -> Element {
        rsx! {
            Menu { default_open: true,
                MenuTrigger { "Open" }
                MenuContent {
                    MenuCheckboxItem { index: 0usize, "Wrap words" }
                }
            }
        }
    }

    #[component]
    fn CheckedCheckboxItem() -> Element {
        rsx! {
            Menu { default_open: true,
                MenuTrigger { "Open" }
                MenuContent {
                    MenuCheckboxItem { index: 0usize, default_checked: true, "Wrap words" }
                }
            }
        }
    }

    // `MenuContent` gates on `use_animated_open`, whose real (`web`/`native`)
    // implementation only flips its content-mounted signal from inside a
    // `use_effect` — which a plain `rebuild_in_place()` schedules but does
    // not itself drive to completion outside a running app. Matching
    // `date_picker.rs`'s identical, already-established precedent for this
    // exact class of test, these run only on the SSR-fallback path (no
    // `web`/`native` feature), where `use_animated_open` returns `open`
    // directly with no effect involved.
    #[cfg(not(any(feature = "web", feature = "native")))]
    #[test]
    fn checkbox_item_defaults_to_unchecked() {
        let html = render(UncheckedCheckboxItem);
        assert!(html.contains("data-state=\"unchecked\""));
    }

    #[cfg(not(any(feature = "web", feature = "native")))]
    #[test]
    fn checkbox_item_honors_default_checked() {
        let html = render(CheckedCheckboxItem);
        assert!(html.contains("data-state=\"checked\""));
    }

    #[component]
    fn RadioGroupWithDefaultSelection() -> Element {
        rsx! {
            Menu { default_open: true,
                MenuTrigger { "Open" }
                MenuContent {
                    MenuRadioGroup::<String> { default_value: "b".to_string(),
                        MenuRadioItem::<String> { value: "a", index: 0usize, "A" }
                        MenuRadioItem::<String> { value: "b", index: 1usize, "B" }
                    }
                }
            }
        }
    }

    #[cfg(not(any(feature = "web", feature = "native")))]
    #[test]
    fn radio_group_marks_only_the_default_value_checked() {
        let html = render(RadioGroupWithDefaultSelection);
        assert!(html.contains(r#"data-state="unchecked" data-disabled=false tabindex="-1">A"#));
        assert!(html.contains(r#"data-state="checked" data-disabled=false tabindex="-1">B"#));
    }

    #[component]
    fn ClosedSubmenu() -> Element {
        rsx! {
            Menu { default_open: true,
                MenuTrigger { "Open" }
                MenuContent {
                    MenuSubmenuRoot { index: 0usize,
                        MenuSubmenuTrigger { "More" }
                        MenuContent {
                            MenuItem::<String> { value: "x".to_string(), index: 0usize, "X" }
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(any(feature = "web", feature = "native")))]
    #[test]
    fn submenu_defaults_to_closed_and_its_content_is_not_rendered() {
        let html = render(ClosedSubmenu);
        assert!(html.contains("data-state=\"closed\""));
        assert!(!html.contains("\">X<"));
    }
}
