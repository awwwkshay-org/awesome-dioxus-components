use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, TextControl};
use crate::components::demo::Demo;

#[component]
pub fn CardPage() -> Element {
    let show_footer = use_signal(|| true);
    let title = use_signal(|| "Card title".to_string());
    let description = use_signal(|| "Supporting description text.".to_string());
    rsx! {
        Demo {
            name: "Card",
            controls: rsx! {
                TextControl { label: "Title", value: title }
                TextControl { label: "Description", value: description }
                BoolControl { label: "Show actions", value: show_footer }
            },
            components::ui::Card { class: "max-w-md",
                components::ui::CardHeader {
                    components::ui::CardTitle { "{title}" }
                    components::ui::CardDescription { "{description}" }
                }
                components::ui::CardContent { "Card body content uses composed semantic regions." }
                if show_footer() {
                    components::ui::CardFooter {
                        components::ui::Button { variant: components::ui::ButtonVariant::Outline, "Cancel" }
                        components::ui::Button { "Continue" }
                    }
                }
            }
        }
    }
}
