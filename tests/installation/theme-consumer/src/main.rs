use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        main { class: "flex items-center gap-4 p-6",
            components::ui::ModeToggle {}
            components::ui::ThemeSwitcher {}
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
