use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{SelectTriggerControls, SelectTriggerDemoState};

#[component]
pub fn SelectPage() -> Element {
    let disabled = use_signal(|| false);
    let multiple = use_signal(|| false);
    let mut value = use_signal(|| None::<&'static str>);
    let value_string = use_memo(move || value().map(str::to_string));
    let mut values = use_signal(|| Some(Vec::<String>::new()));
    let open = use_signal(|| None::<bool>);
    let trigger_state = use_signal(SelectTriggerDemoState::default);
    rsx! {
        Demo {
            name: "Select",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Multi-select", value: multiple }
                SelectTriggerControls { state: trigger_state }
                if !multiple() {
                    SelectControl {
                        label: "Value",
                        value,
                        options: &[("None", None), ("Apple", Some("apple")), ("Banana", Some("banana"))],
                    }
                } else {
                    p { class: "self-end pb-2 text-sm text-muted-foreground", "Choose one or more options in the preview." }
                }
                OptionalBoolControl { label: "Open state", value: open }
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
                        aria_invalid: trigger_state().aria_invalid,
                        size: trigger_state().size,
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
                    value: ReadSignal::from(value_string),
                    open: open,
                    on_value_change: move |next: Option<String>| {
                        value
                            .set(
                                match next.as_deref() {
                                    Some("apple") => Some("apple"),
                                    Some("banana") => Some("banana"),
                                    _ => None,
                                },
                            );
                    },
                    components::ui::SelectTrigger {
                        class: "w-48",
                        aria_label: "Choose a fruit",
                        aria_invalid: trigger_state().aria_invalid,
                        size: trigger_state().size,
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
