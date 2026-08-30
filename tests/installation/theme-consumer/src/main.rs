use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        main { class: "flex items-center gap-4 p-6",
            div { id: "mode-toggle-demo", components::ui::ModeToggle {} }
            div { id: "theme-switcher-demo", components::ui::ThemeSwitcher {} }
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
