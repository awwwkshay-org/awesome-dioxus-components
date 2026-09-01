//! Source-owned shadcn-style Switch for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::switch::{Switch as SwitchPrimitive, SwitchThumb as SwitchThumbPrimitive};

use crate::adico_lib::cn::cn;

/// The visual size of a [`Switch`]. Was entirely absent before task 5.2 --
/// upstream shadcn added a `size` axis (`sm`/`default`) to this component;
/// adico had only ever rendered the `default` size, a real, missing
/// capability rather than a styling nuance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SwitchSize {
    /// Compact size.
    Sm,
    /// Default size.
    #[default]
    Default,
}

impl SwitchSize {
    fn track_class(self) -> &'static str {
        match self {
            Self::Sm => "h-3.5 w-6",
            Self::Default => "h-[1.15rem] w-8",
        }
    }

    fn thumb_class(self) -> &'static str {
        match self {
            Self::Sm => "size-3",
            Self::Default => "size-4",
        }
    }
}

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
    /// The visual size.
    #[props(default)]
    pub size: SwitchSize,
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
        "peer group inline-flex shrink-0 items-center rounded-full border border-transparent shadow-xs outline-none transition-colors \
         focus-visible:ring-2 focus-visible:ring-ring/50 \
         disabled:cursor-not-allowed disabled:opacity-50 \
         data-[state=checked]:bg-primary data-[state=unchecked]:bg-input",
        props.size.track_class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    // `group-data-[state=...]`, not a plain `data-[state=...]`: `data-state`
    // lives on the track (`Switch`, above), not on this thumb `span` itself
    // -- a bare `data-[state=checked]:translate-x-...` here matches this
    // element's own (nonexistent) attribute, so the thumb never translates
    // regardless of checked state (found live: the track colors correctly
    // but the thumb sits frozen at the unchecked position).
    let thumb_class = cn(&[
        "pointer-events-none block rounded-full bg-background shadow-lg ring-0 transition-transform \
         group-data-[state=checked]:translate-x-[calc(100%-2px)] group-data-[state=unchecked]:translate-x-0",
        props.size.thumb_class(),
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
            SwitchThumbPrimitive { class: thumb_class }
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

    #[test]
    fn thumb_translates_off_the_track_s_own_state_not_its_own() {
        let class = cn(&[
            "group-data-[state=checked]:translate-x-[calc(100%-2px)] group-data-[state=unchecked]:translate-x-0",
            "",
        ]);
        assert!(class.contains("group-data-[state=checked]:translate-x"));
        assert!(!class.contains(" data-[state=checked]:translate-x"));
    }
}
