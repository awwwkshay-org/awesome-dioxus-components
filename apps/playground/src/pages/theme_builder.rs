use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ThemeBuilderPage() -> Element {
    rsx! {
        Demo {
            name: "ThemeBuilder",
            wide: true,
            div { class: "w-full max-w-md", components::ui::ThemeBuilder {} }
        }
    }
}
