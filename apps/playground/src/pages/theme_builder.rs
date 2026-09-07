use dioxus::prelude::*;

use crate::components::demo::Demo;
use crate::components::theme_builder_launcher::ThemeBuilderLauncher;

#[component]
pub fn ThemeBuilderPage() -> Element {
    rsx! {
        Demo {
            name: "Theme Builder",
            ThemeBuilderLauncher {}
        }
    }
}
