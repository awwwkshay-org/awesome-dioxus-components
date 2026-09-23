use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{ControlGroup, TextControl};
use crate::components::demo::Demo;
use crate::generated::controls::CopyButtonControls;

#[component]
pub fn CopyButtonPage() -> Element {
    let value = use_signal(|| "npm install adico-cli".to_string());
    rsx! {
        Demo {
            name: "Copy Button",
            controls: rsx! {
                ControlGroup { part: "Copy Button",
                    TextControl { label: "Value", value }
                }
                CopyButtonControls {}
            },
            div { class: "flex items-center gap-2 rounded-md border px-3 py-2 font-mono text-sm",
                span { class: "flex-1 truncate", "{value}" }
                components::ui::CopyButton { value: value() }
            }
        }
    }
}
