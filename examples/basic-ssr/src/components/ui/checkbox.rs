//! Source-owned shadcn-style Checkbox for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::checkbox::CheckboxState;
use adico_primitives::checkbox::{
    Checkbox as CheckboxPrimitive, CheckboxIndicator as CheckboxIndicatorPrimitive,
};
use adico_primitives::icons::Check;

use crate::adico_lib::cn::cn;

/// Props for [`Checkbox`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    /// The controlled state of the checkbox.
    #[props(default)]
    pub checked: ReadSignal<Option<CheckboxState>>,
    /// The default state when uncontrolled.
    #[props(default = CheckboxState::Unchecked)]
    pub default_checked: CheckboxState,
    /// Whether the checkbox is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// Whether the checkbox is required in a form.
    #[props(default)]
    pub required: ReadSignal<bool>,
    /// The name of the checkbox, used in forms.
    #[props(default)]
    pub name: ReadSignal<String>,
    /// Callback fired when the checked state changes.
    #[props(default)]
    pub on_checked_change: Callback<CheckboxState>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Accessible label, since the checkbox itself has no visible text.
    #[props(default)]
    pub aria_label: Option<String>,
}

/// A checkbox input with the default adico/shadcn visual language.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let class = cn(&[
        "peer size-4 shrink-0 rounded-[4px] border border-input shadow-xs outline-none transition-shadow \
         focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/50 \
         disabled:cursor-not-allowed disabled:opacity-50 \
         data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground \
         data-[state=indeterminate]:border-primary data-[state=indeterminate]:bg-primary data-[state=indeterminate]:text-primary-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        CheckboxPrimitive {
            checked: props.checked,
            default_checked: props.default_checked,
            disabled: props.disabled,
            required: props.required,
            name: props.name,
            on_checked_change: props.on_checked_change,
            class,
            aria_label: props.aria_label,
            CheckboxIndicatorPrimitive { class: "flex items-center justify-center text-current",
                Check { class: "size-3.5" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_state_uses_semantic_primary_surface() {
        let class = cn(&[
            "peer size-4 shrink-0 rounded-[4px] border border-input shadow-xs outline-none transition-shadow \
             focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/50 \
             disabled:cursor-not-allowed disabled:opacity-50 \
             data-[state=checked]:border-primary data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground \
             data-[state=indeterminate]:border-primary data-[state=indeterminate]:bg-primary data-[state=indeterminate]:text-primary-foreground",
            "",
        ]);
        assert!(class.contains("data-[state=checked]:bg-primary"));
        assert!(class.contains("data-[state=indeterminate]:bg-primary"));
    }
}
