use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn AvatarPage() -> Element {
    rsx! {
        Demo {
            name: "Avatar",
            components::ui::Avatar {
                components::ui::AvatarFallback { "AB" }
            }
        }
    }
}
