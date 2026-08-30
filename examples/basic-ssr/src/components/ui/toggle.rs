//! Source-owned shadcn-style Toggle for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::toggle::Toggle as TogglePrimitive;

use crate::adico_lib::cn::cn;

/// The visual size of a [`Toggle`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToggleSize {
    /// Compact size.
    Sm,
    /// Default size.
    #[default]
    Default,
    /// Larger size.
    Lg,
}

impl ToggleSize {
    fn class(self) -> &'static str {
        match self {
            Self::Sm => "h-8 px-1.5 min-w-8",
            Self::Default => "h-9 px-2 min-w-9",
            Self::Lg => "h-10 px-2.5 min-w-10",
        }
    }
}

/// Props for [`Toggle`].
#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    /// The controlled pressed state of the toggle.
    #[props(default)]
    pub pressed: ReadSignal<Option<bool>>,
    /// The default pressed state when uncontrolled.
    #[props(default)]
    pub default_pressed: bool,
    /// Whether the toggle is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,
    /// The visual size.
    #[props(default)]
    pub size: ToggleSize,
    /// Callback fired when the pressed state changes.
    #[props(default)]
    pub on_pressed_change: Callback<bool>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed toggle content.
    pub children: Element,
}

/// A two-state pressable button with the default adico/shadcn visual
/// language.
#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let class = cn(&[
        "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium outline-none transition-colors \
         hover:bg-muted hover:text-muted-foreground \
         focus-visible:ring-2 focus-visible:ring-ring/50 \
         disabled:pointer-events-none disabled:opacity-50 \
         data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
        props.size.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        TogglePrimitive {
            pressed: props.pressed,
            default_pressed: props.default_pressed,
            disabled: props.disabled,
            on_pressed_change: props.on_pressed_change,
            class,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_size_has_a_distinct_class() {
        assert_ne!(ToggleSize::Sm.class(), ToggleSize::Default.class());
        assert_ne!(ToggleSize::Default.class(), ToggleSize::Lg.class());
    }

    #[test]
    fn pressed_state_uses_semantic_accent_surface() {
        let class = cn(&[
            "inline-flex items-center justify-center gap-2 rounded-md text-sm font-medium outline-none transition-colors \
             hover:bg-muted hover:text-muted-foreground \
             focus-visible:ring-2 focus-visible:ring-ring/50 \
             disabled:pointer-events-none disabled:opacity-50 \
             data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
            "",
        ]);
        assert!(class.contains("data-[state=on]:bg-accent"));
    }
}
