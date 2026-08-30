//! Source-owned Dioxus-only Tag Group for Dioxus, backed by the owned adico
//! primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

pub use adico_primitives::tag_group::{TagGroupEmpty, TagGroupEmptyProps};
use adico_primitives::tag_group::{
    TagGroup as TagGroupPrimitive, TagGroupLabel as TagGroupLabelPrimitive,
    TagGroupMulti as TagGroupMultiPrimitive, TagList as TagListPrimitive,
    TagOption as TagOptionPrimitive, TagRemoveButton as TagRemoveButtonPrimitive,
};

use crate::adico_lib::cn::cn;

/// A focusable group of tags with single selection, styled with the semantic
/// surface tokens.
#[component]
pub fn TagGroup<T: Clone + PartialEq + 'static>(
    #[props(default)] value: Option<ReadSignal<Option<T>>>,
    #[props(default)] default_value: Option<T>,
    #[props(default)] on_value_change: Callback<Option<T>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] selectable: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] allow_empty_selection: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] escape_clears_selection: ReadSignal<
        bool,
    >,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&["flex flex-col gap-2", class.as_deref().unwrap_or_default()]);
    rsx! {
        TagGroupPrimitive::<T> {
            value,
            default_value,
            on_value_change,
            disabled,
            selectable,
            allow_empty_selection,
            escape_clears_selection,
            roving_loop,
            class,
            {children}
        }
    }
}

/// A focusable group of tags with multiple selection, styled with the
/// semantic surface tokens.
#[component]
pub fn TagGroupMulti<T: Clone + PartialEq + 'static>(
    #[props(default)] values: ReadSignal<Option<Vec<T>>>,
    #[props(default)] default_values: Vec<T>,
    #[props(default)] on_values_change: Callback<Vec<T>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] selectable: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] allow_empty_selection: ReadSignal<bool>,
    #[props(default = ReadSignal::new(Signal::new(true)))] escape_clears_selection: ReadSignal<
        bool,
    >,
    #[props(default = ReadSignal::new(Signal::new(true)))] roving_loop: ReadSignal<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&["flex flex-col gap-2", class.as_deref().unwrap_or_default()]);
    rsx! {
        TagGroupMultiPrimitive::<T> {
            values,
            default_values,
            on_values_change,
            disabled,
            selectable,
            allow_empty_selection,
            escape_clears_selection,
            roving_loop,
            class,
            {children}
        }
    }
}

/// Visible label for a [`TagGroup`]/[`TagGroupMulti`].
#[component]
pub fn TagGroupLabel(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "text-sm font-medium text-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagGroupLabelPrimitive { class, {children} }
    }
}

/// Wrapping row container for [`TagOption`] tags.
#[component]
pub fn TagList(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-wrap items-center gap-1.5 outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagListPrimitive { class, {children} }
    }
}

/// A single tag inside a [`TagList`], styled to match the badge/secondary
/// surface used elsewhere in this registry.
#[component]
pub fn TagOption<T: Clone + PartialEq + 'static>(
    value: ReadSignal<T>,
    index: ReadSignal<usize>,
    #[props(default)] text_value: ReadSignal<Option<String>>,
    #[props(default)] disabled: ReadSignal<bool>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex items-center gap-1 rounded-md border border-transparent bg-secondary px-2 py-1 text-xs font-medium text-secondary-foreground outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring/50 data-[selected=true]:bg-primary data-[selected=true]:text-primary-foreground data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagOptionPrimitive::<T> {
            value,
            index,
            text_value,
            disabled,
            class,
            {children}
        }
    }
}

/// Remove button for the enclosing [`TagOption`]. Rendering this makes the
/// tag removable via click and Delete/Backspace.
#[component]
pub fn TagRemoveButton(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "ml-0.5 inline-flex size-3.5 items-center justify-center rounded-full outline-none hover:bg-black/10 disabled:pointer-events-none disabled:opacity-50 dark:hover:bg-white/10",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagRemoveButtonPrimitive { class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_tag_uses_the_semantic_primary_surface() {
        let class = cn(&["data-[selected=true]:bg-primary", ""]);
        assert!(class.contains("data-[selected=true]:bg-primary"));
    }
}
