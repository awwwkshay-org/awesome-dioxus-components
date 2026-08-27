//! Small bound controls used on component pages to edit a prop live and
//! re-render the demo immediately, Storybook-style.

use dioxus::prelude::*;

#[component]
pub fn BoolControl(label: &'static str, value: Signal<bool>) -> Element {
    rsx! {
        label { class: "flex w-full flex-col gap-1 text-sm font-medium",
            span { "{label}" }
            span { class: "flex h-9 w-full items-center rounded-md border border-input bg-background px-3 shadow-xs",
                input {
                    r#type: "checkbox",
                    aria_label: label,
                    checked: value(),
                    onchange: move |event| value.set(event.checked()),
                }
            }
        }
    }
}

#[component]
pub fn TextControl(label: &'static str, value: Signal<String>) -> Element {
    rsx! {
        label { class: "flex w-full flex-col gap-1 text-sm font-medium",
            span { "{label}" }
            input {
                class: "h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal shadow-xs outline-none focus-visible:ring-1 focus-visible:ring-ring",
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
        label { class: "flex w-full flex-col gap-1 text-sm font-medium",
            span { "{label}" }
            select {
                class: "h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal shadow-xs outline-none focus-visible:ring-1 focus-visible:ring-ring",
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
