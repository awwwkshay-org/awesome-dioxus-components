use dioxus::prelude::*;

/// A labeled section wrapping one component's live demo, so a page reads as
/// "component name -> what it looks like" rather than an undifferentiated
/// blob of markup.
#[component]
pub fn Demo(name: &'static str, children: Element) -> Element {
    rsx! {
        section { class: "space-y-3",
            h1 { class: "text-2xl font-bold", "{name}" }
            div { class: "rounded-lg border p-6", {children} }
        }
    }
}
