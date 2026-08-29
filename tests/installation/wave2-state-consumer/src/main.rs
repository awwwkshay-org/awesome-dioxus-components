use dioxus::prelude::*;

fn app() -> Element {
    let mut checked = use_signal(|| components::ui::CheckboxState::Unchecked);
    let mut switched = use_signal(|| false);
    let mut pressed = use_signal(|| false);
    let mut open = use_signal(|| true);

    rsx! {
        components::ui::Avatar {
            components::ui::AvatarImage { src: "https://example.invalid/broken.png", alt: "Broken avatar" }
            components::ui::AvatarFallback { "AB" }
        }
        components::ui::Checkbox {
            checked: checked(),
            on_checked_change: move |value| checked.set(value),
            aria_label: "Accept terms",
        }
        components::ui::Switch {
            checked: switched(),
            on_checked_change: move |value| switched.set(value),
            aria_label: "Enable notifications",
        }
        components::ui::Toggle {
            pressed: pressed(),
            on_pressed_change: move |value| pressed.set(value),
            "Bold"
        }
        components::ui::Collapsible {
            open: open(),
            on_open_change: move |value| open.set(value),
            components::ui::CollapsibleTrigger { "Toggle section" }
            components::ui::CollapsibleContent { "Collapsible content" }
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
