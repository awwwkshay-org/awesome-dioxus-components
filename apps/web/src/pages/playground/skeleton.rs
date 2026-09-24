use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;
use crate::generated::controls::{SkeletonControls, SkeletonDemoState};

#[component]
pub fn SkeletonPage() -> Element {
    // `Skeleton`'s own real default is `decorative: true`; the generator's
    // fixed-default convention always starts a bool field at `false`
    // (design.md's D2), so this overrides the initial demo value to match
    // the real component's own default.
    let state = use_signal(|| SkeletonDemoState {
        decorative: true,
        ..Default::default()
    });
    rsx! {
        Demo {
            name: "Skeleton",
            controls: rsx! {
                SkeletonControls { state }
            },
            components::ui::Skeleton {
                variant: state().variant,
                decorative: state().decorative,
                class: if state().variant == components::ui::SkeletonVariant::Circle { "size-16" } else { "h-4 w-40" },
            }
        }
    }
}
