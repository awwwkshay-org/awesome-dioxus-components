use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn CheckboxPage() -> Element {
    let mut checked = use_signal(|| components::ui::CheckboxState::Unchecked);
    rsx! {
        Demo {
            name: "Checkbox",
            components::ui::Checkbox {
                checked: checked(),
                on_checked_change: move |value| checked.set(value),
                aria_label: "Accept terms",
            }
        }
    }
}
