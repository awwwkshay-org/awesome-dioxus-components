use dioxus::prelude::*;

/// A fixed preview workspace: its title stays at the top, the live component
/// occupies the upper three quarters, and its prop controls occupy the lower
/// quarter. The controls pane alone scrolls when it contains more content.
#[component]
pub fn Demo(name: &'static str, controls: Option<Element>, children: Element) -> Element {
    rsx! {
        section { class: "flex h-full min-h-0 w-full flex-col text-foreground",
            h1 { class: "shrink-0 text-2xl font-bold", "{name}" }
            div { class: "mt-3 grid min-h-0 flex-1 gap-3", style: "grid-template-rows: minmax(0, 3fr) minmax(0, 1fr);",
                div {
                    class: "relative z-20 grid min-h-0 place-items-center overflow-auto rounded-lg border border-border bg-muted/20 p-6",
                    style: "background-image: linear-gradient(hsl(var(--border) / 0.08) 1px, transparent 1px), linear-gradient(90deg, hsl(var(--border) / 0.08) 1px, transparent 1px); background-size: 2rem 2rem;",
                    div {
                        class: "text-card-foreground",
                        style: "display: flex; width: min(100%, 36rem); height: 100%; align-items: center; justify-content: center; align-self: center; justify-self: center;",
                        {children}
                    }
                }
                div { class: "relative z-10 flex min-h-0 flex-col rounded-lg border border-border bg-card text-card-foreground",
                    div { class: "shrink-0 border-b border-border px-4 py-2",
                        h2 { class: "text-sm font-semibold", "Component controls" }
                    }
                    div { class: "min-h-0 flex-1 overflow-y-auto p-4",
                        if let Some(controls) = controls {
                            div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3", {controls} }
                        } else {
                            p { class: "text-sm text-muted-foreground", "This component has no live props in the playground yet." }
                        }
                    }
                }
            }
        }
    }
}
