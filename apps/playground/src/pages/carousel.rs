use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn CarouselPage() -> Element {
    rsx! {
        Demo {
            name: "Carousel",
            controls: rsx! {
                p { class: "self-end pb-2 text-sm text-muted-foreground", "Focus the track and use the arrow keys, or use the paging buttons." }
            },
            components::ui::Carousel { class: "w-full max-w-xs",
                components::ui::CarouselContent {
                    for i in 1..=5 {
                        components::ui::CarouselItem {
                            div { class: "flex aspect-square items-center justify-center rounded-md border bg-muted text-4xl font-semibold",
                                "{i}"
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
