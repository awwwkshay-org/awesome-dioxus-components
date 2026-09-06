use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

/// Offline "photo" slides: a CSS-gradient backdrop, a large glyph, and a
/// caption per slide — no network images, per the playground's offline
/// constraint.
const SLIDES: &[(&str, &str, &str)] = &[
    (
        "linear-gradient(135deg, hsl(210 70% 55%), hsl(250 70% 40%))",
        "🏔️",
        "Alpine Ridge",
    ),
    (
        "linear-gradient(135deg, hsl(20 80% 55%), hsl(340 70% 45%))",
        "🌅",
        "Harbor Sunset",
    ),
    (
        "linear-gradient(135deg, hsl(140 60% 45%), hsl(180 60% 35%))",
        "🌲",
        "Old Growth",
    ),
    (
        "linear-gradient(135deg, hsl(45 90% 55%), hsl(25 85% 45%))",
        "🏜️",
        "Dune Sea",
    ),
    (
        "linear-gradient(135deg, hsl(270 60% 50%), hsl(300 60% 35%))",
        "🌌",
        "Night Sky",
    ),
];

#[component]
pub fn CarouselPage() -> Element {
    rsx! {
        Demo {
            name: "Carousel",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground",
                    "Drag a slide with the mouse, focus the track and use the arrow keys, or use the paging buttons."
                }
            },
            components::ui::Carousel { class: "w-full max-w-xs",
                components::ui::CarouselContent {
                    for (gradient , glyph , caption) in SLIDES.iter().copied() {
                        components::ui::CarouselItem { key: "{caption}",
                            div {
                                class: "relative flex aspect-square flex-col items-center justify-center overflow-hidden rounded-md text-white",
                                style: "background-image: {gradient};",
                                span { class: "text-6xl drop-shadow", "{glyph}" }
                                span { class: "absolute bottom-3 left-4 text-sm font-medium drop-shadow", "{caption}" }
                            }
                        }
                    }
                }
                components::ui::CarouselPrevious {}
                components::ui::CarouselNext {}
            }
        }
    }
}
