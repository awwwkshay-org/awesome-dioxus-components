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
/// value) two-way bound to `value`, matching `BoolControl`/`TextControl`'s
/// calling convention.
#[component]
pub fn SelectControl<T: Clone + PartialEq + 'static>(
    label: &'static str,
    value: Signal<T>,
    options: &'static [(&'static str, T)],
) -> Element {
    rsx! {
        label { class: "flex w-full flex-col gap-1 text-sm font-medium",
            span { "{label}" }
            select {
                class: "h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal shadow-xs outline-none focus-visible:ring-1 focus-visible:ring-ring",
                onchange: move |event| {
                    if let Ok(index) = event.value().parse::<usize>()
                        && let Some((_, option)) = options.get(index)
                    {
                        value.set(option.clone());
                    }
                },
                for (index , (option_label , option)) in options.iter().enumerate() {
                    option {
                        value: "{index}",
                        selected: *option == value(),
                        "{option_label}"
                    }
                }
            }
        }
    }
}

/// A bound `<input type="number">`, for numeric props (e.g. `Slider`'s
/// `min`/`max`/`step`, `Progress`'s `value`/`max`).
#[component]
pub fn NumberControl(
    label: &'static str,
    value: Signal<f64>,
    #[props(default)] min: Option<f64>,
    #[props(default)] max: Option<f64>,
    #[props(default)] step: Option<f64>,
) -> Element {
    rsx! {
        label { class: "flex w-full flex-col gap-1 text-sm font-medium",
            span { "{label}" }
            input {
                class: "h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal shadow-xs outline-none focus-visible:ring-1 focus-visible:ring-ring",
                r#type: "number",
                value: "{value}",
                min: min.map(|value| value.to_string()),
                max: max.map(|value| value.to_string()),
                step: step.map(|value| value.to_string()),
                oninput: move |event| {
                    if let Ok(parsed) = event.value().parse::<f64>() {
                        value.set(parsed);
                    }
                },
            }
        }
    }
}

/// A tri-state `<select>` (`Uncontrolled` / `On` / `Off`) for the
/// `Option<ReadSignal<Option<bool>>>`-shaped optional-controlled idiom, so a
/// demo can show a component's own uncontrolled default alongside forcing it
/// open or closed.
#[component]
pub fn OptionalBoolControl(label: &'static str, value: Signal<Option<bool>>) -> Element {
    rsx! {
        label { class: "flex w-full flex-col gap-1 text-sm font-medium",
            span { "{label}" }
            select {
                class: "h-9 w-full rounded-md border border-input bg-background px-3 text-sm font-normal shadow-xs outline-none focus-visible:ring-1 focus-visible:ring-ring",
                onchange: move |event| {
                    value
                        .set(
                            match event.value().as_str() {
                                "on" => Some(true),
                                "off" => Some(false),
                                _ => None,
                            },
                        );
                },
                option { value: "uncontrolled", selected: value().is_none(), "Uncontrolled" }
                option { value: "on", selected: value() == Some(true), "On" }
                option { value: "off", selected: value() == Some(false), "Off" }
            }
        }
    }
}
