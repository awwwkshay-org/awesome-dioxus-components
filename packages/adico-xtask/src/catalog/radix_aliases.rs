//! Resolves a shadcn `inherits_from: "<axis>.<component>.<part>"` reference
//! against the already-fetched `statics/catalogs/base-ui.json`, applying a
//! small, explicit, hand-maintained alias table for the component-id/part-id
//! renames observed between Radix and Base UI (design.md's Decisions
//! section). Deliberately not a fifth network-fetched axis -- see
//! design.md's "Radix references resolve downstream" decision.
//!
//! Every alias entry is added only after its target is verified against the
//! committed `base-ui.json` -- a plausible-but-wrong resolution (comparing
//! adico's props against the wrong upstream part) is worse than an
//! unresolved reference, so a component/part pair with no *verified*
//! equivalent stays unresolved with a specific reason instead of guessed.

use super::schema::{CatalogSnapshot, Prop, PropsSource};

/// Component-id aliases observed between Radix and Base UI (radix -> base-ui).
const COMPONENT_ALIASES: &[(&str, &str)] = &[
    ("dropdown-menu", "menu"),
    ("hover-card", "preview-card"),
    ("radio-group", "radio"),
];

/// Part-id aliases scoped to a specific (already component-aliased)
/// component id -- checked before the generic table below, since `content`
/// resolves differently depending on which component it belongs to.
///
/// `radio.root`/`radio.item` are the two entries that fixed a real defect:
/// Radix's `RadioGroup.Root` is the *group* container (Base UI's
/// `radio.group`: `name`, `defaultValue`, `value`, `onValueChange`, ...),
/// while Radix's `RadioGroup.Item` is the individual button (Base UI's
/// `radio.root`). Without both, the generic `component_alias`-then-pass-
/// through-part-id path silently lands `radix.radio-group.root` on
/// `radio.root` -- the wrong part -- instead of `radio.group`.
const COMPONENT_SCOPED_PART_ALIASES: &[(&str, &str, &str)] = &[
    ("dialog", "content", "popup"),
    ("popover", "content", "popup"),
    ("select", "content", "popup"),
    ("tooltip", "content", "popup"),
    ("menu", "content", "popup"),
    ("preview-card", "content", "popup"),
    ("accordion", "content", "panel"),
    ("collapsible", "content", "panel"),
    ("collapsible", "collapsible-content", "panel"),
    ("tabs", "content", "panel"),
    ("tabs", "trigger", "tab"),
    ("radio", "root", "group"),
    ("radio", "item", "root"),
    ("select", "scroll-up-button", "scroll-up-arrow"),
    ("select", "scroll-down-button", "scroll-down-arrow"),
    ("scroll-area", "scroll-area-scrollbar", "scrollbar"),
    ("context-menu", "content", "popup"),
    ("context-menu", "sub", "submenu-root"),
    ("context-menu", "sub-content", "popup"),
    ("menu", "sub", "submenu-root"),
    ("menu", "sub-content", "popup"),
];

/// Part-id aliases that apply regardless of component.
const GENERIC_PART_ALIASES: &[(&str, &str)] =
    &[("overlay", "backdrop"), ("collapsible-trigger", "trigger")];

/// Component/part pairs (post component-alias, pre part-alias) with a known,
/// specific reason they don't resolve -- either a genuine absence in Base
/// UI, or a resolution that would require a composition judgment call
/// rather than a mechanical rename, and is deliberately left unmade. Checked
/// before the generic Base UI lookup so these get their real reason instead
/// of the generic "no resolvable Base UI equivalent" message.
const KNOWN_UNRESOLVED: &[(&str, &str, &str)] = &[
    (
        "alert-dialog",
        "action",
        "Base UI's AlertDialog exposes only a single `close` part; it does not distinguish an affirmative `action` from a `cancel` the way Radix does",
    ),
    (
        "alert-dialog",
        "cancel",
        "Base UI's AlertDialog exposes only a single `close` part; it does not distinguish an affirmative `action` from a `cancel` the way Radix does",
    ),
    (
        "toggle-group",
        "item",
        "Base UI's `toggle-group` component only carries a `root` part; the individual toggle button is a separate `toggle.root` component, not a sub-part of toggle-group -- composing that mapping is a judgment call left unmade",
    ),
    (
        "aspect-ratio",
        "root",
        "Base UI has no aspect-ratio component",
    ),
    ("slot", "root", "Base UI has no slot/merge-props component"),
    ("label", "root", "Base UI has no standalone label component"),
    (
        "popover",
        "anchor",
        "Base UI's popover has no separate anchor part",
    ),
    (
        "navigation-menu",
        "indicator",
        "Base UI's navigation-menu has no separate indicator part",
    ),
];

fn known_unresolved_reason(component: &str, part: &str) -> Option<&'static str> {
    KNOWN_UNRESOLVED
        .iter()
        .find(|(c, p, _)| *c == component && *p == part)
        .map(|(_, _, reason)| *reason)
}

/// `menubar.root` resolves directly and legitimately (Base UI's `menubar`
/// component's own root props -- `loopFocus`, `modal`, `disabled`,
/// `orientation` -- are a genuine match for Radix's `Menubar.Root`); only
/// its sub-parts are the composition judgment being deliberately left
/// unmade, since Base UI's `menubar` carries no part besides `root` and its
/// menu items actually come from the separate `menu` component.
fn is_menubar_composition_judgment(component: &str, part: &str) -> bool {
    component == "menubar" && part != "root"
}

fn alias_component(component: &str) -> &str {
    for &(radix, base_ui) in COMPONENT_ALIASES {
        if radix == component {
            return base_ui;
        }
    }
    component
}

fn alias_part<'a>(component: &str, part: &'a str) -> &'a str {
    for &(scoped_component, radix_part, base_ui) in COMPONENT_SCOPED_PART_ALIASES {
        if scoped_component == component && radix_part == part {
            return base_ui;
        }
    }
    for &(radix, base_ui) in GENERIC_PART_ALIASES {
        if radix == part {
            return base_ui;
        }
    }
    part
}

/// The result of resolving an `inherits_from` reference: either the resolved
/// upstream part's explicit props, or an explicit record of why it couldn't
/// be resolved (never silently treated as "zero missing props").
#[derive(Debug, Clone)]
pub enum ResolvedProps {
    Resolved { props: Vec<Prop> },
    Unresolved { reason: String },
}

/// Resolves a dotted `"<axis>.<component>.<part>"` reference against
/// `base_ui`. When `axis == "radix"`, the component alias is applied first,
/// then the part alias (component-scoped aliases checked before generic
/// ones), before the lookup. A third-party axis with no catalog of its own
/// (`vaul`, `cmdk`, `@shadcn/react/*`) or a shadcn-internal self-reference
/// (`@/registry/*`) is reported unresolved with a reason naming that fact,
/// rather than being looked up against Base UI at all. Any other axis
/// (already `"base-ui"`, or a future direct match) is looked up with no
/// aliasing.
pub fn resolve_inherits_from(reference: &str, base_ui: &CatalogSnapshot) -> ResolvedProps {
    let mut segments = reference.splitn(3, '.');
    let (axis, component, part) = match (segments.next(), segments.next(), segments.next()) {
        (Some(axis), Some(component), Some(part)) => (axis, component, part),
        _ => {
            return ResolvedProps::Unresolved {
                reason: format!("malformed inherits_from reference: {reference}"),
            };
        }
    };

    if axis == "vaul" || axis == "cmdk" || axis.starts_with("@shadcn/react/") {
        return ResolvedProps::Unresolved {
            reason: format!(
                "`{axis}` is a third-party dependency with no catalog axis in this project"
            ),
        };
    }
    if axis.starts_with("@/registry/") {
        return ResolvedProps::Unresolved {
            reason: format!(
                "shadcn-internal self-reference to `{axis}` -- resolving shadcn's own components against each other is out of scope for this change"
            ),
        };
    }

    let component = if axis == "radix" {
        alias_component(component)
    } else {
        component
    };
    let part = if axis == "radix" {
        alias_part(component, part)
    } else {
        part
    };

    if axis == "radix" {
        if is_menubar_composition_judgment(component, part) {
            return ResolvedProps::Unresolved {
                reason: "Base UI's `menubar` exposes only a `root` part; its menu items come from the separate `menu` component, and mapping menubar's sub-parts onto menu's is a composition judgment left unmade".to_string(),
            };
        }
        if let Some(reason) = known_unresolved_reason(component, part) {
            return ResolvedProps::Unresolved {
                reason: reason.to_string(),
            };
        }
    }

    let Some(entry) = base_ui.entries.iter().find(|entry| entry.id == component) else {
        return ResolvedProps::Unresolved {
            reason: format!("no resolvable Base UI equivalent for component `{component}`"),
        };
    };
    let Some(part_entry) = entry.parts.iter().find(|candidate| candidate.id == part) else {
        return ResolvedProps::Unresolved {
            reason: format!("no resolvable Base UI equivalent for part `{component}.{part}`"),
        };
    };

    match &part_entry.props_source {
        PropsSource::Explicit { props } => ResolvedProps::Resolved {
            props: props.clone(),
        },
        _ => ResolvedProps::Unresolved {
            reason: format!("Base UI entry `{component}.{part}` has no explicit props recorded"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::schema::{CatalogEntry, PartEntry};

    fn base_ui_fixture() -> CatalogSnapshot {
        CatalogSnapshot {
            axis: "base-ui".to_string(),
            source: "https://base-ui.com".to_string(),
            revision: "test".to_string(),
            refreshed_at: "2026-09-05".to_string(),
            entries: vec![
                CatalogEntry {
                    id: "dialog".to_string(),
                    name: "Dialog".to_string(),
                    parts: vec![PartEntry {
                        id: "trigger".to_string(),
                        composition: Vec::new(),
                        props_source: PropsSource::Explicit {
                            props: vec![Prop {
                                name: "disabled".to_string(),
                                type_name: "boolean".to_string(),
                                default: None,
                                description: None,
                            }],
                        },
                    }],
                },
                CatalogEntry {
                    id: "menu".to_string(),
                    name: "Menu".to_string(),
                    parts: vec![PartEntry {
                        id: "popup".to_string(),
                        composition: Vec::new(),
                        props_source: PropsSource::Explicit {
                            props: vec![Prop {
                                name: "side".to_string(),
                                type_name: "string".to_string(),
                                default: None,
                                description: None,
                            }],
                        },
                    }],
                },
                CatalogEntry {
                    id: "radio".to_string(),
                    name: "Radio".to_string(),
                    parts: vec![
                        PartEntry {
                            id: "group".to_string(),
                            composition: Vec::new(),
                            props_source: PropsSource::Explicit {
                                props: vec![Prop {
                                    name: "onValueChange".to_string(),
                                    type_name: "function".to_string(),
                                    default: None,
                                    description: None,
                                }],
                            },
                        },
                        PartEntry {
                            id: "root".to_string(),
                            composition: Vec::new(),
                            props_source: PropsSource::Explicit {
                                props: vec![Prop {
                                    name: "value".to_string(),
                                    type_name: "string".to_string(),
                                    default: None,
                                    description: None,
                                }],
                            },
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn direct_match_resolves_with_no_alias() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("radix.dialog.trigger", &base_ui);
        match resolved {
            ResolvedProps::Resolved { props } => {
                assert_eq!(props.len(), 1);
                assert_eq!(props[0].name, "disabled");
            }
            other => panic!("expected resolved, got {other:?}"),
        }
    }

    #[test]
    fn component_and_part_alias_both_apply() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("radix.dropdown-menu.content", &base_ui);
        match resolved {
            ResolvedProps::Resolved { props } => {
                assert_eq!(props.len(), 1);
                assert_eq!(props[0].name, "side");
            }
            other => panic!("expected resolved, got {other:?}"),
        }
    }

    #[test]
    fn non_radix_axis_is_looked_up_directly() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("base-ui.dialog.trigger", &base_ui);
        assert!(matches!(resolved, ResolvedProps::Resolved { .. }));
    }

    #[test]
    fn genuine_non_match_is_unresolved_with_reason() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("radix.command.root", &base_ui);
        match resolved {
            ResolvedProps::Unresolved { reason } => {
                assert!(reason.contains("command"), "reason was: {reason}");
            }
            other => panic!("expected unresolved, got {other:?}"),
        }
    }

    /// The defect this change fixes: `radix.radio-group.root` (Radix's
    /// group *container*) must resolve to Base UI's `radio.group`, not
    /// `radio.root` (Base UI's individual radio button) -- a
    /// plausible-but-wrong resolution that would compare adico's
    /// `RadioGroup` against the wrong prop list.
    #[test]
    fn radio_group_root_resolves_to_radio_group_not_radio_root() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("radix.radio-group.root", &base_ui);
        match resolved {
            ResolvedProps::Resolved { props } => {
                assert_eq!(props.len(), 1);
                assert_eq!(props[0].name, "onValueChange");
            }
            other => panic!("expected resolved to radio.group, got {other:?}"),
        }
    }

    /// The other half of the same defect: `radix.radio-group.item` (Radix's
    /// individual radio button) must resolve to Base UI's `radio.root`.
    #[test]
    fn radio_group_item_resolves_to_radio_root() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("radix.radio-group.item", &base_ui);
        match resolved {
            ResolvedProps::Resolved { props } => {
                assert_eq!(props.len(), 1);
                assert_eq!(props[0].name, "value");
            }
            other => panic!("expected resolved to radio.root, got {other:?}"),
        }
    }

    #[test]
    fn third_party_axis_gets_its_own_reason() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("vaul.drawer.content", &base_ui);
        match resolved {
            ResolvedProps::Unresolved { reason } => {
                assert!(
                    reason.contains("third-party") && reason.contains("vaul"),
                    "reason was: {reason}"
                );
            }
            other => panic!("expected unresolved, got {other:?}"),
        }
    }

    #[test]
    fn shadcn_internal_self_reference_gets_its_own_reason() {
        let base_ui = base_ui_fixture();
        let resolved =
            resolve_inherits_from("@/registry/new-york-v4/ui/button.button.root", &base_ui);
        match resolved {
            ResolvedProps::Unresolved { reason } => {
                assert!(reason.contains("self-reference"), "reason was: {reason}");
            }
            other => panic!("expected unresolved, got {other:?}"),
        }
    }

    #[test]
    fn menubar_reference_gets_composition_judgment_reason() {
        let base_ui = base_ui_fixture();
        let resolved = resolve_inherits_from("radix.menubar.trigger", &base_ui);
        match resolved {
            ResolvedProps::Unresolved { reason } => {
                assert!(reason.contains("menubar"), "reason was: {reason}");
            }
            other => panic!("expected unresolved, got {other:?}"),
        }
    }

    fn repo_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("packages/adico-xtask has two parent directories")
            .to_path_buf()
    }

    fn load_snapshot(relative_path: &str) -> CatalogSnapshot {
        let path = repo_root().join(relative_path);
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        serde_json::from_str(&contents)
            .unwrap_or_else(|error| panic!("{} is invalid: {error}", path.display()))
    }

    /// Task 2.2's real-data verification: runs the actual resolver (not a
    /// simulation) over the committed `statics/catalogs/shadcn.json` and
    /// `statics/catalogs/base-ui.json`, asserting the exact measured split
    /// (88 resolved / 19 unresolved of 107 `radix.*` references, after the
    /// `cva` fix moved 2 of the original 109 references -- `tabs.list` and
    /// `toggle.root` -- to `explicit`, since both those components also
    /// declare `cva()` variant groups) and that the 19 unresolved
    /// references are exactly the named residual list from design.md's
    /// Decisions section -- so a future catalog refresh or
    /// table edit that silently changes this split fails a real test
    /// instead of only being caught by hand-inspection.
    #[test]
    fn real_shadcn_radix_references_match_the_measured_90_19_split() {
        let shadcn = load_snapshot("statics/catalogs/shadcn.json");
        let base_ui = load_snapshot("statics/catalogs/base-ui.json");

        let mut resolved_count = 0usize;
        let mut unresolved_refs: Vec<String> = Vec::new();
        for entry in &shadcn.entries {
            for part in &entry.parts {
                if let PropsSource::InheritsFrom { reference } = &part.props_source
                    && reference.starts_with("radix.")
                {
                    match resolve_inherits_from(reference, &base_ui) {
                        ResolvedProps::Resolved { .. } => resolved_count += 1,
                        ResolvedProps::Unresolved { .. } => unresolved_refs.push(reference.clone()),
                    }
                }
            }
        }
        unresolved_refs.sort();
        unresolved_refs.dedup();

        let expected_unresolved: Vec<&str> = vec![
            "radix.alert-dialog.action",
            "radix.alert-dialog.cancel",
            "radix.aspect-ratio.root",
            "radix.label.root",
            "radix.menubar.checkbox-item",
            "radix.menubar.content",
            "radix.menubar.group",
            "radix.menubar.menu",
            "radix.menubar.portal",
            "radix.menubar.radio-group",
            "radix.menubar.radio-item",
            "radix.menubar.separator",
            "radix.menubar.sub",
            "radix.menubar.sub-content",
            "radix.menubar.trigger",
            "radix.navigation-menu.indicator",
            "radix.popover.anchor",
            "radix.slot.root",
            "radix.toggle-group.item",
        ];

        assert_eq!(
            unresolved_refs, expected_unresolved,
            "unresolved radix.* references drifted from design.md's named residual list"
        );
        assert_eq!(
            resolved_count, 88,
            "resolved radix.* reference count drifted from design.md's measured 88/107"
        );
    }

    /// The other half of task 2.2: every non-`radix` `inherits_from`
    /// reference in the real committed shadcn catalog gets a specific
    /// third-party/self-reference reason, never the generic "no resolvable
    /// Base UI equivalent" message a naive fallback would produce. 30, not
    /// the original 32: the `cva` fix moved 2 of the original 32 (a
    /// `button` self-reference on `attachment`'s trigger and a `tooltip`
    /// self-reference on `sidebar`'s menu-button) to `explicit`, since both
    /// those components also declare `cva()` variant groups.
    #[test]
    fn real_non_radix_references_get_specific_reasons_not_a_base_ui_message() {
        let shadcn = load_snapshot("statics/catalogs/shadcn.json");
        let base_ui = load_snapshot("statics/catalogs/base-ui.json");

        let mut checked = 0usize;
        for entry in &shadcn.entries {
            for part in &entry.parts {
                if let PropsSource::InheritsFrom { reference } = &part.props_source
                    && !reference.starts_with("radix.")
                {
                    checked += 1;
                    match resolve_inherits_from(reference, &base_ui) {
                        ResolvedProps::Resolved { .. } => {
                            // `base-ui.*` self-references (already the base-ui
                            // axis) are expected to resolve directly.
                            assert!(
                                reference.starts_with("base-ui."),
                                "{reference} resolved but is not a base-ui.* self-reference"
                            );
                        }
                        ResolvedProps::Unresolved { reason } => {
                            assert!(
                                !reason.contains("no resolvable Base UI equivalent"),
                                "{reference} got the generic Base UI message: {reason}"
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(
            checked, 30,
            "expected 30 non-radix inherits_from references"
        );
    }
}
