// Implements a Dioxus-idiomatic translation of Base UI's `Fieldset`
// anatomy (`Fieldset.Root`/`Fieldset.Legend`), a semantic `<fieldset>`/
// `<legend>` group whose `disabled` cascades to every nested
// [`crate::field::FieldRoot`]'s own `data-disabled`/[`crate::field::use_field_control`]
// state -- not just to native form controls, which a plain `<fieldset
// disabled>` already disables for free per the HTML spec without any of
// this module's help. This module exists specifically so a nested
// `FieldRoot`'s *own* Dioxus-side state (used for `data-disabled` styling,
// and read by any control composing `use_field_control`) stays in sync with
// that native cascade, matching Base UI's own `FieldsetRootContext`.

//! Defines the [`FieldsetRoot`] and [`FieldsetLegend`] components.

use dioxus::prelude::*;

#[derive(Clone, Copy)]
struct FieldsetContext {
    disabled: ReadSignal<bool>,
}

/// The props for the [`FieldsetRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FieldsetRootProps {
    /// Whether every field nested inside this fieldset should ignore user
    /// interaction.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the fieldset element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the fieldset: typically a [`FieldsetLegend`] followed
    /// by one or more [`crate::field::FieldRoot`]s.
    pub children: Element,
}

/// # FieldsetRoot
///
/// The `FieldsetRoot` component renders a native `fieldset` element and
/// cascades its `disabled` state to every nested [`crate::field::FieldRoot`].
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::fieldset::{FieldsetRoot, FieldsetLegend};
/// use adico_primitives::field::{FieldRoot, FieldLabel};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         FieldsetRoot {
///             FieldsetLegend { "Shipping address" }
///             FieldRoot {
///                 FieldLabel { "Street" }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn FieldsetRoot(props: FieldsetRootProps) -> Element {
    use_context_provider(|| FieldsetContext {
        disabled: props.disabled,
    });
    rsx! {
        fieldset { disabled: props.disabled, ..props.attributes, {props.children} }
    }
}

/// Read the disabled state cascaded down from an enclosing
/// [`FieldsetRoot`], or `false` if there isn't one. Used internally by
/// [`crate::field::FieldRoot`]; exposed publicly for any other component
/// that wants to react to the same cascade.
pub fn use_fieldset_disabled() -> ReadSignal<bool> {
    try_consume_context::<FieldsetContext>()
        .map(|ctx| ctx.disabled)
        .unwrap_or_else(|| ReadSignal::new(Signal::new(false)))
}

/// The props for the [`FieldsetLegend`] component.
#[derive(Props, Clone, PartialEq)]
pub struct FieldsetLegendProps {
    /// Additional attributes to apply to the legend element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the legend element.
    pub children: Element,
}

/// # FieldsetLegend
///
/// The `FieldsetLegend` component renders a native `legend` element,
/// labeling the enclosing [`FieldsetRoot`].
///
/// This must be used inside a [`FieldsetRoot`] component.
#[component]
pub fn FieldsetLegend(props: FieldsetLegendProps) -> Element {
    rsx! {
        legend { ..props.attributes, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_enclosing_fieldset_defaults_to_not_disabled() {
        let mut dom = VirtualDom::new(|| {
            let disabled = use_fieldset_disabled();
            rsx! {
                div { "{disabled()}" }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("false"), "{html}");
    }
}
