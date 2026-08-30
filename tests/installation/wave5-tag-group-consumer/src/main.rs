use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        components::ui::TagGroupMulti::<&'static str> {
            default_values: vec!["bug"],
            components::ui::TagGroupLabel { "Labels" }
            components::ui::TagList {
                components::ui::TagOption::<&'static str> { index: 0usize, value: "bug",
                    "bug"
                    components::ui::TagRemoveButton { "x" }
                }
                components::ui::TagOption::<&'static str> { index: 1usize, value: "feature",
                    "feature"
                    components::ui::TagRemoveButton { "x" }
                }
                components::ui::TagOption::<&'static str> { index: 2usize, value: "wontfix", disabled: true,
                    "wontfix"
                }
            }
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
