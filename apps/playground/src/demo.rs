use dioxus::prelude::*;

/// A labeled section wrapping one component's live demo, so a page reads as
/// "component name -> what it looks like" rather than an undifferentiated
/// blob of markup. `controls`, when present, renders as a bound prop-editor
/// strip above the demo (Storybook-style) so changing a prop re-renders the
/// component immediately with no page reload.
#[component]
pub fn Demo(name: &'static str, controls: Option<Element>, children: Element) -> Element {
    rsx! {
        section { class: "space-y-3",
            h1 { class: "text-2xl font-bold", "{name}" }
            if let Some(controls) = controls {
                div { class: "flex flex-wrap gap-4 rounded-lg border bg-muted/50 p-4", {controls} }
            }
            div { class: "rounded-lg border p-6", {children} }
        }
    }
}
