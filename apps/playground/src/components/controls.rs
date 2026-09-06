//! Small bound controls used on component pages to edit a prop live and
//! re-render the demo immediately, Storybook-style.
//!
//! Each widget's interactive element is an installed registry component
//! (`Switch`, `Input`, `NativeSelect`) rather than a hand-styled raw HTML
//! input, so the playground's own tooling demonstrates the components it
//! distributes. The caption stays a wrapping native `<label>` (implicit
//! association): the installed `Label` requires `html_for`, and minting a
//! unique id per control would collide when several generated panels on one
//! page bind the same prop name (e.g. two "Disabled" controls).
//!
//! The five public signatures here are frozen: generated
//! `<Component>Controls` panels (`cargo xtask playground-controls sync`)
//! emit calls against exactly these prop lists.

use dioxus::prelude::*;

use crate::components::ui;

/// `NativeSelect`'s root wrapper is deliberately `w-fit`; inside the
/// controls grid every widget should stretch. Composing around the existing
/// API (spec: no registry change for playground convenience): a child
/// selector on the wrapping label out-specifies `w-fit` without touching
/// the registry component.
const CONTROL_LABEL_CLASS: &str = "flex w-full flex-col gap-1 text-sm font-medium [&>div]:w-full";

#[component]
pub fn BoolControl(label: &'static str, value: Signal<bool>) -> Element {
    rsx! {
        label { class: CONTROL_LABEL_CLASS,
            span { "{label}" }
            span { class: "flex h-9 w-full items-center",
                ui::Switch {
                    checked: ReadSignal::from(Signal::new(Some(value()))),
                    on_checked_change: move |checked| value.set(checked),
                    aria_label: label,
                }
            }
        }
    }
}

#[component]
pub fn TextControl(label: &'static str, value: Signal<String>) -> Element {
    rsx! {
        label { class: CONTROL_LABEL_CLASS,
            span { "{label}" }
            ui::Input {
                value: Some(value()),
                oninput: move |event: FormEvent| value.set(event.value()),
            }
        }
    }
}

/// A closed-enum control: renders an installed [`ui::NativeSelect`] over
/// `options` (display label, value) two-way bound to `value`, matching
/// `BoolControl`/`TextControl`'s calling convention.
#[component]
pub fn SelectControl<T: Clone + PartialEq + 'static>(
    label: &'static str,
    value: Signal<T>,
    options: &'static [(&'static str, T)],
) -> Element {
    let selected_index = options
        .iter()
        .position(|(_, option)| *option == value())
        .unwrap_or(0);
    rsx! {
        label { class: CONTROL_LABEL_CLASS,
            span { "{label}" }
            ui::NativeSelect {
                value: Some(selected_index.to_string()),
                oninput: move |event: FormEvent| {
                    if let Ok(index) = event.value().parse::<usize>()
                        && let Some((_, option)) = options.get(index)
                    {
                        value.set(option.clone());
                    }
                },
                for (index , (option_label , _)) in options.iter().enumerate() {
                    ui::NativeSelectOption { value: "{index}", "{option_label}" }
                }
            }
        }
    }
}

/// A bound installed [`ui::Input`] of `type="number"`, for numeric props
/// (e.g. `Slider`'s `min`/`max`/`step`, `Progress`'s `value`/`max`).
#[component]
pub fn NumberControl(
    label: &'static str,
    value: Signal<f64>,
    #[props(default)] min: Option<f64>,
    #[props(default)] max: Option<f64>,
    #[props(default)] step: Option<f64>,
) -> Element {
    rsx! {
        label { class: CONTROL_LABEL_CLASS,
            span { "{label}" }
            ui::Input {
                r#type: "number",
                value: Some(value().to_string()),
                min: min.map(|value| value.to_string()),
                max: max.map(|value| value.to_string()),
                step: step.map(|value| value.to_string()),
                oninput: move |event: FormEvent| {
                    if let Ok(parsed) = event.value().parse::<f64>() {
                        value.set(parsed);
                    }
                },
            }
        }
    }
}

/// A tri-state select (`Uncontrolled` / `On` / `Off`) for the
/// `Option<ReadSignal<Option<bool>>>`-shaped optional-controlled idiom, so a
/// demo can show a component's own uncontrolled default alongside forcing it
/// open or closed.
#[component]
pub fn OptionalBoolControl(label: &'static str, value: Signal<Option<bool>>) -> Element {
    let current = match value() {
        Some(true) => "on",
        Some(false) => "off",
        None => "uncontrolled",
    };
    rsx! {
        label { class: CONTROL_LABEL_CLASS,
            span { "{label}" }
            ui::NativeSelect {
                value: Some(current.to_string()),
                oninput: move |event: FormEvent| {
                    value
                        .set(
                            match event.value().as_str() {
                                "on" => Some(true),
                                "off" => Some(false),
                                _ => None,
                            },
                        );
                },
                ui::NativeSelectOption { value: "uncontrolled", "Uncontrolled" }
                ui::NativeSelectOption { value: "on", "On" }
                ui::NativeSelectOption { value: "off", "Off" }
            }
        }
    }
}
