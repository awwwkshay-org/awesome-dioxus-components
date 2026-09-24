use dioxus::prelude::*;

use crate::adico_lib::variants::Tone;
use crate::components;
use crate::components::controls::{ControlGroup, SelectControl};
use crate::components::demo::Demo;
use crate::generated::controls::{TagOptionControls, TagOptionDemoState};

/// `color: Tone` is generator-skipped (see `button.rs`'s own note) --
/// hand-rolled here, alongside the generated `TagOptionControls` (which
/// already covers `variant: TagOptionVariant`, a locally-defined enum the
/// generator does recognize). Both props affect only the selected tag's
/// fill (per `TagOption`'s doc comment, the resting look never changes).
const COLOR_OPTIONS: &[(&str, Tone)] = &[
    ("Default", Tone::Default),
    ("Success", Tone::Success),
    ("Warning", Tone::Warning),
    ("Error", Tone::Error),
    ("Info", Tone::Info),
];

#[component]
pub fn TagGroupPage() -> Element {
    let mut value = use_signal(|| Some("rust".to_string()));
    let state = use_signal(TagOptionDemoState::default);
    let color = use_signal(|| Tone::Default);
    rsx! {
        Demo {
            name: "Tag Group",
            controls: rsx! {
                TagOptionControls { state }
                ControlGroup { part: "Tag Option",
                    SelectControl { label: "Color (selected)", value: color, options: COLOR_OPTIONS }
                }
            },
            components::ui::TagGroup::<String> {
                value: ReadSignal::from(value),
                on_value_change: move |v| value.set(v),
                components::ui::TagGroupLabel { "Favorite language" }
                components::ui::TagList {
                    components::ui::TagOption::<String> {
                        value: "rust".to_string(),
                        index: 0usize,
                        variant: state().variant,
                        color: color(),
                        "Rust"
                    }
                    components::ui::TagOption::<String> {
                        value: "dioxus".to_string(),
                        index: 1usize,
                        variant: state().variant,
                        color: color(),
                        "Dioxus"
                    }
                    components::ui::TagOption::<String> {
                        value: "typescript".to_string(),
                        index: 2usize,
                        variant: state().variant,
                        color: color(),
                        "TypeScript"
                    }
                }
            }
        }
    }
}
