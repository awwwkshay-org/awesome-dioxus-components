use dioxus::prelude::*;

use crate::components;
use crate::components::controls::{BoolControl, OptionalBoolControl, SelectControl};
use crate::components::demo::Demo;

#[component]
pub fn ComboboxPage() -> Element {
    let disabled = use_signal(|| false);
    let multiple = use_signal(|| false);
    let mut value = use_signal(|| None::<&'static str>);
    let value_string = use_memo(move || value().map(str::to_string));
    let mut values = use_signal(|| Some(Vec::<String>::new()));
    let open = use_signal(|| None::<bool>);
    rsx! {
        Demo {
            name: "Combobox",
            controls: rsx! {
                BoolControl { label: "Disabled", value: disabled }
                BoolControl { label: "Multi-select", value: multiple }
                if !multiple() {
                    SelectControl {
                        label: "Value",
                        value,
                        options: &[("None", None), ("Apple", Some("Apple")), ("Banana", Some("Banana"))],
                    }
                } else {
                    p { class: "self-end pb-2 text-sm text-muted-foreground", "Choose one or more options in the preview." }
                }
                OptionalBoolControl { label: "Open state", value: open }
            },
            if multiple() {
                components::ui::ComboboxMulti::<String> {
                    disabled: disabled(),
                    values: ReadSignal::from(values),
                    open: open,
                    on_values_change: move |next| values.set(Some(next)),
                    components::ui::ComboboxInput { class: "w-48", placeholder: "Search fruits" }
                    components::ui::ComboboxList { class: "w-48",
                        components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                        components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                        components::ui::ComboboxEmpty { "No results" }
                    }
                }
            } else {
                components::ui::Combobox::<String> {
                    disabled: disabled(),
                    value: ReadSignal::from(value_string),
                    open: open,
                    on_value_change: move |next: Option<String>| {
                        value
                            .set(
                                match next.as_deref() {
                                    Some("Apple") => Some("Apple"),
                                    Some("Banana") => Some("Banana"),
                                    _ => None,
                                },
                            );
                    },
                    components::ui::ComboboxInput { class: "w-48", placeholder: "Search fruit" }
                    components::ui::ComboboxList { class: "w-48",
                        components::ui::ComboboxOption::<String> { value: "Apple".to_string(), index: 0usize, "Apple" }
                        components::ui::ComboboxOption::<String> { value: "Banana".to_string(), index: 1usize, "Banana" }
                        components::ui::ComboboxEmpty { "No results" }
                    }
                }
            }
        }
    }
}
