//! adico documentation site: a read-only renderer over `registry/registry.json`
//! (embedded at compile time), not a second live-demo tree -- `apps/playground`
//! already owns live component demos. Each component's usage snippet,
//! accessibility note, and keyboard note come directly from the registry
//! manifest's `documentation` field (see `packages/adico-registry-core`'s
//! `DocumentationMetadata`), so this page can never drift from what's
//! actually shipped: editing a component's documentation means editing
//! `registry.json`, not this file.

use std::sync::OnceLock;

use dioxus::prelude::*;
use serde::Deserialize;

const REGISTRY_JSON: &str = include_str!("../../../registry/registry.json");

#[derive(Deserialize, Clone, PartialEq)]
struct DocumentationData {
    #[serde(default, rename = "compositionNote")]
    composition_note: Option<String>,
    #[serde(default)]
    usage: Option<String>,
    #[serde(default)]
    accessibility: Option<String>,
    #[serde(default)]
    keyboard: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq)]
struct RegistryItemData {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
    description: String,
    #[serde(default)]
    documentation: Option<DocumentationData>,
}

#[derive(Deserialize)]
struct RegistryManifestData {
    items: Vec<RegistryItemData>,
}

fn registry_items() -> &'static [RegistryItemData] {
    static ITEMS: OnceLock<Vec<RegistryItemData>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        let manifest: RegistryManifestData =
            serde_json::from_str(REGISTRY_JSON).expect("registry.json is valid JSON");
        manifest.items
    })
}

fn ui_components() -> impl Iterator<Item = &'static RegistryItemData> {
    registry_items()
        .iter()
        .filter(|item| item.item_type == "registry:ui")
}

fn find_component(name: &str) -> Option<&'static RegistryItemData> {
    registry_items().iter().find(|item| item.name == name)
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/components/:name")]
    ComponentPage { name: String },
}

const DOCS_STYLE: &str = "
body { font-family: system-ui, sans-serif; margin: 0; color: #e5e7eb; background: #0a0a0f; }
a { color: #93c5fd; }
main { max-width: 860px; margin: 0 auto; padding: 2rem 1.5rem 4rem; }
header { border-bottom: 1px solid #27272a; padding: 1rem 1.5rem; }
nav ul { columns: 3; list-style: none; padding: 0; }
nav li { margin-bottom: 0.25rem; }
h1 { margin-top: 0; }
section { margin-top: 2rem; }
section h2 { font-size: 0.9rem; text-transform: uppercase; letter-spacing: 0.05em; color: #a1a1aa; margin-bottom: 0.5rem; }
pre { background: #16161d; border: 1px solid #27272a; border-radius: 6px; padding: 1rem; overflow-x: auto; }
code { font-family: ui-monospace, monospace; font-size: 0.85rem; }
.back { display: inline-block; margin-bottom: 1rem; }
";

fn main() {
    #[cfg(feature = "server")]
    dioxus::serve(|| async move { Ok(dioxus::server::router(App)) });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        style { {DOCS_STYLE} }
        Router::<Route> {}
    }
}

#[component]
fn Layout() -> Element {
    rsx! {
        header {
            Link { to: Route::Home {}, "adico documentation" }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let mut components: Vec<&RegistryItemData> = ui_components().collect();
    components.sort_by(|a, b| a.name.cmp(&b.name));
    rsx! {
        main {
            h1 { "Components" }
            p { "Every source-owned registry component, with its real usage, accessibility, and keyboard documentation." }
            nav {
                ul {
                    for component in components {
                        li {
                            Link { to: Route::ComponentPage { name: component.name.clone() }, "{component.name}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComponentPage(name: String) -> Element {
    let Some(component) = find_component(&name) else {
        return rsx! {
            main {
                Link { class: "back", to: Route::Home {}, "← Back to components" }
                h1 { "Not found" }
                p { "No registry component named \"{name}\"." }
            }
        };
    };
    let doc = component.documentation.as_ref();
    rsx! {
        main {
            Link { class: "back", to: Route::Home {}, "← Back to components" }
            h1 { "{component.name}" }
            p { "{component.description}" }
            if let Some(note) = doc.and_then(|d| d.composition_note.as_deref()) {
                section {
                    h2 { "Composition note" }
                    p { "{note}" }
                }
            }
            if let Some(usage) = doc.and_then(|d| d.usage.as_deref()) {
                section {
                    h2 { "Usage" }
                    pre { code { "{usage}" } }
                }
            }
            if let Some(accessibility) = doc.and_then(|d| d.accessibility.as_deref()) {
                section {
                    h2 { "Accessibility" }
                    p { "{accessibility}" }
                }
            }
            if let Some(keyboard) = doc.and_then(|d| d.keyboard.as_deref()) {
                section {
                    h2 { "Keyboard" }
                    p { "{keyboard}" }
                }
            }
        }
    }
}
