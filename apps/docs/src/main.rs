//! adico documentation site: a read-only renderer over `registry/registry.json`
//! (embedded at compile time), not a second live-demo tree -- `apps/playground`
//! already owns live component demos. Each component's usage snippet,
//! accessibility note, and keyboard note come directly from the registry
//! manifest's `documentation` field (see `packages/adico-registry-core`'s
//! `DocumentationMetadata`), so this page can never drift from what's
//! actually shipped: editing a component's documentation means editing
//! `registry.json`, not this file.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use dioxus::prelude::*;
use serde::Deserialize;

const REGISTRY_JSON: &str = include_str!("../../../registry/registry.json");

/// The same introspected prop data `packages/adico-xtask/src/
/// playground_controls.rs`/`prop_parity.rs` already extract from registry
/// source, emitted once by `cargo xtask component-props sync` as one
/// committed JSON file and consumed here as a second, generic consumer --
/// proving the metadata isn't playground-coupled. See `design.md`'s D5
/// decision. Not every component has an entry: a bare `pub use` re-export
/// of a primitive (e.g. `AspectRatio`) has no props visible to
/// introspection, a pre-existing, documented limitation of the underlying
/// tool, not something specific to this table.
const COMPONENT_PROPS_JSON: &str = include_str!("../../../statics/component_props.json");

#[derive(Deserialize, Clone, PartialEq)]
struct ComponentPropField {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    #[serde(default)]
    default: Option<String>,
}

fn component_props() -> &'static BTreeMap<String, BTreeMap<String, Vec<ComponentPropField>>> {
    static PROPS: OnceLock<BTreeMap<String, BTreeMap<String, Vec<ComponentPropField>>>> =
        OnceLock::new();
    PROPS.get_or_init(|| {
        serde_json::from_str(COMPONENT_PROPS_JSON).expect("component_props.json is valid JSON")
    })
}

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
table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
th, td { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 1px solid #27272a; }
th { color: #a1a1aa; font-weight: 500; }
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
            {
                let item_key = component.name.replace('-', "_");
                match component_props().get(&item_key) {
                    Some(components) if !components.is_empty() => rsx! {
                        for (component_name , fields) in components {
                            section {
                                h2 { "{component_name} props" }
                                table {
                                    thead {
                                        tr {
                                            th { "Name" }
                                            th { "Type" }
                                            th { "Default" }
                                        }
                                    }
                                    tbody {
                                        for field in fields {
                                            tr {
                                                td { code { "{field.name}" } }
                                                td { code { "{field.type_name}" } }
                                                td {
                                                    code {
                                                        {field.default.as_deref().unwrap_or("—")}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        section {
                            h2 { "Props" }
                            p { "No props are visible to introspection for this component -- its root is a bare re-export of a primitive with no local wrapper." }
                        }
                    },
                }
            }
        }
    }
}
