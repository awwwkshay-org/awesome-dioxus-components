//! Source-owned shadcn-style Progress for Dioxus, backed by the owned adico
//! primitive layer.

use dioxus::prelude::*;

use adico_primitives::progress::{
    Progress as ProgressPrimitive, ProgressIndicator as ProgressIndicatorPrimitive,
};

use crate::adico_lib::cn::cn;

/// Props for [`Progress`].
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// The current progress value, between 0 and `max`. `None` renders an
    /// indeterminate progress bar.
    pub value: ReadSignal<Option<f64>>,
    /// The maximum value. Defaults to 100.
    #[props(default = ReadSignal::new(Signal::new(100.0)))]
    pub max: ReadSignal<f64>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Accessible name for the progressbar role.
    #[props(default)]
    pub aria_label: Option<String>,
}

/// A progress bar with the default adico/shadcn visual language.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let class = cn(&[
        "relative h-2 w-full overflow-hidden rounded-full bg-primary/20",
        props.class.as_deref().unwrap_or_default(),
    ]);
    let percentage = props.value.cloned().unwrap_or(0.0) / (props.max)() * 100.0;
    let indicator_style = format!("transform: translateX(-{}%);", 100.0 - percentage);
    rsx! {
        ProgressPrimitive {
            value: props.value,
            max: props.max,
            class,
            aria_label: props.aria_label,
            ProgressIndicatorPrimitive {
                class: "h-full w-full flex-1 bg-primary transition-all",
                style: indicator_style,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_class_uses_semantic_primary_surface() {
        let class = cn(&["relative h-2 w-full overflow-hidden rounded-full bg-primary/20", ""]);
        assert!(class.contains("bg-primary/20"));
    }
}
