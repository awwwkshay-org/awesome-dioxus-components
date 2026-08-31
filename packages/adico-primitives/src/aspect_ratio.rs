// No ARIA role applies (a layout container is not itself a widget); its spec is the classic
// "padding-bottom percentage hack" for a fixed-aspect-ratio box (the nested-container +
// absolute-inset structure below is what makes it work without an intrinsic-ratio CSS property
// dependency), which this file already implements. `ratio` is a *ratio*, so the padding
// percentage is `100 / ratio`, not `100 * ratio`.

//! Defines the [`AspectRatio`] component, which maintains a specific aspect ratio for its children.

use dioxus::prelude::*;

/// The props for the [`AspectRatio`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AspectRatioProps {
    /// The desired ratio. E.g. 16.0 / 9.0
    #[props(default = 1.0)]
    pub ratio: f64,

    /// Additional attributes to apply to the aspect ratio container.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children to render inside the aspect ratio container.
    pub children: Element,
}

/// # AspectRatio
///
/// A component that maintains a specific aspect ratio for its children.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::aspect_ratio::AspectRatio;
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         AspectRatio { ratio: 16.0 / 9.0,
///             div { style: "background-color: lightblue; width: 100%; height: 100%;",
///                 "This div maintains a 16:9 aspect ratio."
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let ratio = 100.0 / (props.ratio);

    rsx! {
        div {
            style: "position: relative; width: 100%; padding-bottom: {ratio}%;",
            div {
                style: "position: absolute; inset: 0;",
                ..props.attributes,

                {props.children}
            }
        }
    }
}
