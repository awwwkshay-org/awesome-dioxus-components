//! Shared catalog schema for `cargo xtask catalog fetch <axis>`. One shape
//! for all axes (shadcn, Base UI, dioxus-components, dioxus-primitives),
//! replacing the two divergent ad hoc snapshot shapes `upstreams/` used.
//!
//! `PropsSource` is the load-bearing type: the four axes are not symmetric
//! in what prop data is even obtainable (see design.md), so a prop set is
//! either a real list (`explicit`), a note that it passes another axis's
//! component through unmodified (`inherits_from`), or absent (`unavailable`)
//! -- never a silently empty list standing in for "we don't know".

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSnapshot {
    pub axis: String,
    pub source: String,
    pub revision: String,
    pub refreshed_at: String,
    pub entries: Vec<CatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    /// Stable identifier, kebab-case (e.g. `dialog`).
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<PartEntry>,
}

/// One exported part of a component/primitive (e.g. Dialog's `Root`,
/// `Trigger`, `Popup`). A leaf primitive with no sub-parts still gets one
/// part named `root`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartEntry {
    pub id: String,
    /// The other primitives/parts (possibly on a different axis) this part
    /// is built from. Upstream-side composition only -- adico's own
    /// composition is derived locally from `registry/ui/*.rs` and never
    /// stored here (see spec's composition requirement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub composition: Vec<CompositionRef>,
    pub props_source: PropsSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositionRef {
    pub axis: String,
    pub component: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PropsSource {
    Explicit {
        props: Vec<Prop>,
    },
    /// A dotted reference to another part this one passes props through to
    /// unmodified, e.g. `base-ui.dialog.trigger`. Kept as a string rather
    /// than a resolved link because the target may be on an axis this
    /// snapshot doesn't itself carry (see design.md's Radix open question).
    InheritsFrom {
        reference: String,
    },
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prop {
    pub name: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let snapshot = CatalogSnapshot {
            axis: "base-ui".to_string(),
            source: "https://base-ui.com/react/components".to_string(),
            revision: "2026-08-31T00:00:00Z".to_string(),
            refreshed_at: "2026-08-31".to_string(),
            entries: vec![CatalogEntry {
                id: "dialog".to_string(),
                name: "Dialog".to_string(),
                parts: vec![
                    PartEntry {
                        id: "root".to_string(),
                        composition: vec![],
                        props_source: PropsSource::Explicit {
                            props: vec![Prop {
                                name: "open".to_string(),
                                type_name: "boolean".to_string(),
                                default: Some("false".to_string()),
                                description: Some("Whether the dialog is open.".to_string()),
                            }],
                        },
                    },
                    PartEntry {
                        id: "trigger".to_string(),
                        composition: vec![CompositionRef {
                            axis: "base-ui".to_string(),
                            component: "dialog".to_string(),
                            part: Some("root".to_string()),
                        }],
                        props_source: PropsSource::InheritsFrom {
                            reference: "base-ui.dialog.root".to_string(),
                        },
                    },
                ],
            }],
        };

        let json = serde_json::to_string_pretty(&snapshot).expect("serialize");
        let parsed: CatalogSnapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.axis, snapshot.axis);
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].parts.len(), 2);
        assert!(matches!(
            parsed.entries[0].parts[0].props_source,
            PropsSource::Explicit { .. }
        ));
        assert!(matches!(
            parsed.entries[0].parts[1].props_source,
            PropsSource::InheritsFrom { .. }
        ));
    }

    #[test]
    fn unavailable_props_source_round_trips() {
        let part = PartEntry {
            id: "root".to_string(),
            composition: vec![],
            props_source: PropsSource::Unavailable,
        };
        let json = serde_json::to_string(&part).expect("serialize");
        let parsed: PartEntry = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed.props_source, PropsSource::Unavailable));
    }
}
