use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::SpinnerControls;

#[component]
pub fn SpinnerPage() -> Element {
    rsx! {
        Demo {
            name: "Spinner",
            controls: rsx! {
                SpinnerControls {}
            },
            components::ui::Spinner { class: "size-8 text-primary" }
        }
    }
}
