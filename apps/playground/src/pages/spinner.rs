use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn SpinnerPage() -> Element {
    rsx! {
        Demo { name: "Spinner",
            components::ui::Spinner { class: "size-8 text-primary" }
        }
    }
}
