//! Source-owned Dioxus-only Tag Group for Dioxus, backed by the owned adico
//! primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

use adico_primitives::tag_group::{
    TagGroup as TagGroupPrimitive, TagGroupLabel as TagGroupLabelPrimitive,
    TagGroupMulti as TagGroupMultiPrimitive, TagList as TagListPrimitive,
    TagOption as TagOptionPrimitive, TagRemoveButton as TagRemoveButtonPrimitive,
};
pub use adico_primitives::tag_group::{TagGroupEmpty, TagGroupEmptyProps};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::{Radius, Tone};

/// The shape/structure of a selected [`TagOption`], independent of its
/// [`Tone`] color (`TagOption::color`). Both props affect only the
/// *selected*-state fill; a `TagOption`'s resting (unselected) look is
/// always the existing neutral `bg-secondary`/`text-secondary-foreground`
/// surface, regardless of shape or color.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TagOptionVariant {
    /// Filled, solid selected fill.
    #[default]
    Primary,
    /// Muted filled selected fill.
    Secondary,
    /// Bordered, unfilled selected fill.
    Outline,
    /// Transparent-until-hover selected fill.
    Ghost,
    /// Inline semantic-link selected styling.
    Link,
}

/// [`TagOption`]'s selected-state fill per shape family, as
/// `data-[selected=true]:` prefixed literals -- Tailwind's source scanner
/// extracts class names as literal text, so these cannot be built at
/// runtime via `format!`, and cannot reuse [`Tone`]'s own (unprefixed)
/// methods directly; each combination must appear verbatim in source.
/// Structurally mirrors `Tone`'s five methods one-for-one so the two stay
/// easy to keep in sync by eye.
fn selected_solid_class(color: Tone) -> &'static str {
    match color {
        Tone::Default => {
            "data-[selected=true]:bg-primary data-[selected=true]:text-primary-foreground"
        }
        Tone::Success => {
            "data-[selected=true]:bg-success data-[selected=true]:text-success-foreground"
        }
        Tone::Warning => {
            "data-[selected=true]:bg-warning data-[selected=true]:text-warning-foreground"
        }
        Tone::Error => {
            "data-[selected=true]:bg-destructive data-[selected=true]:text-destructive-foreground"
        }
        Tone::Info => "data-[selected=true]:bg-info data-[selected=true]:text-info-foreground",
    }
}

fn selected_soft_class(color: Tone) -> &'static str {
    match color {
        Tone::Default => {
            "data-[selected=true]:bg-secondary data-[selected=true]:text-secondary-foreground"
        }
        Tone::Success => "data-[selected=true]:bg-success/15 data-[selected=true]:text-success",
        Tone::Warning => "data-[selected=true]:bg-warning/15 data-[selected=true]:text-warning",
        Tone::Error => {
            "data-[selected=true]:bg-destructive/15 data-[selected=true]:text-destructive"
        }
        Tone::Info => "data-[selected=true]:bg-info/15 data-[selected=true]:text-info",
    }
}

fn selected_outline_class(color: Tone) -> &'static str {
    match color {
        Tone::Default => "data-[selected=true]:border-input data-[selected=true]:bg-background",
        Tone::Success => "data-[selected=true]:border-success data-[selected=true]:text-success",
        Tone::Warning => "data-[selected=true]:border-warning data-[selected=true]:text-warning",
        Tone::Error => {
            "data-[selected=true]:border-destructive data-[selected=true]:text-destructive"
        }
        Tone::Info => "data-[selected=true]:border-info data-[selected=true]:text-info",
    }
}

fn selected_ghost_class(color: Tone) -> &'static str {
    match color {
        Tone::Default => "",
        Tone::Success => "data-[selected=true]:text-success",
        Tone::Warning => "data-[selected=true]:text-warning",
        Tone::Error => "data-[selected=true]:text-destructive",
        Tone::Info => "data-[selected=true]:text-info",
    }
}

fn selected_link_class(color: Tone) -> &'static str {
    match color {
        Tone::Default => "data-[selected=true]:text-primary data-[selected=true]:underline",
        Tone::Success => "data-[selected=true]:text-success data-[selected=true]:underline",
        Tone::Warning => "data-[selected=true]:text-warning data-[selected=true]:underline",
        Tone::Error => "data-[selected=true]:text-destructive data-[selected=true]:underline",
        Tone::Info => "data-[selected=true]:text-info data-[selected=true]:underline",
    }
}

fn tag_option_class(shape: TagOptionVariant, color: Tone) -> &'static str {
    match shape {
        TagOptionVariant::Primary => selected_solid_class(color),
        TagOptionVariant::Secondary => selected_soft_class(color),
        TagOptionVariant::Outline => selected_outline_class(color),
        TagOptionVariant::Ghost => selected_ghost_class(color),
        TagOptionVariant::Link => selected_link_class(color),
    }
}

/// A focusable group of tags with single selection, styled with the semantic
/// surface tokens.
#[component]
pub fn TagGroup<T: Clone + PartialEq + 'static>(
    /// **BREAKING** (task 2.6): was `Option<ReadSignal<Option<T>>>`; see
    /// `select.rs`'s own `Select::value` doc comment for the full rationale.
    #[props(default = ReadSignal::new(Signal::new(None)))]
    value: ReadSignal<Option<T>>,
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
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&["flex flex-col gap-2", class.as_deref().unwrap_or_default()]);
    rsx! {
        TagGroupPrimitive::<T> {
            value: Some(value),
            default_value,
            on_value_change,
            disabled,
            selectable,
            allow_empty_selection,
            escape_clears_selection,
            roving_loop,
            class,
            attributes,
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
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
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
            attributes,
            {children}
        }
    }
}

/// Visible label for a [`TagGroup`]/[`TagGroupMulti`].
#[component]
pub fn TagGroupLabel(
    children: Element,
    id: Option<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "text-sm font-medium text-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagGroupLabelPrimitive { id, class, attributes, {children} }
    }
}

/// Wrapping row container for [`TagOption`] tags.
#[component]
pub fn TagList(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex flex-wrap items-center gap-1.5 outline-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagListPrimitive { class, attributes, {children} }
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
    id: Option<String>,
    #[props(default = Radius::Md)] radius: Radius,
    /// Shape/structure of this tag's selected-state fill, independent of
    /// `color`. Has no effect on the resting (unselected) look.
    #[props(default)]
    variant: TagOptionVariant,
    /// Color of this tag's selected-state fill, independent of `variant`.
    /// Has no effect on the resting (unselected) look.
    #[props(default)]
    color: Tone,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let class = cn(&[
        "inline-flex items-center gap-1 border border-transparent bg-secondary px-2 py-1 text-xs font-medium text-secondary-foreground outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring/50 data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50",
        tag_option_class(variant, color),
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagOptionPrimitive::<T> {
            value,
            index,
            text_value,
            disabled,
            id,
            class,
            attributes,
            {children}
        }
    }
}

/// Remove button for the enclosing [`TagOption`]. Rendering this makes the
/// tag removable via click and Delete/Backspace.
#[component]
pub fn TagRemoveButton(
    children: Element,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "ml-0.5 inline-flex size-3.5 items-center justify-center rounded-full outline-none hover:bg-black/10 disabled:pointer-events-none disabled:opacity-50 dark:hover:bg-white/10",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TagRemoveButtonPrimitive { class, attributes, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shape_and_color_matches_the_pre_existing_selected_surface() {
        let class = tag_option_class(TagOptionVariant::Primary, Tone::Default);
        assert_eq!(
            class,
            "data-[selected=true]:bg-primary data-[selected=true]:text-primary-foreground"
        );
    }

    #[test]
    fn every_shape_and_non_default_color_combination_names_its_token() {
        for shape in [
            TagOptionVariant::Primary,
            TagOptionVariant::Secondary,
            TagOptionVariant::Outline,
            TagOptionVariant::Ghost,
            TagOptionVariant::Link,
        ] {
            for color in [Tone::Success, Tone::Warning, Tone::Error, Tone::Info] {
                let class = tag_option_class(shape, color);
                assert!(
                    class.contains(color.name()),
                    "{class} missing {}",
                    color.name()
                );
                assert!(class.starts_with("data-[selected=true]:") || class.is_empty());
            }
        }
    }
}
