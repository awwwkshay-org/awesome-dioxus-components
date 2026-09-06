use dioxus::prelude::*;

use crate::components::demo::Demo;
use crate::generated::controls::{
    ThemeSwitcherControls, ThemeSwitcherDemoState, ThemeSwitcherPreview,
};

#[component]
pub fn ThemeSwitcherPage() -> Element {
    let state = use_signal(|| ThemeSwitcherDemoState { show_label: true });
    rsx! {
        Demo {
            name: "ThemeSwitcher",
            controls: rsx! {
                ThemeSwitcherControls { state }
            },
            ThemeSwitcherPreview { state: state() }
        }
    }
}
