//! Styled Combobox parts backed by the owned `adico-primitives` behavior.
//!
//! The primitive retains filtering, typeahead, keyboard navigation, focus, and
//! ARIA. These façades provide a positioned shadcn-style popup surface.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use adico_primitives::icons::{ChevronDown, ChevronUp};

use adico_primitives::combobox::{
    Combobox as PrimitiveCombobox, ComboboxInput as PrimitiveComboboxInput,
    ComboboxList as PrimitiveComboboxList, ComboboxMulti as PrimitiveComboboxMulti,
    ComboboxOption as PrimitiveComboboxOption,
};

pub use adico_primitives::combobox::{
    ComboboxItemIndicator, ComboboxItemIndicatorProps, default_combobox_filter,
};

/// A positioned root retaining the primitive's controlled state and filtering API.
#[component]
pub fn Combobox<T: Clone + PartialEq + 'static>(
    #[props(default)] value: Option<ReadSignal<Option<T>>>,
    #[props(default)] default_value: Option<T>,
    #[props(default)] on_value_change: Callback<Option<T>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: ReadSignal<bool>,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] query: ReadSignal<Option<String>>,
    #[props(default)] default_query: ReadSignal<String>,
    #[props(default)] on_query_change: Callback<String>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    #[props(default = Callback::new(|(query, text): (String, String)| default_combobox_filter(&query, &text)))]
    filter: Callback<(String, String), bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&["group inline-block", class.as_deref().unwrap_or_default()]);
    rsx! {
        PrimitiveCombobox::<T> {
            value,
            default_value,
            on_value_change,
            disabled,
            open,
            default_open,
            on_open_change,
            query,
            default_query,
            on_query_change,
            roving_loop,
            filter,
            class,
            {children}
            ComboboxChevron {}
        }
    }
}

/// Trailing chevron shared by [`Combobox`] and [`ComboboxMulti`]; swaps
/// direction with the root's `data-state` rather than being duplicated per
/// consumer.
#[component]
fn ComboboxChevron() -> Element {
    rsx! {
        span {
            class: "pointer-events-none absolute right-3 top-1/2 inline-flex size-4 -translate-y-1/2 text-muted-foreground",
            "aria-hidden": "true",
            ChevronDown { class: "size-4 group-data-[state=open]:hidden", size: 16 }
            ChevronUp { class: "hidden size-4 group-data-[state=open]:block", size: 16 }
        }
    }
}

/// A styled filtered option with the same interaction treatment as
/// [`SelectOption`](crate::components::ui::SelectOption). The primitive owns
/// filtering, selection, focus, pointer, and keyboard behavior.
#[component]
pub fn ComboboxOption<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    #[props(default)] text_value: ReadSignal<Option<String>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] id: ReadSignal<Option<String>>,
    index: ReadSignal<usize>,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] aria_roledescription: Option<String>,
    #[props(default)] class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative flex cursor-default select-none items-center rounded-sm py-1.5 pr-2 pl-8 text-sm outline-none data-[highlighted=true]:bg-accent data-[highlighted=true]:text-accent-foreground data-[selected=true]:bg-accent data-[selected=true]:font-medium data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveComboboxOption::<T> {
            value,
            text_value,
            disabled,
            id,
            index,
            aria_label,
            aria_roledescription,
            class,
            span { class: "absolute left-2 flex size-3.5 items-center justify-center", "aria-hidden": "true",
                ComboboxItemIndicator { "✓" }
            }
            {children}
        }
    }
}

/// The semantic empty state shown when no combobox option matches the query.
#[component]
pub fn ComboboxEmpty(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "px-2 py-1.5 text-sm text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        adico_primitives::combobox::ComboboxEmpty { class, {children} }
    }
}

/// A styled multiple-value autocomplete. Options toggle while the popup stays
/// open, preserving the owned primitive's filter, keyboard, and ARIA model.
#[component]
pub fn ComboboxMulti<T: Clone + PartialEq + 'static>(
    #[props(default)] values: ReadSignal<Option<Vec<T>>>,
    #[props(default)] default_values: Vec<T>,
    #[props(default)] on_values_change: Callback<Vec<T>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: ReadSignal<bool>,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] query: ReadSignal<Option<String>>,
    #[props(default)] default_query: ReadSignal<String>,
    #[props(default)] on_query_change: Callback<String>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    #[props(default = Callback::new(|(query, text): (String, String)| default_combobox_filter(&query, &text)))]
    filter: Callback<(String, String), bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&["group inline-block", class.as_deref().unwrap_or_default()]);
    rsx! {
        PrimitiveComboboxMulti::<T> {
            values,
            default_values,
            on_values_change,
            disabled,
            open,
            default_open,
            on_open_change,
            query,
            default_query,
            on_query_change,
            roving_loop,
            filter,
            class,
            {children}
            ComboboxChevron {}
        }
    }
}

/// A styled autocomplete input retaining primitive keyboard behavior.
#[component]
pub fn ComboboxInput(
    placeholder: Option<String>,
    id: Option<String>,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "h-9 w-full min-w-48 rounded-md border border-input bg-background px-3 pr-8 text-sm outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:ring-1 focus-visible:ring-ring/25 disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveComboboxInput {
            id,
            placeholder: placeholder.unwrap_or_default(),
            class,
        }
    }
}

/// An absolutely positioned listbox so opening it does not shift layout.
#[component]
pub fn ComboboxList(children: Element, id: Option<String>, class: Option<String>) -> Element {
    // `w-full` would now mean 100% of the viewport (`Positioner`'s
    // `position: fixed` has no positioned ancestor to size against, unlike
    // the `absolute`-positioned listbox this replaced) — `min-w-48` is the
    // width baseline instead, matching `popover.rs`'s own fixed-width
    // precedent rather than trying to exactly match the input's width.
    let class = cn(&[
        "z-50 max-h-72 min-w-48 overflow-y-auto rounded-md bg-popover p-1 text-popover-foreground shadow-md outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveComboboxList {
            id,
            class,
            {children}
        }
    }
}
