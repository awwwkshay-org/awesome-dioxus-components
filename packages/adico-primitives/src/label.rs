// No dedicated WAI-ARIA APG pattern exists for a form label (it relies on native HTML `<label
// for>` semantics, which every assistive technology already understands without an ARIA role);
// its spec is that native contract, which this file already implements by rendering a plain
// `label` element with a `for` attribute.

//! Defines the [`Label`] component

use dioxus::prelude::*;

/// The props for the [`Label`] component
#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    /// The id of the element that this label is associated with
    pub html_for: ReadSignal<String>,

    /// Additional attributes to apply to the label element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the label element
    pub children: Element,
}

/// # Label
///
/// The `Label` component is used to create a label for form elements. It must be associated with an element using the [`LabelProps::html_for`] attribute.
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::label::Label;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Label {
///             html_for: "name",
///             "Name"
///         }
///
///         input {
///             id: "name",
///             placeholder: "Enter your name",
///         }
///     }
/// }
/// ```
#[component]
pub fn Label(props: LabelProps) -> Element {
    // TODO: (?) the Radix primitive prevents selection on double click (but not intentional highlighting)
    rsx! {
        label {
            for: props.html_for,
            ..props.attributes,

            {props.children}
        }
    }
}
