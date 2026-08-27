// SPDX-License-Identifier: MIT OR Apache-2.0
// Forked from DioxusLabs/dioxus-components at bf007c15d0cf4d04d3181cc46cf12325aa773955.
// Upstream path: primitives/src/combobox/components/combobox.rs. See provenance/records/adico-primitives-wave4-collection.json.

//! Root combobox component.

use dioxus::prelude::*;

use super::super::context::{ComboboxContext, default_combobox_filter};
use crate::{
    Controlled,
    selectable::{
        RcPartialEqValue, SelectionMode, use_selectable_root, use_single_selectable_value,
    },
    use_controlled,
};

/// Props for [`Combobox`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps<T: Clone + PartialEq + 'static = String> {
    /// The controlled value. If supplied, the combobox is controlled
    /// and the signal's `None` value means no option is selected.
    #[props(default)]
    pub value: Option<ReadSignal<Option<T>>>,

    /// The default uncontrolled value.
    #[props(default)]
    pub default_value: Option<T>,

    /// Callback fired when the value changes.
    #[props(default)]
    pub on_value_change: Callback<Option<T>>,

    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the popup.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,

    /// The initial open state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,

    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// The controlled text query used to filter options.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,

    /// The initial text query when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,

    /// Callback fired when the text query changes.
    #[props(default)]
    pub on_query_change: Callback<String>,

    /// Whether arrow-key navigation should wrap.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Custom filter callback. Receives `(query, option_text_value)`.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Children.
    pub children: Element,
}

/// Props for [`ComboboxMulti`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxMultiProps<T: Clone + PartialEq + 'static = String> {
    /// Controlled selected values. `None` leaves selection uncontrolled.
    #[props(default)]
    pub values: ReadSignal<Option<Vec<T>>>,
    /// Initial selected values when uncontrolled.
    #[props(default)]
    pub default_values: Vec<T>,
    /// Callback fired with the complete value set after an option toggles.
    #[props(default)]
    pub on_values_change: Callback<Vec<T>>,
    /// Whether the combobox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Controlled popup state.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// Initial popup state when uncontrolled.
    #[props(default)]
    pub default_open: ReadSignal<bool>,
    /// Callback fired when the popup open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Controlled filter text.
    #[props(default)]
    pub query: ReadSignal<Option<String>>,
    /// Initial filter text when uncontrolled.
    #[props(default)]
    pub default_query: ReadSignal<String>,
    /// Callback fired when filter text changes.
    #[props(default)]
    pub on_query_change: Callback<String>,
    /// Whether keyboard focus loops around visible options.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,
    /// Custom filter callback. Receives `(query, option_text_value)`.
    #[props(default = Callback::new(|(q, t): (String, String)| default_combobox_filter(&q, &t)))]
    pub filter: Callback<(String, String), bool>,
    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// Children.
    pub children: Element,
}

fn use_combobox_root(
    values: Memo<Vec<RcPartialEqValue>>,
    set_value: Callback<RcPartialEqValue>,
    selection_mode: SelectionMode,
    disabled: ReadSignal<bool>,
    roving_loop: ReadSignal<bool>,
    open: Controlled<bool>,
    query: Controlled<String>,
    filter: Callback<(String, String), bool>,
) -> Memo<bool> {
    let selectable = use_selectable_root(
        values,
        set_value,
        selection_mode,
        disabled,
        roving_loop,
        open,
    );
    let (query, set_query) = use_controlled(query.value, query.default.cloned(), query.on_change);
    let open = selectable.open;

    use_context_provider(|| ComboboxContext {
        selectable,
        query,
        set_query,
        filter,
    });

    open
}

/// A single-select autocomplete input with a filterable popup list.
#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(props: ComboboxProps<T>) -> Element {
    let (selected, set_value) = use_single_selectable_value(
        props.value,
        props.default_value,
        props.on_value_change,
        "combobox",
    );

    let open = use_combobox_root(
        selected,
        set_value,
        SelectionMode::Single,
        props.disabled,
        props.roving_loop,
        Controlled {
            value: props.open,
            default: props.default_open,
            on_change: props.on_open_change,
        },
        Controlled {
            value: props.query,
            default: props.default_query,
            on_change: props.on_query_change,
        },
        props.filter,
    );

    rsx! {
        div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            ..props.attributes,
            {props.children}
        }
    }
}

/// A filterable multi-select autocomplete. Selecting an option toggles it and
/// keeps the popup open; the full selected value set is emitted to the caller.
#[component]
pub fn ComboboxMulti<T: Clone + PartialEq + 'static>(props: ComboboxMultiProps<T>) -> Element {
    let (selected_values, set_selected_values) =
        use_controlled(props.values, props.default_values, props.on_values_change);
    let values = use_memo(move || {
        selected_values()
            .into_iter()
            .map(RcPartialEqValue::new)
            .collect()
    });
    let set_value = use_callback(move |incoming: RcPartialEqValue| {
        let value = incoming
            .as_ref::<T>()
            .unwrap_or_else(|| panic!("combobox and option value types must match"))
            .clone();
        let mut current = selected_values();
        if let Some(index) = current.iter().position(|selected| selected == &value) {
            current.remove(index);
        } else {
            current.push(value);
        }
        set_selected_values.call(current);
    });

    let open = use_combobox_root(
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
        Controlled {
            value: props.query,
            default: props.default_query,
            on_change: props.on_query_change,
        },
        props.filter,
    );

    rsx! {
        div {
            "data-state": if open() { "open" } else { "closed" },
            "data-disabled": (props.disabled)(),
            ..props.attributes,
            {props.children}
        }
    }
}
