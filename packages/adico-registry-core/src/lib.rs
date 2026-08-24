//! Versioned registry-domain types for adico.
//!
//! This crate owns registry parsing and planning semantics. The CLI owns
//! project discovery, presentation, and file mutation.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// The registry format supported by this version of adico.
pub const REGISTRY_FORMAT_VERSION: u32 = 1;

/// A stable, named registry namespace such as `@adico` or `@awwwkshay`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RegistryNamespace(String);

impl RegistryNamespace {
    /// The official registry namespace embedded by adico.
    pub const OFFICIAL: &'static str = "@adico";

    /// Returns this namespace as configured in registry metadata.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegistryNamespace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for RegistryNamespace {
    type Err = RegistryError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let suffix = value
            .strip_prefix('@')
            .filter(|suffix| !suffix.is_empty())
            .ok_or_else(|| RegistryError::InvalidNamespace(value.to_string()))?;
        if suffix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            Ok(Self(value.to_string()))
        } else {
            Err(RegistryError::InvalidNamespace(value.to_string()))
        }
    }
}

impl<'de> Deserialize<'de> for RegistryNamespace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

/// An item address supplied by a user or another registry item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryAddress {
    /// An item resolved through the consuming project's default registry.
    Bare(String),
    /// An item resolved through a specific configured namespace.
    Namespaced {
        /// The source namespace.
        namespace: RegistryNamespace,
        /// The stable item name.
        item: String,
    },
}

impl RegistryAddress {
    /// Parses a bare item name or a namespaced `@namespace/item` address.
    pub fn parse(value: &str) -> Result<Self, RegistryError> {
        if let Some((namespace, item)) = value.rsplit_once('/')
            && namespace.starts_with('@')
        {
            return Ok(Self::Namespaced {
                namespace: namespace.parse()?,
                item: validate_item_name(item)?,
            });
        }
        Ok(Self::Bare(validate_item_name(value)?))
    }
}

/// A complete versioned registry manifest.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryManifest {
    /// The registry schema format.
    pub format_version: u32,
    /// Stable source identity used in namespaced item addresses.
    pub namespace: RegistryNamespace,
    /// Human-readable registry name.
    pub name: String,
    /// Optional human-readable registry description.
    #[serde(default)]
    pub description: Option<String>,
    /// CLI/runtime compatibility requirements.
    pub compatibility: RegistryCompatibility,
    /// All source-installable registry items.
    pub items: Vec<RegistryItem>,
}

impl RegistryManifest {
    /// Validates invariants that JSON deserialization alone cannot express.
    pub fn validate(&self) -> Result<(), RegistryError> {
        if self.format_version != REGISTRY_FORMAT_VERSION {
            return Err(RegistryError::UnsupportedFormat {
                actual: self.format_version,
                supported: REGISTRY_FORMAT_VERSION,
            });
        }
        let mut names = BTreeSet::new();
        for item in &self.items {
            validate_item_name(&item.name)?;
            if !names.insert(&item.name) {
                return Err(RegistryError::DuplicateItem(item.name.clone()));
            }
            if item.files.is_empty() {
                return Err(RegistryError::ItemHasNoFiles(item.name.clone()));
            }
        }
        Ok(())
    }
}

/// A registry item that can install source into a consumer project.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryItem {
    /// Stable item name, e.g. `dialog`.
    pub name: String,
    /// Extensible item category.
    #[serde(rename = "type")]
    pub item_type: RegistryItemType,
    /// Short purpose shown by CLI and documentation tooling.
    pub description: String,
    /// Source files and their logical installation intent.
    pub files: Vec<RegistryFile>,
    /// Dependencies on items in this or explicitly named registries.
    #[serde(default)]
    pub registry_dependencies: Vec<String>,
    /// Ordinary Cargo dependencies required by installed source.
    #[serde(default)]
    pub cargo_dependencies: Vec<CargoDependency>,
    /// Theme, CSS, and source utility requirements.
    #[serde(default)]
    pub style: StyleRequirements,
    /// Rust module exports managed for the installed source.
    #[serde(default)]
    pub module_exports: Vec<ModuleExport>,
    /// Optional documentation metadata.
    #[serde(default)]
    pub documentation: Option<DocumentationMetadata>,
    /// Item-specific compatibility narrower than the registry's requirement.
    #[serde(default)]
    pub compatibility: Option<RegistryCompatibility>,
    /// Optional source-provenance reference for registry/docs tooling.
    #[serde(default)]
    pub provenance: Option<ProvenanceReference>,
}

/// Categories supported by the v1 schema, including reserved future categories.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum RegistryItemType {
    /// A conventional styled UI component.
    #[serde(rename = "registry:ui")]
    Ui,
    /// A compositional component.
    #[serde(rename = "registry:component")]
    Component,
    /// A Rust hook.
    #[serde(rename = "registry:hook")]
    Hook,
    /// A supporting Rust utility/library.
    #[serde(rename = "registry:lib")]
    Lib,
    /// A reusable composed block.
    #[serde(rename = "registry:block")]
    Block,
    /// A page-level template.
    #[serde(rename = "registry:page")]
    Page,
    /// A theme contribution.
    #[serde(rename = "registry:theme")]
    Theme,
    /// An arbitrary source/configuration file.
    #[serde(rename = "registry:file")]
    File,
    /// A multi-file project template.
    #[serde(rename = "registry:template")]
    Template,
}

/// A source file installed as part of an item.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryFile {
    /// Path relative to the registry source root.
    pub source: String,
    /// Logical consumer root rather than a hard-coded source path.
    pub target_root: TargetRoot,
    /// Path relative to the resolved logical root.
    pub target: String,
    /// SHA-256 checksum of the authored source content.
    pub checksum: String,
}

/// Consumer destination roots resolved through `components.json`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetRoot {
    /// The UI-component destination.
    Ui,
    /// The general components destination.
    Components,
    /// The source-installed utility destination.
    Lib,
    /// The hook destination.
    Hooks,
    /// The configured CSS entry destination.
    Css,
}

/// A Cargo dependency installed or reconciled with the consumer manifest.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CargoDependency {
    /// Manifest key used by installed source.
    #[serde(rename = "crate")]
    pub crate_name: String,
    /// Optional crates.io package when the manifest key is an alias.
    #[serde(default)]
    pub package: Option<String>,
    /// Required semver range.
    pub version: String,
    /// Enabled package features.
    #[serde(default)]
    pub features: Vec<String>,
    /// Whether Cargo default features remain enabled.
    #[serde(default = "default_true")]
    pub default_features: bool,
    /// Optional target predicate.
    #[serde(default)]
    pub target: Option<String>,
}

/// CSS/theme requirements for an item.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StyleRequirements {
    /// Whether the standard semantic token set is required.
    #[serde(default)]
    pub semantic_tokens: bool,
    /// Whether the standard radius token is required.
    #[serde(default)]
    pub radius_token: bool,
    /// Source-installed utility items required for class composition.
    #[serde(default)]
    pub utilities: Vec<String>,
}

/// A Rust module registration required by an installed file.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModuleExport {
    /// Logical root containing the managed `mod.rs` file.
    pub target_root: TargetRoot,
    /// Module name registered inside the managed region.
    pub module: String,
    /// Whether the module is re-exported from the same managed region.
    #[serde(default = "default_true")]
    pub reexport: bool,
}

/// Optional documentation metadata retained with the registry item.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentationMetadata {
    /// Documentation slug.
    pub slug: String,
    /// Optional composition/API note.
    #[serde(default)]
    pub composition_note: Option<String>,
}

/// Compatibility requirements declared by a registry or individual item.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryCompatibility {
    /// Compatible adico CLI semver range.
    pub cli: String,
    /// Compatible adico-primitives semver range, when required.
    #[serde(default)]
    pub runtime: Option<String>,
}

/// Source provenance retained by the registry model.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProvenanceReference {
    /// Project-relative provenance record path or company-managed equivalent.
    pub record: String,
    /// Immutable origin revision when the item ports upstream source.
    #[serde(default)]
    pub revision: Option<String>,
}

/// A named source available to a consumer configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RegistrySource {
    /// The official source embedded in the adico CLI distribution.
    Embedded,
    /// A local organization registry directory or manifest path.
    Local {
        /// Consumer-configured local source path.
        path: String,
    },
    /// A static HTTPS manifest hosted by an organization.
    Https {
        /// Absolute HTTPS manifest URL.
        url: String,
    },
}

/// Named registry source selection, shared by future `components.json` parsing.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistrySourceConfiguration {
    /// Source definitions by stable namespace.
    pub registries: BTreeMap<RegistryNamespace, RegistrySource>,
    /// Namespace used to resolve bare component names.
    pub default_registry: RegistryNamespace,
}

/// Registry parsing and invariant errors.
#[derive(Debug, Error, PartialEq)]
pub enum RegistryError {
    /// A namespace does not follow the stable `@lowercase-name` grammar.
    #[error(
        "invalid registry namespace {0:?}; use @ followed by lowercase letters, digits, or hyphens"
    )]
    InvalidNamespace(String),
    /// An item name is not portable across registries.
    #[error("invalid registry item name {0:?}; use lowercase letters, digits, or hyphens")]
    InvalidItemName(String),
    /// The manifest format cannot be interpreted by this CLI/core version.
    #[error("unsupported registry format {actual}; this build supports format {supported}")]
    UnsupportedFormat {
        /// Registry format encountered.
        actual: u32,
        /// Registry format supported by this build.
        supported: u32,
    },
    /// An item name appears more than once in one manifest.
    #[error("duplicate registry item {0:?}")]
    DuplicateItem(String),
    /// An item cannot install source without source files.
    #[error("registry item {0:?} does not declare source files")]
    ItemHasNoFiles(String),
}

fn validate_item_name(value: &str) -> Result<String, RegistryError> {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Ok(value.to_string())
    } else {
        Err(RegistryError::InvalidItemName(value.to_string()))
    }
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_and_company_manifests_deserialize() {
        let official: RegistryManifest = serde_json::from_str(include_str!(
            "../../../tests/compile/registry/official-valid.json"
        ))
        .expect("official fixture should deserialize");
        official
            .validate()
            .expect("official fixture should validate");

        let company: RegistryManifest = serde_json::from_str(include_str!(
            "../../../tests/compile/registry/awwwkshay-valid.json"
        ))
        .expect("company fixture should deserialize");
        company.validate().expect("company fixture should validate");
        assert_eq!(company.namespace.as_str(), "@awwwkshay");
    }

    #[test]
    fn invalid_item_type_is_rejected_during_deserialization() {
        let result = serde_json::from_str::<RegistryManifest>(include_str!(
            "../../../tests/compile/registry/invalid-item-type.json"
        ));
        assert!(result.is_err());
    }

    #[test]
    fn addresses_preserve_explicit_source_selection() {
        assert_eq!(
            RegistryAddress::parse("@adico/dialog").expect("valid address"),
            RegistryAddress::Namespaced {
                namespace: "@adico".parse().expect("valid namespace"),
                item: "dialog".to_string(),
            }
        );
        assert_eq!(
            RegistryAddress::parse("dialog").expect("valid bare item"),
            RegistryAddress::Bare("dialog".to_string())
        );
    }
}
