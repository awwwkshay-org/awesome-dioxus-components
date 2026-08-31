// Implements the WAI-ARIA APG's Listbox pattern (single- and multi-select
// variants) as a searchable, keyboard-navigable dropdown: `Select`/`SelectMulti`
// own the root open/value state, `SelectTrigger` is the APG's "combobox-like
// button", and `SelectList` is the `role="listbox"` popup, anchored to the
// trigger via `positioner::Positioner` and dismissed via the shared
// `layer`/`use_escape_key`/`use_outside_dismiss` stack the same way
// `popover.rs` composes them — this file previously rendered its listbox
// inline in normal document flow with no positioning or layer participation
// at all. Selection, roving focus, and typeahead are `selectable.rs`/
// `collection.rs`/`typeahead.rs`'s existing, independently-tested behavior;
// this file is the ARIA-role glue and event wiring around them, following
// this crate's "one-stop shop" synthesis of Base UI's Select anatomy and
// dioxus-primitives' original listbox implementation rather than mirroring
// either one-to-one.

//! Defines the [`Select`] component and its sub-components, which provide a searchable select input with keyboard navigation.
//!
//! The Select component consists of several parts that work together:
//! - [`Select`] - The root container component (see [`SelectMulti`] for multi-select)
//! - [`SelectTrigger`] - The button that opens/closes the dropdown
//! - [`SelectList`] - The dropdown container for options, anchored to the trigger
//! - [`SelectOption`] - Individual selectable options
//! - [`SelectItemIndicator`] - Visual indicator for selected items
//! - [`SelectGroup`] - Groups related options together
//! - [`SelectGroupLabel`] - Labels for option groups
//! - [`SelectValue`] - Displays the currently selected value
//!
//! ## Features
//!
//! - **Keyboard Navigation**: Full keyboard support with arrow keys, home/end, enter, and escape
//! - **Typeahead Search**: Smart text search that adapts to different keyboard layouts
//! - **Accessibility**: ARIA compliant with proper roles and attributes
//! - **Customizable**: Flexible styling through data attributes and CSS
//! - **Focus Management**: Automatic focus handling and restoration
//! - **Anchored Positioning**: The list is anchored to the trigger via [`crate::positioner::Positioner`]
//!   and participates in the shared [`crate::layer`] dismiss stack, so Escape and outside-click
//!   close it the same way as every other overlay in this crate.
//!
//! ## Typeahead Buffer Behavior
//!
//! The Select component implements an typeahead search buffer that lets you type while the dropdown is open to focus a matching
//! option. The buffer will be cleared after some amount of time has passed with no new input. The timeout is 1 second by default,
//! but can be configured by setting the [`SelectProps::typeahead_timeout`].
//!
//! ## Example
//!
//! ```rust
//! use dioxus::prelude::*;
//! use adico_primitives::select::{
//!     Select, SelectGroup, SelectGroupLabel, SelectItemIndicator,
//!     SelectList, SelectOption, SelectTrigger, SelectValue,
//! };
//!
//! #[component]
//! fn Demo() -> Element {
//!     rsx! {
//!         Select::<String> {
//!             SelectTrigger{
//!                 aria_label: "Select Trigger",
//!                 width: "12rem",
//!                 SelectValue { placeholder: "Select a fruit..." }
//!             }
//!             SelectList {
//!                 aria_label: "Select Demo",
//!                 SelectGroup {
//!                     SelectGroupLabel { "Fruits" }
//!                     SelectOption::<String> {
//!                         index: 0usize,
//!                         value: "apple",
//!                         "Apple"
//!                         SelectItemIndicator { "✔️" }
//!                     }
//!                     SelectOption::<String> {
//!                         index: 1usize,
//!                         value: "banana",
//!                         "Banana"
//!                         SelectItemIndicator { "✔️" }
//!                     }
//!                 }
//!             }
//!         }
//!     }
//! }
//! ```

use core::panic;
use std::time::Duration;

use dioxus::prelude::*;

use crate::{
    ContentAlign, ContentSide, Controlled,
    collection::{collection_item, use_item},
    listbox::{ListboxContext, ListboxItemIndicator, use_listbox_container},
    positioner::Positioner,
    selectable::{
        RcPartialEqValue, SelectableContext, SelectableOptionConfig, SelectionMode,
        pointer_select_cancel, pointer_select_commit, pointer_select_start, use_selectable_option,
        use_selectable_root, use_single_selectable_value,
    },
    typeahead::{Typeahead, use_typeahead},
    use_controlled, use_effect, use_escape_key, use_id_or, use_outside_dismiss, use_unique_id,
};

/// Shared state for every part of a [`Select`]/[`SelectMulti`] tree.
///
/// `pub` only for `packages/adico-primitives/tests/`, per this crate's test-placement
/// convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`);
/// not part of the intended public API.
#[derive(Clone, Copy)]
pub struct SelectContext {
    /// Shared selectable listbox state.
    pub selectable: SelectableContext,
    /// Buffered, auto-clearing typeahead search.
    pub typeahead: Typeahead,
    /// The root element's stable id, bounding [`use_outside_dismiss`]'s
    /// pointer-outside check so clicking the trigger doesn't count as
    /// "outside" the select.
    pub root_id: Signal<String>,
    /// The trigger's stable id, used as [`SelectList`]'s [`Positioner`] anchor.
    pub trigger_id: Signal<String>,
}

impl SelectContext {
    pub fn set_open(&mut self, open: bool) {
        self.selectable.set_open(open);
    }

    pub fn multi(&self) -> bool {
        self.selectable.selection_mode.is_multiple()
    }

    /// Select the currently focused item
    pub fn select_current_item(&mut self) {
        self.selectable.select_focused();
    }

    /// Learn from a keyboard event mapping physical key to logical character
    pub fn learn_from_keyboard_event(&mut self, physical_code: &str, logical_char: char) {
        self.typeahead
            .learn_from_keyboard_event(physical_code, logical_char);
    }

    /// Add text to the typeahead buffer for searching and focus the best match
    pub fn add_to_typeahead_buffer(&mut self, text: &str) {
        let options = self.selectable.options.read();
        let collection = self.selectable.collection;

        if let Some(best_match_index) = self
            .typeahead
            .on_input(text, &options, move |index| collection.is_available(index))
        {
            self.selectable.collection.set_focus(Some(best_match_index));
        }
    }
}

/// Context for select group components. `pub` only for
/// `packages/adico-primitives/tests/`; not part of the intended API.
#[derive(Clone, Copy)]
pub struct SelectGroupContext {
    /// ID of the element that labels this group
    pub labeled_by: Signal<Option<String>>,
}

/// Props for the [`Select`] (single-select) component
#[derive(Props, Clone, PartialEq)]
pub struct SelectProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled value of the select. If supplied, the select is controlled
    /// and the signal's `None` value means no option is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// The initial value of the select when uncontrolled. `None` means no initial
    /// selection — the placeholder is shown until the user picks an option.
    #[props(default)]
    pub default_value: Option<T>,

    /// Callback fired when the selected value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Whether the select is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the select popup.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Name of the select for form submission
    #[props(default)]
    pub name: ReadSignal<String>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Timeout in milliseconds before clearing typeahead buffer
    #[props(default = ReadSignal::new(Signal::new(Duration::from_millis(1000))))]
    pub typeahead_timeout: ReadSignal<Duration>,

    /// Additional attributes for the select element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the Select component
    pub children: Element,
}

/// Props for the [`SelectMulti`] (multi-select) component
#[derive(Props, Clone, PartialEq)]
pub struct SelectMultiProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled list of selected values.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<T>>>,

    /// The default list of selected values.
    #[props(default)]
    pub default_values: Vec<T>,

    /// Callback when the list of selected values changes.
    #[props(default)]
    pub on_values_change: Callback<Vec<T>>,

    /// Whether the select is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the select popup.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Name of the select for form submission
    #[props(default)]
    pub name: ReadSignal<String>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Timeout in milliseconds before clearing typeahead buffer
    #[props(default = ReadSignal::new(Signal::new(Duration::from_millis(1000))))]
    pub typeahead_timeout: ReadSignal<Duration>,

    /// Additional attributes for the select element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the SelectMulti component
    pub children: Element,
}

/// Sets up the shared signals, focus, positioning ids, dismiss wiring, and
/// context that both [`Select`] and [`SelectMulti`] need. Returns the `open`
/// signal and the root's own `onkeydown` (Escape-to-close, gated by the
/// shared `layer` stack) for the root `<div>`.
fn use_select_root(
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    selection_mode: SelectionMode,
    disabled: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    open: Controlled<bool>,
    typeahead_timeout: ReadSignal<Duration>,
) -> (Memo<bool>, impl FnMut(Event<KeyboardData>) + Clone) {
    let selectable = use_selectable_root(
        values,
        set_value,
        selection_mode,
        disabled,
        roving_loop,
        open,
    );
    let mut typeahead = use_typeahead(typeahead_timeout);
    let open = selectable.open;

    // Clear the typeahead buffer when the select is closed
    use_effect(move || {
        if !open() {
            typeahead.clear();
        }
    });

    let root_id = use_unique_id();
    let trigger_id = use_unique_id();

    let mut ctx = SelectContext {
        selectable,
        typeahead,
        root_id,
        trigger_id,
    };
    use_context_provider(|| ctx);

    // Escape closes the select regardless of which descendant has focus; see
    // `use_escape_key`'s doc comment for why this lives on the root rather
    // than a document-level listener. `SelectList`'s own outside-dismiss
    // (below) joins this same layer via `use_layer_member`, so both agree on
    // which of them is topmost.
    let onkeydown = use_escape_key(move || ctx.set_open(false));

    (open, onkeydown)
}

/// # Select
///
/// The `Select` component is a searchable single-select dropdown that allows users to choose
/// one option from a list with keyboard navigation and typeahead search functionality. For
/// selecting multiple values, see [`SelectMulti`]. See this module's doc comment for a full
/// worked example.
///
/// ## Styling
///
/// The [`Select`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the select. Values are `open` or `closed`.
#[component]
pub fn Select<T: Clone + PartialEq + 'static>(props: SelectProps<T>) -> Element {
    let (values, set_value) = use_single_selectable_value(
        props.value,
        props.default_value,
        props.on_value_change,
        "select",
    );

    let (open, onkeydown) = use_select_root(
        values,
        set_value,
        SelectionMode::Single,
        props.disabled,
        props.roving_loop,
        Controlled {
            value: props.open,
            default: props.default_open,
            on_change: props.on_open_change,
        },
        props.typeahead_timeout,
    );
    let root_id = use_context::<SelectContext>().root_id;

    rsx! {
        div {
            id: root_id,
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            onkeydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// # SelectMulti
///
/// The `SelectMulti` component is a searchable multi-select dropdown. Selecting an option
/// toggles it in or out of the selection and the dropdown stays open across selections; it
/// closes via Escape, the trigger, or tabbing out of the listbox. For single-selection use
/// [`Select`] instead.
///
/// ## Styling
///
/// The [`SelectMulti`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the select. Values are `open` or `closed`.
#[component]
pub fn SelectMulti<T: Clone + PartialEq + 'static>(props: SelectMultiProps<T>) -> Element {
    let (multi_values, set_multi_internal) =
        use_controlled(props.values, props.default_values, props.on_values_change);

    let values = use_memo(move || {
        multi_values()
            .into_iter()
            .map(RcPartialEqValue::new)
            .collect()
    });
    let set_value = use_callback(move |value: RcPartialEqValue| {
        let value_t = value
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("The values of select and all options must match types"))
            .clone();
        let mut current = multi_values();
        if let Some(pos) = current.iter().position(|v| v == &value_t) {
            current.remove(pos);
        } else {
            current.push(value_t);
        }
        set_multi_internal.call(current);
    });

    let (open, onkeydown) = use_select_root(
        values,
        set_value,
        SelectionMode::Multiple,
        props.disabled,
        props.roving_loop,
        Controlled {
            value: props.open,
            default: props.default_open,
            on_change: props.on_open_change,
        },
        props.typeahead_timeout,
    );
    let root_id = use_context::<SelectContext>().root_id;

    rsx! {
        div {
            id: root_id,
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            onkeydown,
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`SelectTrigger`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectTriggerProps {
    /// Additional attributes for the trigger button
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the trigger
    pub children: Element,
}

/// # SelectTrigger
///
/// The trigger button for the [`Select`] component which controls if the [`SelectList`] is
/// rendered, and which [`SelectList`] anchors its position to.
///
/// This must be used inside a [`Select`] component.
///
/// ## Styling
///
/// The [`SelectTrigger`] component defines a span with a `data-placeholder` attribute if a placeholder is set.
#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let open = ctx.selectable.open;

    rsx! {
        button {
            id: ctx.trigger_id,
            // Standard HTML attributes
            disabled: (ctx.selectable.disabled)(),
            type: "button",

            onclick: move |_| {
                ctx.set_open(!open());
            },
            onkeydown: move |event| {
                match event.key() {
                    Key::ArrowUp => {
                        ctx.set_open(true);
                        ctx.selectable
                            .initial_focus
                            .set(ctx.selectable.collection.last_available_index());
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    Key::ArrowDown => {
                        ctx.set_open(true);
                        ctx.selectable
                            .initial_focus
                            .set(ctx.selectable.collection.first_available_index());
                        event.prevent_default();
                        event.stop_propagation();
                    }
                    _ => {}
                }
            },

            // ARIA attributes
            aria_haspopup: "listbox",
            aria_expanded: open(),
            aria_controls: ctx.selectable.list_id,

            // Pass through other attributes
            ..props.attributes,

            // Render children (options)
            {props.children}
        }
    }
}

/// The props for the [`SelectList`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectListProps {
    /// The ID of the list for ARIA attributes
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the list
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the list
    pub children: Element,
}

/// # SelectList
///
/// The dropdown list container for the [`Select`] component that contains the [`SelectOption`]s.
/// The list will only be rendered when the select is open, and is anchored to the
/// [`SelectTrigger`] via [`crate::positioner::Positioner`].
///
/// This must be used inside a [`Select`] component.
#[component]
pub fn SelectList(props: SelectListProps) -> Element {
    let mut ctx = use_context::<SelectContext>();

    let open = ctx.selectable.open;
    let mut listbox_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    let focused = move || open() && !ctx.selectable.collection.any_focused();

    use_effect(move || {
        let Some(listbox_ref) = listbox_ref() else {
            return;
        };
        if focused() {
            spawn(async move {
                _ = listbox_ref.set_focus(true);
            });
        }
    });

    let onkeydown = move |event: KeyboardEvent| {
        let key = event.key();
        let code = event.code();

        // Learn from keyboard events for adaptive matching
        if let Key::Character(actual_char) = &key {
            if let Some(actual_char) = actual_char.chars().next() {
                ctx.learn_from_keyboard_event(&code.to_string(), actual_char);
            }
        }

        let mut arrow_key_navigation = |event: KeyboardEvent| {
            // Clear the typeahead buffer
            ctx.typeahead.clear();
            event.prevent_default();
            event.stop_propagation();
        };

        match key {
            Key::Character(new_text) => {
                if new_text == " " {
                    ctx.select_current_item();
                    event.prevent_default();
                    event.stop_propagation();
                    return;
                }

                ctx.add_to_typeahead_buffer(&new_text);
            }
            Key::ArrowUp => {
                arrow_key_navigation(event);
                ctx.selectable.collection.focus_prev();
            }
            Key::End => {
                arrow_key_navigation(event);
                ctx.selectable.collection.focus_last();
            }
            Key::ArrowDown => {
                arrow_key_navigation(event);
                ctx.selectable.collection.focus_next();
            }
            Key::Home => {
                arrow_key_navigation(event);
                ctx.selectable.collection.focus_first();
            }
            Key::Enter => {
                ctx.select_current_item();
                event.prevent_default();
                event.stop_propagation();
            }
            // Escape is handled by the root's own `onkeydown` (via
            // `use_escape_key`), not here — falling through to `_` lets it
            // bubble up uninterrupted.
            _ => {}
        }
    };

    let listbox = use_listbox_container(props.id, ctx.selectable);
    let render = listbox.render;

    // Outside-pointer dismiss joins the root's layer via `use_layer_member`
    // (see its doc comment) so this later-mounted scope never shadows the
    // root's own `is_topmost` check for Escape.
    use_outside_dismiss(ctx.root_id, move || ctx.set_open(false));

    rsx! {
        if render() {
            Positioner {
                id: (listbox.id)(),
                anchor_id: ctx.trigger_id,
                side: ContentSide::Bottom,
                align: ContentAlign::Center,
                offset: 4.0,
                role: "listbox",
                tabindex: if focused() { "0" } else { "-1" },
                aria_multiselectable: ctx.multi(),

                // Data attributes
                "data-state": if open() { "open" } else { "closed" },

                on_mounted: move |evt: Event<MountedData>| listbox_ref.set(Some(evt.data())),
                on_keydown: onkeydown,
                on_blur: move |_| {
                    if focused() {
                        ctx.set_open(false);
                    }
                },

                attributes: props.attributes,
                {props.children}
            }
        } else {
            // If not rendering, return children directly so we can populate the selected list, but they should choose to not render themselves
            {props.children}
        }
    }
}

/// The props for the [`SelectGroup`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectGroupProps {
    /// Whether the group is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional ID for the group
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the group
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the group
    pub children: Element,
}

/// # SelectGroup
///
/// Groups related options within a [`SelectList`]. This must be used inside a [`SelectList`].
#[component]
pub fn SelectGroup(props: SelectGroupProps) -> Element {
    let ctx = use_context::<SelectContext>();
    let disabled = ctx.selectable.disabled.cloned() || props.disabled.cloned();

    let labeled_by = use_signal(|| None);

    use_context_provider(|| SelectGroupContext { labeled_by });
    let render = use_context::<ListboxContext>().render;

    rsx! {
        if render() {
            div {
                role: "group",

                // ARIA attributes
                aria_disabled: disabled,
                aria_labelledby: labeled_by,

                ..props.attributes,
                {props.children}
            }
        } else {
            // If we are not rendering, still render the children components
            {props.children}
        }
    }
}

/// The props for the [`SelectGroupLabel`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectGroupLabelProps {
    /// Optional ID for the label
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the label
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the label
    pub children: Element,
}

/// # SelectGroupLabel
///
/// A label for a group of options within a [`SelectList`]. This must be used inside a
/// [`SelectGroup`].
#[component]
pub fn SelectGroupLabel(props: SelectGroupLabelProps) -> Element {
    let mut ctx: SelectGroupContext = use_context();

    let id = use_unique_id();
    let id = use_id_or(id, props.id);

    use_effect(move || {
        ctx.labeled_by.set(Some(id()));
    });

    let render = use_context::<ListboxContext>().render;

    rsx! {
        if render () {
            div {
                // Set the ID for the label
                id,
                ..props.attributes,
                {props.children}
            }
        }
    }
}

/// The props for the [`SelectOption`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectOptionProps<T: Clone + PartialEq + 'static> {
    /// The value of the option
    pub value: ReadSignal<T>,

    /// The text value of the option used for typeahead search
    #[props(default)]
    pub text_value: ReadSignal<Option<String>>,

    /// Whether the option is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional ID for the option
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// The index of the option in the list. This is used to define the focus order for keyboard navigation.
    pub index: ReadSignal<usize>,

    /// Optional label for the option (for accessibility)
    #[props(default)]
    pub aria_label: Option<String>,

    /// Optional description role for the option (for accessibility)
    #[props(default)]
    pub aria_roledescription: Option<String>,

    /// Additional attributes for the option element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the option
    pub children: Element,
}

/// # SelectOption
///
/// An individual selectable option within a [`SelectList`]. Each option represents a value
/// that can be selected.
///
/// ## Value vs Text Value
///
/// - **`value`**: The programmatic value (e.g., `"apple"`, `"user_123"`) used internally
/// - **`text_value`**: The text value (e.g., `"Apple"`, `"John Doe"`) used for typeahead search and displayed in the [`SelectValue`]
///
/// This must be used inside a [`SelectList`] component.
#[component]
pub fn SelectOption<T: PartialEq + Clone + 'static>(props: SelectOptionProps<T>) -> Element {
    let index = props.index;

    let mut ctx: SelectContext = use_context();
    let option = use_selectable_option(
        ctx.selectable,
        SelectableOptionConfig {
            id: props.id,
            index,
            value: props.value,
            text_value: props.text_value,
            option_disabled: props.disabled,
            component_name: "SelectOption",
        },
    );

    let item = use_item(
        collection_item(ctx.selectable.collection, props.index)
            .key(move || Some(option.id.cloned()))
            .disabled(move || option.disabled.cloned())
            .selected(move || (option.selected)()),
    );
    let onmounted = item.onmounted();

    let render = use_context::<ListboxContext>().render;

    rsx! {
        if render() {
            div {
                role: "option",
                id: option.id,
                tabindex: if (option.focused)() { "0" } else { "-1" },
                onmounted,

                aria_selected: (option.selected)(),
                aria_disabled: (option.disabled)(),
                aria_label: props.aria_label.clone(),
                aria_roledescription: props.aria_roledescription.clone(),
                "data-disabled": (option.disabled)(),

                onpointerdown: move |event| {
                    pointer_select_start(&event, (option.disabled)(), option.down_pos);
                },
                onpointerup: move |event| {
                    if pointer_select_commit(&event, (option.disabled)(), option.down_pos) {
                        ctx.selectable.select_value(RcPartialEqValue::new(option.value.cloned()));
                    }
                },
                onpointercancel: move |_| {
                    pointer_select_cancel(option.down_pos);
                },
                onblur: move |_| {
                    if (option.focused)() {
                        ctx.selectable.collection.clear_focus();
                        ctx.set_open(false);
                    }
                },

                ..props.attributes,
                {props.children}
            }
        }
    }
}

/// The props for the [`SelectItemIndicator`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectItemIndicatorProps {
    /// The children to render inside the indicator
    pub children: Element,
}

/// # SelectItemIndicator
///
/// Renders its children only when the enclosing [`SelectOption`] is selected. This must be
/// used inside a [`SelectOption`] component.
#[component]
pub fn SelectItemIndicator(props: SelectItemIndicatorProps) -> Element {
    rsx! {
        ListboxItemIndicator {
            {props.children}
        }
    }
}

/// The props for the [`SelectValue`] component
#[derive(Props, Clone, PartialEq)]
pub struct SelectValueProps {
    /// Optional placeholder text shown when no option is selected.
    #[props(default = ReadSignal::new(Signal::new(String::from("Select an option"))))]
    pub placeholder: ReadSignal<String>,

    /// Additional attributes for the value element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # SelectValue
///
/// Displays the currently selected value(s), or a placeholder when nothing is selected. This
/// must be used inside a [`Select`] or [`SelectMulti`] component (typically inside the
/// [`SelectTrigger`]).
///
/// ## Styling
///
/// The [`SelectValue`] component defines a span with a `data-placeholder` attribute if a placeholder is set.
#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
    let ctx = use_context::<SelectContext>();

    let is_empty = move || ctx.selectable.is_empty();
    let selected_values = ctx.selectable.selected_texts();
    let display_value = if !selected_values.is_empty() {
        selected_values.join(", ")
    } else {
        props.placeholder.cloned()
    };

    rsx! {
        // Add placeholder option if needed
        span {
            "data-placeholder": is_empty(),
            ..props.attributes,
            {display_value}
        }
    }
}

/// Test-only adapters used by the workspace's shared interaction helpers.
#[cfg(feature = "test-utils")]
#[doc(hidden)]
pub mod test_support {
    use crate::selection::{OptionState, RcPartialEqValue};
    use crate::typeahead::{AdaptiveKeyboard, best_match};

    /// Returns the selected option index for a typeahead sequence.
    pub fn typeahead_best_match(query: &str, options: &[&str]) -> Option<usize> {
        let options: Vec<_> = options
            .iter()
            .enumerate()
            .map(|(index, text_value)| OptionState {
                id: format!("test-option-{index}"),
                index,
                value: RcPartialEqValue::new(index),
                text_value: (*text_value).to_string(),
            })
            .collect();
        best_match(&AdaptiveKeyboard::new(), query, &options, |_| true)
    }
}
