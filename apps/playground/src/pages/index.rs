use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        h1 { class: "text-2xl font-bold", "adico playground" }
        p { class: "text-sm text-muted-foreground",
            "Every currently migrated component, installed through the real `adico` CLI. Pick one from the list to see its live demo."
        }
    }
}
