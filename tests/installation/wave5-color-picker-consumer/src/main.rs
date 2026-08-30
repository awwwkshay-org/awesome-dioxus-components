use dioxus::prelude::*;
use palette::{encoding, IntoColor};

fn app() -> Element {
    let mut color = use_signal(|| -> palette::Hsv<encoding::Srgb, f64> {
        components::ui::color_picker::Color::new(155, 128, 255)
            .into_format::<f64>()
            .into_color()
    });
    rsx! {
        components::ui::ColorPicker {
            color: color(),
            on_color_change: move |c| color.set(c),
            components::ui::ColorArea {
                components::ui::AreaTrack {
                    components::ui::AreaThumb {
                        components::ui::AreaThumbSaturationInput {}
                        components::ui::AreaThumbValueInput {}
                    }
                }
            }
        }
    }
}

fn main() {
    launch(app);
}

// adico:start
pub mod adico_lib;
pub mod components;
// adico:end
