use dioxus::prelude::*;

fn app() -> Element {
    rsx! {
        components::ui::Toolbar { aria_label: "Text formatting",
            components::ui::ToolbarButton { index: 0usize, "Bold" }
            components::ui::ToolbarSeparator {}
            components::ui::ToolbarButton { index: 1usize, "Italic" }
        }
        components::ui::VirtualList {
            count: 200usize,
            estimate_size: |_idx| 32,
            style: "height: 300px; overflow-y: auto; border: 1px solid #ccc;",
            render_item: move |idx: usize| rsx! {
                div { key: "{idx}", "Row {idx}" }
            },
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
