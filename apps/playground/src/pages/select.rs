use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn SelectPage() -> Element {
    let disabled = use_signal(|| false);
    let multiple = use_signal(|| false);
    let mut value = use_signal(|| None::<String>);
    let mut values = use_signal(|| Some(Vec::<String>::new()));
    let mut open = use_signal(|| None::<bool>);
    let invalid = use_signal(|| false);
    rsx! {
        Demo {
            name: "Select",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Multi-select", value: multiple }
                BoolControl { label: "Invalid presentation", value: invalid }
                if !multiple() {
                    SelectControl {
                        label: "Value",
                        value: value(),
                        options: vec![
                            ("None", None),
                            ("Apple", Some("apple".to_string())),
                            ("Banana", Some("banana".to_string())),
                        ],
                        on_change: move |next| value.set(next),
                    }
                } else {
                    p { class: "self-end pb-2 text-sm text-muted-foreground", "Choose one or more options in the preview." }
                }
                SelectControl {
                    label: "Open state",
                    value: open(),
                    options: vec![("Uncontrolled", None), ("Closed", Some(false)), ("Open", Some(true))],
                    on_change: move |next| open.set(next),
                }
            },
            if multiple() {
                components::ui::SelectMulti::<String> {
                    disabled: disabled(),
                    values: ReadSignal::from(values),
                    open: open,
                    on_values_change: move |next| values.set(Some(next)),
                    components::ui::SelectTrigger {
                        class: "w-48",
                        aria_label: "Choose one or more fruits",
                        aria_invalid: invalid(),
                        components::ui::SelectValue { placeholder: "Choose fruits" }
                    }
                    components::ui::SelectList { class: "w-48", aria_label: "Fruit options",
                        components::ui::SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                        components::ui::SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                    }
                }
            } else {
                components::ui::Select::<String> {
                    disabled: disabled(),
                    value: Some(ReadSignal::from(value)),
                    open: open,
                    on_value_change: move |next| value.set(next),
                    components::ui::SelectTrigger {
                        class: "w-48",
                        aria_label: "Choose a fruit",
                        aria_invalid: invalid(),
                        components::ui::SelectValue { placeholder: "Choose a fruit" }
                    }
                    components::ui::SelectList { class: "w-48", aria_label: "Fruit options",
                        components::ui::SelectOption::<String> { index: 0usize, value: "apple", text_value: "Apple", "Apple" }
                        components::ui::SelectOption::<String> { index: 1usize, value: "banana", text_value: "Banana", "Banana" }
                    }
                }
            }
        }
    }
}
