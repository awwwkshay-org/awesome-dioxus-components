use dioxus::prelude::*;

use crate::adico_lib::variants::Tone;
use crate::components;
use crate::components::controls::{ControlGroup, SelectControl, TextControl};
use crate::components::demo::Demo;
use crate::generated::controls::{BadgeControls, BadgeDemoState};

/// `color: Tone` is generator-skipped (see `button.rs`'s own note) --
/// rendering `Badge` directly here (instead of the generated `BadgePreview`,
/// which has no `color` param) so this control has a visible effect.
const COLOR_OPTIONS: &[(&str, Tone)] = &[
    ("Default", Tone::Default),
    ("Success", Tone::Success),
    ("Warning", Tone::Warning),
    ("Error", Tone::Error),
    ("Info", Tone::Info),
];

#[component]
pub fn BadgePage() -> Element {
    let state = use_signal(BadgeDemoState::default);
    let color = use_signal(|| Tone::Default);
    let label = use_signal(|| "New".to_string());
    rsx! {
        Demo {
            name: "Badge",
            controls: rsx! {
                BadgeControls { state }
                ControlGroup { part: "Badge",
                    SelectControl { label: "Color", value: color, options: COLOR_OPTIONS }
                }
                TextControl { label: "Content", value: label }
            },
            components::ui::Badge { variant: state().variant, color: color(), "{label}" }
        }
    }
}
