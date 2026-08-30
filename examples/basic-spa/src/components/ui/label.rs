//! Source-owned shadcn-style Label for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::label::Label as LabelPrimitive;

use crate::adico_lib::cn::cn;

/// Props for [`Label`].
#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    /// The id of the form element this label is associated with.
    pub html_for: ReadSignal<String>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed label content.
    pub children: Element,
}

/// A form label with the default adico/shadcn visual language.
#[component]
pub fn Label(props: LabelProps) -> Element {
    let class = cn(&[
        "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        LabelPrimitive {
            html_for: props.html_for,
            class,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_class_disables_visually_on_the_disabled_peer() {
        let class = cn(&[
            "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
            "",
        ]);
        assert!(class.contains("peer-disabled:opacity-70"));
    }
}
