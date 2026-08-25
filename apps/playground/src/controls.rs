//! Small bound controls used on component pages to edit a prop live and
//! re-render the demo immediately, Storybook-style.

use dioxus::prelude::*;

#[component]
pub fn BoolControl(label: &'static str, value: Signal<bool>) -> Element {
    rsx! {
        label { class: "flex items-center gap-2 text-sm",
            input {
                r#type: "checkbox",
                checked: value(),
                onchange: move |event| value.set(event.checked()),
            }
            "{label}"
        }
    }
}

#[component]
pub fn TextControl(label: &'static str, value: Signal<String>) -> Element {
    rsx! {
        label { class: "flex items-center gap-2 text-sm",
            "{label}"
            input {
                class: "rounded border px-2 py-1 text-sm",
                r#type: "text",
                value: "{value}",
                oninput: move |event| value.set(event.value()),
            }
        }
    }
}

/// A closed-enum control: renders a `<select>` over `options` (display label,
/// value) and reports the chosen value through `on_change`.
#[component]
pub fn SelectControl<T: Clone + PartialEq + 'static>(
    label: &'static str,
    value: T,
    options: Vec<(&'static str, T)>,
    on_change: EventHandler<T>,
) -> Element {
    rsx! {
        label { class: "flex items-center gap-2 text-sm",
            "{label}"
            select {
                class: "rounded border px-2 py-1 text-sm",
                onchange: move |event| {
                    if let Ok(index) = event.value().parse::<usize>() {
                        if let Some((_, option)) = options.get(index) {
                            on_change.call(option.clone());
                        }
                    }
                },
                for (index , (option_label , option)) in options.iter().enumerate() {
                    option {
                        value: "{index}",
                        selected: *option == value,
                        "{option_label}"
                    }
                }
            }
        }
    }
}
