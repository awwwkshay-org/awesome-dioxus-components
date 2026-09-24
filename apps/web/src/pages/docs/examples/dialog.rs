//! Examples for `dialog`.
//!
//! Deliberately closed by default: a variant grid that auto-opened several
//! modals would fight for the viewport, and anchored/portalled content on this
//! page has a known pre-existing `visibility: hidden` failure mode.

use dioxus::prelude::*;

use super::DocExampleMeta;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogOverlay,
    DialogTitle, DialogTrigger,
};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;

pub const SRC: &str = include_str!("dialog.rs");

pub const METAS: &[DocExampleMeta] = &[DocExampleMeta {
    id: "composition",
    title: "Composition",
    description: "Trigger, overlay, and content are siblings inside `Dialog`, not nested — the overlay and content are portalled, so nesting them under the trigger would trap them in its stacking context. Click to open.",
}];

pub fn render(id: &str) -> Element {
    match id {
        "composition" => rsx! { Composition {} },
        _ => rsx! {},
    }
}

#[component]
fn Composition() -> Element {
    rsx! {
        // doc-example:start composition
        Dialog {
            DialogTrigger { "Edit profile" }
            DialogOverlay {}
            DialogContent {
                DialogHeader {
                    DialogTitle { "Edit profile" }
                    DialogDescription {
                        "Make changes to your profile here. Click save when you're done."
                    }
                }
                div { class: "flex flex-col gap-2 py-2",
                    Label { html_for: "dialog-example-name", "Name" }
                    Input { id: "dialog-example-name", value: Some("Ada Lovelace".to_string()) }
                }
                DialogFooter {
                    Button { variant: ButtonVariant::Outline, "Cancel" }
                    Button { "Save changes" }
                }
            }
        }
        // doc-example:end
    }
}
