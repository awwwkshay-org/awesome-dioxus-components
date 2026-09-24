//! Shared registry manifest and prop-introspection data for the docs route
//! tree. Ported unchanged from `apps/docs`'s pre-merge `main.rs` -- see its
//! module doc, preserved here: a read-only renderer over
//! `registry/registry.json` (embedded at compile time), not a second
//! live-demo tree, since `/playground` already owns live component demos.
//! Each component's usage snippet, accessibility note, and keyboard note
//! come directly from the registry manifest's `documentation` field (see
//! `packages/adico-registry-core`'s `DocumentationMetadata`), so this page
//! can never drift from what's actually shipped: editing a component's
//! documentation means editing `registry.json`, not this file.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

const REGISTRY_JSON: &str = include_str!("../../../../../registry/registry.json");

/// The same introspected prop data `packages/adico-xtask/src/
/// playground_controls.rs`/`prop_parity.rs` already extract from registry
/// source, emitted once by `cargo xtask component-props sync` as one
/// committed JSON file and consumed here as a second, generic consumer --
/// proving the metadata isn't playground-coupled. Not every component has
/// an entry: a bare `pub use` re-export of a primitive (e.g. `AspectRatio`)
/// has no props visible to introspection, a pre-existing, documented
/// limitation of the underlying tool, not something specific to this table.
const COMPONENT_PROPS_JSON: &str = include_str!("../../../../../statics/component_props.json");

#[derive(Deserialize, Clone, PartialEq)]
pub struct ComponentPropField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub default: Option<String>,
}

pub fn component_props() -> &'static BTreeMap<String, BTreeMap<String, Vec<ComponentPropField>>> {
    static PROPS: OnceLock<BTreeMap<String, BTreeMap<String, Vec<ComponentPropField>>>> =
        OnceLock::new();
    PROPS.get_or_init(|| {
        serde_json::from_str(COMPONENT_PROPS_JSON).expect("component_props.json is valid JSON")
    })
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct DocumentationData {
    #[serde(default, rename = "compositionNote")]
    pub composition_note: Option<String>,
    #[serde(default)]
    pub usage: Option<String>,
    #[serde(default)]
    pub accessibility: Option<String>,
    #[serde(default)]
    pub keyboard: Option<String>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub struct RegistryItemData {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub description: String,
    #[serde(default)]
    pub documentation: Option<DocumentationData>,
}

#[derive(Deserialize)]
struct RegistryManifestData {
    items: Vec<RegistryItemData>,
}

pub fn registry_items() -> &'static [RegistryItemData] {
    static ITEMS: OnceLock<Vec<RegistryItemData>> = OnceLock::new();
    ITEMS.get_or_init(|| {
        let manifest: RegistryManifestData =
            serde_json::from_str(REGISTRY_JSON).expect("registry.json is valid JSON");
        manifest.items
    })
}

pub fn ui_components() -> impl Iterator<Item = &'static RegistryItemData> {
    registry_items()
        .iter()
        .filter(|item| item.item_type == "registry:ui")
}

pub fn find_component(name: &str) -> Option<&'static RegistryItemData> {
    registry_items().iter().find(|item| item.name == name)
}
