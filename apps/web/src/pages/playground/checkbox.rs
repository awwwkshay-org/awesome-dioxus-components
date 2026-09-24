use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::CheckboxControls;

#[component]
pub fn CheckboxPage() -> Element {
    let mut checked = use_signal(|| components::ui::CheckboxState::Unchecked);
    rsx! {
        Demo {
            name: "Checkbox",
            controls: rsx! {
                CheckboxControls {}
            },
            components::ui::Checkbox {
                checked: checked(),
                on_checked_change: move |value| checked.set(value),
                aria_label: "Accept terms",
            }
        }
    }
}
