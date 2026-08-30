use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ThemeSwitcherPage() -> Element {
    rsx! {
        Demo {
            name: "ThemeSwitcher",
            components::ui::ThemeSwitcher {}
        }
    }
}
