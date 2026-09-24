//! Source-owned Dioxus-only Copy Button for Dioxus, backed by the owned
//! adico primitive layer. This is a Dioxus Components extra with no shadcn
//! equivalent -- it does not count toward shadcn parity.

use dioxus::prelude::*;

use adico_primitives::clipboard::{ClipboardStatus, use_clipboard};
use adico_primitives::icons::{Check, Copy};

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;

#[derive(Props, Clone, PartialEq)]
pub struct CopyButtonProps {
    /// The text copied to the clipboard when this button is activated.
    pub value: ReadSignal<String>,
    #[props(default)]
    pub class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,
}

/// A small, generic button that copies `value` to the clipboard, showing a
/// brief checkmark confirmation. Composable next to any displayed text value
/// -- not coupled to any one other component.
#[component]
pub fn CopyButton(props: CopyButtonProps) -> Element {
    let (status, copy) = use_clipboard();
    let label = match status() {
        ClipboardStatus::Copied => "Copied",
        ClipboardStatus::Failed => "Copy failed",
        ClipboardStatus::Idle => "Copy to clipboard",
    };
    let class = cn(&["shrink-0", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            class,
            "aria-label": "{label}",
            onclick: move |_| copy(props.value.cloned()),
            attributes: props.attributes,
            if status() == ClipboardStatus::Copied {
                Check { class: "size-4", size: 16 }
            } else {
                Copy { class: "size-4", size: 16 }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_reflects_every_status() {
        assert_eq!(
            match ClipboardStatus::Idle {
                ClipboardStatus::Copied => "Copied",
                ClipboardStatus::Failed => "Copy failed",
                ClipboardStatus::Idle => "Copy to clipboard",
            },
            "Copy to clipboard"
        );
        assert_eq!(
            match ClipboardStatus::Copied {
                ClipboardStatus::Copied => "Copied",
                ClipboardStatus::Failed => "Copy failed",
                ClipboardStatus::Idle => "Copy to clipboard",
            },
            "Copied"
        );
        assert_eq!(
            match ClipboardStatus::Failed {
                ClipboardStatus::Copied => "Copied",
                ClipboardStatus::Failed => "Copy failed",
                ClipboardStatus::Idle => "Copy to clipboard",
            },
            "Copy failed"
        );
    }
}
