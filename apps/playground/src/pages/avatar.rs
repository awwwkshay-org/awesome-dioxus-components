use dioxus::prelude::*;

use crate::components;
use crate::components::controls::SelectControl;
use crate::components::demo::Demo;

#[component]
pub fn AvatarPage() -> Element {
    let size = use_signal(|| components::ui::AvatarSize::Default);
    rsx! {
        Demo {
            name: "Avatar",
            controls: rsx! {
                SelectControl {
                    label: "Size",
                    value: size,
                    options: crate::generated::controls::AVATAR_SIZE_OPTIONS,
                }
            },
            components::ui::Avatar { size: size(),
                components::ui::AvatarFallback { "AB" }
            }
        }
    }
}
