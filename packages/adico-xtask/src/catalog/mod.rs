//! `cargo xtask catalog fetch <axis>`: the sole network-touching command in
//! adico-xtask. Every axis is a registered [`AxisDef`] -- adding a fifth
//! upstream later means adding a fetcher module and one entry in [`AXES`],
//! not touching `primitive_compat.rs`/`component_compat.rs`, which only ever
//! filter [`AXES`] by [`AxisKind`].

mod base_ui;
mod case;
mod dioxus_components;
mod dioxus_primitives;
mod dioxus_shared;
pub mod schema;
mod shadcn;

pub use schema::CatalogSnapshot;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AxisKind {
    /// Feeds `primitive-compat` (adico-primitives' upstream comparisons).
    Primitive,
    /// Feeds `component-compat` (registry/ui's upstream comparisons).
    Component,
}

pub struct AxisDef {
    pub id: &'static str,
    pub kind: AxisKind,
    /// `revision`: an explicit pin (`--revision <sha>`) for the two Dioxus
    /// axes; ignored by axes with no meaningful revision override.
    pub fetch: fn(revision: Option<&str>) -> Result<CatalogSnapshot, String>,
}

pub const AXES: &[AxisDef] = &[
    AxisDef {
        id: "base-ui",
        kind: AxisKind::Primitive,
        fetch: base_ui::fetch,
    },
    AxisDef {
        id: "dioxus-primitives",
        kind: AxisKind::Primitive,
        fetch: dioxus_primitives::fetch,
    },
    AxisDef {
        id: "shadcn",
        kind: AxisKind::Component,
        fetch: shadcn::fetch,
    },
    AxisDef {
        id: "dioxus-components",
        kind: AxisKind::Component,
        fetch: dioxus_components::fetch,
    },
];

pub fn find(id: &str) -> Option<&'static AxisDef> {
    AXES.iter().find(|axis| axis.id == id)
}

pub fn axes_of_kind(kind: AxisKind) -> impl Iterator<Item = &'static AxisDef> {
    AXES.iter().filter(move |axis| axis.kind == kind)
}

pub fn statics_dir(root: &std::path::Path) -> std::path::PathBuf {
    root.join("statics/catalogs")
}

pub fn statics_path(root: &std::path::Path, axis_id: &str) -> std::path::PathBuf {
    statics_dir(root).join(format!("{axis_id}.json"))
}

/// Reads and parses a committed `statics/catalogs/<axis>.json` snapshot.
/// Used by `primitive-compat`/`component-compat`, which must never fetch.
pub fn read_snapshot(root: &std::path::Path, axis_id: &str) -> Result<CatalogSnapshot, String> {
    let path = statics_path(root, axis_id);
    let contents = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("{} is invalid: {error}", path.display()))
}

pub fn usage_lines() -> String {
    AXES.iter()
        .map(|axis| {
            let kind = match axis.kind {
                AxisKind::Primitive => "primitive",
                AxisKind::Component => "component",
            };
            format!("    {} ({kind})", axis.id)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axes_are_unique_and_registered() {
        let mut ids: Vec<&str> = AXES.iter().map(|axis| axis.id).collect();
        ids.sort();
        let mut deduped = ids.clone();
        deduped.dedup();
        assert_eq!(ids, deduped, "AXES must not register the same id twice");
        assert_eq!(
            ids,
            vec![
                "base-ui",
                "dioxus-components",
                "dioxus-primitives",
                "shadcn",
            ]
        );
    }

    #[test]
    fn kinds_match_expectations() {
        assert_eq!(find("base-ui").unwrap().kind, AxisKind::Primitive);
        assert_eq!(find("dioxus-primitives").unwrap().kind, AxisKind::Primitive);
        assert_eq!(find("shadcn").unwrap().kind, AxisKind::Component);
        assert_eq!(find("dioxus-components").unwrap().kind, AxisKind::Component);
    }
}
