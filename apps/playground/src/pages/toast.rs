use dioxus::prelude::*;

use crate::components;
use crate::components::demo::Demo;

#[component]
pub fn ToastPage() -> Element {
    rsx! {
        Demo {
            name: "Toast",
            components::ui::ToastProvider {
                ToastButton {}
            }
        }
    }
}

#[component]
fn ToastButton() -> Element {
    let toast_api = components::ui::toast::use_toast();
    rsx! {
        button {
            class: "inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow-xs hover:bg-primary/90",
            onclick: move |_| {
                toast_api.info("Saved".to_string(), components::ui::toast::ToastOptions::new());
            },
            "Show toast"
        }
    }
}
