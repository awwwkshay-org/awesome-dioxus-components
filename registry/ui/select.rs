//! Styled Select parts backed by the owned `adico-primitives` behavior layer.
//!
//! `Select` retains the primitive's typed controlled value/open state,
//! keyboard navigation, typeahead, focus, and ARIA contract. The styled parts
//! below supply the default adico/shadcn semantic surface; callers can still
//! pass Dioxus attributes and compose groups or indicators normally.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use adico_primitives::icons::{ChevronDown, ChevronUp};

use adico_primitives::select::{
    Select as PrimitiveSelect, SelectList as PrimitiveSelectList,
    SelectMulti as PrimitiveSelectMulti, SelectOption as PrimitiveSelectOption,
    SelectTrigger as PrimitiveSelectTrigger, SelectValue as PrimitiveSelectValue,
};

pub use adico_primitives::select::{
    SelectGroup, SelectGroupLabel, SelectGroupLabelProps, SelectGroupProps, SelectItemIndicator,
    SelectItemIndicatorProps,
};

/// A positioned Select root retaining the primitive's complete state model.
#[component]
pub fn Select<T: Clone + PartialEq + 'static>(
    #[props(default)] value: Option<ReadSignal<Option<T>>>,
    #[props(default)] default_value: Option<T>,
    #[props(default)] on_value_change: Callback<Option<T>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: ReadSignal<bool>,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] name: ReadSignal<String>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(std::time::Duration::from_millis(1000))))]
    typeahead_timeout: ReadSignal<std::time::Duration>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative inline-block",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveSelect::<T> {
            value,
            default_value,
            on_value_change,
            disabled,
            open,
            default_open,
            on_open_change,
            name,
            roving_loop,
            typeahead_timeout,
            class,
            {children}
        }
    }
}

/// A styled multiple-value root backed by the primitive's toggle selection
/// behavior. Its trigger, value, list, and options are shared with [`Select`].
#[component]
pub fn SelectMulti<T: Clone + PartialEq + 'static>(
    #[props(default)] values: ReadSignal<Option<Vec<T>>>,
    #[props(default)] default_values: Vec<T>,
    #[props(default)] on_values_change: Callback<Vec<T>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] open: ReadSignal<Option<bool>>,
    #[props(default)] default_open: ReadSignal<bool>,
    #[props(default)] on_open_change: Callback<bool>,
    #[props(default)] name: ReadSignal<String>,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(std::time::Duration::from_millis(1000))))]
    typeahead_timeout: ReadSignal<std::time::Duration>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative inline-block",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveSelectMulti::<T> {
            values,
            default_values,
            on_values_change,
            disabled,
            open,
            default_open,
            on_open_change,
            name,
            roving_loop,
            typeahead_timeout,
            class,
            {children}
        }
    }
}

/// A shadcn-style native trigger for the primitive [`Select`].
#[component]
pub fn SelectTrigger(
    children: Element,
    class: Option<String>,
    aria_label: Option<String>,
    aria_invalid: Option<bool>,
) -> Element {
    let class = cn(&[
        "group flex h-9 w-full min-w-32 items-center justify-between gap-2 rounded-md border border-input bg-background px-3 text-sm outline-none transition-[color,box-shadow] focus-visible:ring-1 focus-visible:ring-ring/25 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveSelectTrigger {
            class,
            aria_label,
            aria_invalid,
            {children}
            span { class: "relative inline-flex size-4 shrink-0 text-muted-foreground", "aria-hidden": "true",
                ChevronDown { class: "size-4 group-aria-expanded:hidden", size: 16 }
                ChevronUp { class: "hidden size-4 group-aria-expanded:block", size: 16 }
            }
        }
    }
}

/// A complete, keyboard-aware option with shadcn-style hover, focus, selected,
/// and disabled states. The selected checkmark is supplied by the primitive's
/// item-indicator context, so it works for both [`Select`] and [`SelectMulti`].
#[component]
pub fn SelectOption<T: Clone + PartialEq + 'static>(
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
        PrimitiveSelectOption::<T> {
            value,
            text_value,
            disabled,
            id,
            index,
            aria_label,
            aria_roledescription,
            class,
            span { class: "absolute left-2 flex size-3.5 items-center justify-center", "aria-hidden": "true",
                SelectItemIndicator { "✓" }
            }
            {children}
        }
    }
}

/// The selected value or placeholder inside a [`SelectTrigger`].
#[component]
pub fn SelectValue(placeholder: Option<String>, class: Option<String>) -> Element {
    let class = cn(&[
        "line-clamp-1 flex-1 text-left data-[placeholder=true]:text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveSelectValue {
            class,
            placeholder: placeholder.unwrap_or_else(|| "Select an option".to_string()),
        }
    }
}

/// A styled listbox surface. The primitive retains focus and keyboard logic.
#[component]
pub fn SelectList(
    children: Element,
    id: Option<String>,
    class: Option<String>,
    aria_label: Option<String>,
) -> Element {
    let class = cn(&[
        "absolute left-0 top-full z-50 max-h-72 min-w-32 overflow-y-auto rounded-md bg-popover p-1 text-popover-foreground shadow-md outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        PrimitiveSelectList {
            id,
            class,
            aria_label,
            {children}
        }
    }
}
