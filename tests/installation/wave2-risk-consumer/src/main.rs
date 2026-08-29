use dioxus::prelude::*;

fn app() -> Element {
    let mut alert_open = use_signal(|| false);

    rsx! {
        components::ui::ScrollArea {
            style: "height: 6em; width: 12em; border: 1px solid black;",
            div {
                for i in 1..=20 {
                    p { "Scrollable item {i}" }
                }
            }
        }
        components::ui::AlertDialog {
            open: alert_open(),
            on_open_change: move |value| alert_open.set(value),
            components::ui::AlertDialogTrigger { "Delete item" }
            components::ui::AlertDialogOverlay {}
            components::ui::AlertDialogContent {
                components::ui::AlertDialogHeader {
                    components::ui::AlertDialogTitle { "Delete item" }
                    components::ui::AlertDialogDescription { "Are you sure? This cannot be undone." }
                }
                components::ui::AlertDialogActions {
                    components::ui::AlertDialogCancel { "Cancel" }
                    components::ui::AlertDialogAction { "Delete" }
                }
            }
        }
        components::ui::ToastProvider {
            ToastButton {}
        }
        components::ui::Slider { label: "Volume", default_value: 50.0,
            components::ui::SliderTrack {
                components::ui::SliderRange {}
                components::ui::SliderThumb {}
            }
        }
    }
}

#[component]
fn ToastButton() -> Element {
    let toast_api = components::ui::toast::use_toast();
    rsx! {
        button {
            onclick: move |_| {
                toast_api.info("Saved".to_string(), components::ui::toast::ToastOptions::new());
            },
            "Show toast"
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
