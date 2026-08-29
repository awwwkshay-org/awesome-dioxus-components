//! Source-owned shadcn-style Switch for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::switch::{Switch as SwitchPrimitive, SwitchThumb as SwitchThumbPrimitive};

use crate::adico_lib::cn::cn;

/// Props for [`Switch`].
#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    /// The controlled checked state of the switch.
    #[props(default)]
    pub checked: ReadSignal<Option<bool>>,
    /// The default checked state when uncontrolled.
    #[props(default = false)]
    pub default_checked: bool,
    /// Whether the switch is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// The name attribute for form submission.
    #[props(default)]
    pub name: ReadSignal<String>,
    /// Callback fired when the checked state changes.
    #[props(default)]
    pub on_checked_change: Callback<bool>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Accessible label, since the switch itself has no visible text.
    #[props(default)]
    pub aria_label: Option<String>,
}

/// A toggle switch with the default adico/shadcn visual language.
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let class = cn(&[
        "peer inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs outline-none transition-colors \
         focus-visible:ring-2 focus-visible:ring-ring/50 \
         disabled:cursor-not-allowed disabled:opacity-50 \
         data-[state=checked]:bg-primary data-[state=unchecked]:bg-input",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        SwitchPrimitive {
            checked: props.checked,
            default_checked: props.default_checked,
            disabled: props.disabled,
            name: props.name,
            on_checked_change: props.on_checked_change,
            class,
            aria_label: props.aria_label,
            SwitchThumbPrimitive {
                class: "pointer-events-none block size-4 rounded-full bg-background shadow-lg ring-0 transition-transform \
                        data-[state=checked]:translate-x-[calc(100%-2px)] data-[state=unchecked]:translate-x-0",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_class_switches_semantic_surface_by_state() {
        let class = cn(&[
            "peer inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border border-transparent shadow-xs outline-none transition-colors \
             focus-visible:ring-2 focus-visible:ring-ring/50 \
             disabled:cursor-not-allowed disabled:opacity-50 \
             data-[state=checked]:bg-primary data-[state=unchecked]:bg-input",
            "",
        ]);
        assert!(class.contains("data-[state=checked]:bg-primary"));
        assert!(class.contains("data-[state=unchecked]:bg-input"));
    }
}
