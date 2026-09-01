//! Versioned registry-domain types for adico.
//!
//! This crate owns registry parsing and planning semantics. The CLI owns
//! project discovery, presentation, and file mutation.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

/// The registry format supported by this version of adico.
pub const REGISTRY_FORMAT_VERSION: u32 = 1;

/// The CLI API version understood by this registry-core release.
pub const ADICO_CLI_VERSION: &str = "0.1.0";

/// The owned primitive API version understood by this registry-core release.
pub const ADICO_PRIMITIVES_VERSION: &str = "0.1.0";

/// The current schema version for a consumer project's `components.json`.
pub const COMPONENTS_CONFIGURATION_VERSION: u32 = 1;

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

impl RegistryItemType {
    /// Returns the stable manifest spelling for this item category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ui => "registry:ui",
            Self::Component => "registry:component",
            Self::Hook => "registry:hook",
            Self::Lib => "registry:lib",
            Self::Block => "registry:block",
            Self::Page => "registry:page",
            Self::Theme => "registry:theme",
            Self::File => "registry:file",
            Self::Template => "registry:template",
        }
    }
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
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl TargetRoot {
    /// Returns the stable manifest spelling for this logical consumer root.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Components => "components",
            Self::Lib => "lib",
            Self::Hooks => "hooks",
            Self::Css => "css",
        }
    }
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
    /// Optional short composition example (an `rsx!` snippet, as text) showing
    /// typical usage of this item's exported components together.
    #[serde(default)]
    pub usage: Option<String>,
    /// Optional accessibility note: ARIA roles/attributes this item's own
    /// source (or the primitive it composes) actually implements. Must be
    /// derived from that source, not general pattern knowledge, so it stays
    /// true for items with only partial or no accessibility treatment.
    #[serde(default)]
    pub accessibility: Option<String>,
    /// Optional keyboard-interaction note: keys this item's own source (or
    /// the primitive it composes) actually handles. Same sourcing rule as
    /// `accessibility` -- state "no component-specific keyboard handling"
    /// plainly where that is the true, verified state, rather than omit it.
    #[serde(default)]
    pub keyboard: Option<String>,
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

/// Consumer-owned, versioned configuration stored in `components.json`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentsConfiguration {
    /// Optional published JSON Schema identifier for editor integrations.
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    /// Versioned configuration shape, independent from registry format version.
    pub version: u32,
    /// Selected registry component visual style.
    pub style: String,
    /// Theme/token conventions used by source-installed components.
    pub theme: ThemeConfiguration,
    /// Consumer-owned source destinations.
    pub paths: ComponentPaths,
    /// The CSS entry point adico may manage through explicit markers.
    pub css: CssConfiguration,
    /// Registry source definitions by stable namespace.
    pub registries: BTreeMap<RegistryNamespace, RegistrySource>,
    /// Namespace used to resolve bare registry addresses.
    pub default_registry: RegistryNamespace,
}

/// Theme choices retained in consumer configuration rather than hidden in CLI state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThemeConfiguration {
    /// Semantic token set installed into the configured CSS entry.
    pub tokens: String,
    /// Dark-mode selection strategy, initially `class`.
    pub dark_mode: String,
}

/// Logical component paths resolved by registry target roots.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComponentPaths {
    /// General component source directory.
    pub components: String,
    /// Styled UI component source directory.
    pub ui: String,
    /// Source-installed utility directory.
    pub lib: String,
    /// Source-installed hook directory.
    pub hooks: String,
}

/// CSS entry configuration for the selected Dioxus/Tailwind workflow.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CssConfiguration {
    /// CSS file relative to the detected consumer project root.
    pub entry: String,
    /// CSS framework integration selected by initialization.
    pub framework: String,
}

impl ComponentsConfiguration {
    /// Parses and validates a v1 `components.json` document.
    ///
    /// No released pre-v1 adico configuration exists. Missing, v0, and newer
    /// versions are rejected explicitly rather than guessed or silently
    /// rewritten; `adico init` will be the only creator of a fresh v1 config.
    pub fn parse(contents: &str) -> Result<Self, RegistryError> {
        let configuration = serde_json::from_str::<Self>(contents).map_err(|error| {
            RegistryError::MalformedComponentsConfiguration {
                message: error.to_string(),
            }
        })?;
        configuration.validate()?;
        Ok(configuration)
    }

    /// Validates path, registry-source, and default-registry invariants.
    pub fn validate(&self) -> Result<(), RegistryError> {
        if self.version != COMPONENTS_CONFIGURATION_VERSION {
            return Err(RegistryError::UnsupportedComponentsConfigurationVersion {
                actual: self.version,
                supported: COMPONENTS_CONFIGURATION_VERSION,
            });
        }
        if self.style.trim().is_empty() {
            return Err(RegistryError::InvalidComponentsConfigurationValue {
                field: "style".to_string(),
                reason: "must not be empty".to_string(),
            });
        }
        if self.theme.tokens.trim().is_empty() || self.theme.dark_mode.trim().is_empty() {
            return Err(RegistryError::InvalidComponentsConfigurationValue {
                field: "theme".to_string(),
                reason: "tokens and darkMode must not be empty".to_string(),
            });
        }
        for (field, path) in [
            ("paths.components", &self.paths.components),
            ("paths.ui", &self.paths.ui),
            ("paths.lib", &self.paths.lib),
            ("paths.hooks", &self.paths.hooks),
            ("css.entry", &self.css.entry),
        ] {
            validate_project_relative_path(field, path)?;
        }
        if self.css.framework.trim().is_empty() {
            return Err(RegistryError::InvalidComponentsConfigurationValue {
                field: "css.framework".to_string(),
                reason: "must not be empty".to_string(),
            });
        }
        if self.registries.is_empty() {
            return Err(RegistryError::InvalidComponentsConfigurationValue {
                field: "registries".to_string(),
                reason: "must define at least one named registry source".to_string(),
            });
        }
        for (namespace, source) in &self.registries {
            validate_configured_registry_source(namespace, source)?;
        }
        if !self.registries.contains_key(&self.default_registry) {
            return Err(RegistryError::UnknownDefaultRegistry {
                namespace: self.default_registry.to_string(),
            });
        }
        Ok(())
    }
}

fn validate_project_relative_path(field: &str, value: &str) -> Result<(), RegistryError> {
    let path = Path::new(value);
    if value.trim().is_empty() || path.is_absolute() {
        return Err(RegistryError::InvalidComponentsConfigurationPath {
            field: field.to_string(),
            path: value.to_string(),
            reason: "must be a non-empty path relative to the consumer project root".to_string(),
        });
    }
    if path.components().any(|component| {
        matches!(
            component,
            Component::RootDir | Component::Prefix(_) | Component::CurDir | Component::ParentDir
        )
    }) {
        return Err(RegistryError::InvalidComponentsConfigurationPath {
            field: field.to_string(),
            path: value.to_string(),
            reason: "must not contain '.', '..', a root, or a platform prefix".to_string(),
        });
    }
    Ok(())
}

fn validate_configured_registry_source(
    namespace: &RegistryNamespace,
    source: &RegistrySource,
) -> Result<(), RegistryError> {
    match source {
        RegistrySource::Embedded => Ok(()),
        RegistrySource::Local { path } => {
            if path.trim().is_empty() || Path::new(path).is_absolute() {
                return Err(RegistryError::InvalidComponentsConfigurationPath {
                    field: format!("registries.{namespace}.path"),
                    path: path.clone(),
                    reason: "must be a non-empty path relative to the consumer project root"
                        .to_string(),
                });
            }
            Ok(())
        }
        RegistrySource::Https { url } => parse_https_url(url).map(|_| ()),
    }
}

/// Immutable content supplied by the CLI for the embedded official registry.
#[derive(Clone, Debug)]
pub struct EmbeddedRegistry {
    manifest: Vec<u8>,
    source_root: PathBuf,
    label: String,
}

impl EmbeddedRegistry {
    /// Creates an embedded registry from its checked-in manifest and source root.
    pub fn new(manifest: impl Into<Vec<u8>>, source_root: impl Into<PathBuf>) -> Self {
        Self {
            manifest: manifest.into(),
            source_root: source_root.into(),
            label: "embedded official registry".to_string(),
        }
    }

    /// Sets the source label included in diagnostics.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

/// Location from which a manifest and its source files were loaded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryLocation {
    /// Registry data included with the adico distribution.
    Embedded {
        /// Human-readable source label.
        label: String,
        /// Checked-in source root installed by the registry.
        source_root: PathBuf,
    },
    /// Registry data read from a consumer-configured filesystem path.
    Local {
        /// Manifest path selected from the configured path.
        manifest_path: PathBuf,
        /// Root used to resolve item source files.
        source_root: PathBuf,
    },
    /// Registry data read from a static HTTPS endpoint.
    Https {
        /// HTTPS manifest URL.
        manifest_url: Url,
        /// HTTPS directory URL used to resolve item source files.
        source_root: Url,
    },
}

impl fmt::Display for RegistryLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Embedded { label, .. } => formatter.write_str(label),
            Self::Local { manifest_path, .. } => write!(formatter, "{}", manifest_path.display()),
            Self::Https { manifest_url, .. } => write!(formatter, "{manifest_url}"),
        }
    }
}

/// A parsed registry plus the location used to resolve its files.
#[derive(Clone, Debug)]
pub struct LoadedRegistry {
    /// Parsed manifest.
    pub manifest: RegistryManifest,
    /// Immutable source location.
    pub location: RegistryLocation,
    manifest_digest: String,
}

impl LoadedRegistry {
    /// Creates an embedded registry from manifest bytes already compiled into
    /// the `adico` executable. Authored source bytes are verified again by the
    /// CLI file reader immediately before they can be installed.
    pub fn from_embedded_manifest(
        manifest_bytes: &[u8],
        label: impl Into<String>,
    ) -> Result<Self, RegistryError> {
        let label = label.into();
        let manifest: RegistryManifest =
            serde_json::from_slice(manifest_bytes).map_err(|error| {
                RegistryError::MalformedManifest {
                    registry_source: label.clone(),
                    message: error.to_string(),
                }
            })?;
        manifest.validate()?;
        validate_compatibility(&manifest.compatibility, &manifest.namespace, "registry")?;
        Ok(Self {
            manifest,
            location: RegistryLocation::Embedded {
                label,
                source_root: PathBuf::new(),
            },
            manifest_digest: sha256_hex(manifest_bytes),
        })
    }

    /// Returns the SHA-256 digest of the exact manifest bytes that were loaded.
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }
}

/// A fully-qualified item identity preserved through registry resolution.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryItemAddress {
    /// Namespace that supplied the item.
    pub namespace: RegistryNamespace,
    /// Stable name declared by that registry.
    pub item: String,
}

impl fmt::Display for RegistryItemAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.namespace, self.item)
    }
}

/// A resolved registry item, including immutable source identity for later
/// installation planning and lock-file creation.
#[derive(Clone, Debug)]
pub struct ResolvedRegistryItem {
    /// Fully-qualified source identity.
    pub address: RegistryItemAddress,
    /// Registry metadata for the resolved source item.
    pub item: RegistryItem,
    /// Source location retained for source-file installation.
    pub location: RegistryLocation,
    /// Digest of the manifest from which the item was resolved.
    pub manifest_digest: String,
    /// Compatibility declared by the registry that supplied the item.
    pub registry_compatibility: RegistryCompatibility,
}

/// A deterministic, dependency-first sequence of source-owned registry items.
#[derive(Clone, Debug)]
pub struct RegistryInstallPlan {
    /// Canonical requested root items after default-registry resolution.
    pub requested: Vec<RegistryItemAddress>,
    /// Deduplicated dependencies followed by their dependents.
    pub items: Vec<ResolvedRegistryItem>,
}

/// Loaded registry sources available for one consumer installation request.
#[derive(Clone, Debug, Default)]
pub struct RegistryCatalog {
    registries: BTreeMap<RegistryNamespace, LoadedRegistry>,
}

impl RegistryCatalog {
    /// Creates an empty catalog. Sources are inserted only after source
    /// validation has completed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a validated registry source, rejecting duplicate namespaces.
    pub fn insert(&mut self, registry: LoadedRegistry) -> Result<(), RegistryError> {
        let namespace = registry.manifest.namespace.clone();
        if self.registries.contains_key(&namespace) {
            return Err(RegistryError::DuplicateRegistrySource {
                namespace: namespace.to_string(),
            });
        }
        self.registries.insert(namespace, registry);
        Ok(())
    }

    /// Resolves requested bare/namespaced addresses into a deterministic plan.
    ///
    /// A bare request uses `default_registry`. A bare dependency is always
    /// local to its declaring registry; it never falls through to the default
    /// registry. Cross-registry dependencies must therefore be explicitly
    /// namespaced in registry metadata.
    pub fn resolve(
        &self,
        default_registry: &RegistryNamespace,
        requests: &[RegistryAddress],
    ) -> Result<RegistryInstallPlan, RegistryError> {
        self.require_registry(default_registry)?;
        let mut requested: Vec<_> = requests
            .iter()
            .map(|request| self.request_address(default_registry, request))
            .collect::<Result<_, _>>()?;
        requested.sort();
        requested.dedup();

        let mut state = ResolutionState::default();
        for address in &requested {
            self.visit(address, &mut state)?;
        }
        Ok(RegistryInstallPlan {
            requested,
            items: state.items,
        })
    }

    /// Resolves every item published by one configured registry in stable
    /// source-name order. This is the single registry-core entry point behind
    /// `adico add --all`; it deliberately does not include other configured
    /// registries unless the caller selects them as the source.
    pub fn resolve_all(
        &self,
        namespace: &RegistryNamespace,
    ) -> Result<RegistryInstallPlan, RegistryError> {
        let registry = self.require_registry(namespace)?;
        let requests = registry
            .manifest
            .items
            .iter()
            .map(|item| RegistryAddress::Bare(item.name.clone()))
            .collect::<Vec<_>>();
        self.resolve(namespace, &requests)
    }

    /// Lists the source items published by one configured registry in stable
    /// item-name order. This read-only operation backs CLI discovery without
    /// involving dependency resolution or consumer-project mutation.
    pub fn items_in(
        &self,
        namespace: &RegistryNamespace,
    ) -> Result<Vec<ResolvedRegistryItem>, RegistryError> {
        let registry = self.require_registry(namespace)?;
        let mut items = registry.manifest.items.clone();
        items.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(items
            .into_iter()
            .map(|item| ResolvedRegistryItem {
                address: RegistryItemAddress {
                    namespace: namespace.clone(),
                    item: item.name.clone(),
                },
                item,
                location: registry.location.clone(),
                manifest_digest: registry.manifest_digest.clone(),
                registry_compatibility: registry.manifest.compatibility.clone(),
            })
            .collect())
    }

    fn request_address(
        &self,
        default_registry: &RegistryNamespace,
        request: &RegistryAddress,
    ) -> Result<RegistryItemAddress, RegistryError> {
        let address = match request {
            RegistryAddress::Bare(item) => RegistryItemAddress {
                namespace: default_registry.clone(),
                item: item.clone(),
            },
            RegistryAddress::Namespaced { namespace, item } => RegistryItemAddress {
                namespace: namespace.clone(),
                item: item.clone(),
            },
        };
        self.require_item(&address)?;
        Ok(address)
    }

    fn visit(
        &self,
        address: &RegistryItemAddress,
        state: &mut ResolutionState,
    ) -> Result<(), RegistryError> {
        if state.resolved.contains(address) {
            return Ok(());
        }
        if !state.visiting.insert(address.clone()) {
            let start = state
                .path
                .iter()
                .position(|entry| entry == address)
                .unwrap_or(0);
            let mut cycle = state.path[start..].to_vec();
            cycle.push(address.clone());
            return Err(RegistryError::CrossRegistryDependencyCycle {
                cycle: cycle.into_iter().map(|entry| entry.to_string()).collect(),
            });
        }
        state.path.push(address.clone());
        let registry = self.require_registry(&address.namespace)?;
        let item = registry
            .manifest
            .items
            .iter()
            .find(|item| item.name == address.item)
            .expect("require_item verifies resolved items exist");
        let mut dependencies = item
            .registry_dependencies
            .iter()
            .map(|dependency| self.dependency_address(&address.namespace, dependency))
            .collect::<Result<Vec<_>, _>>()?;
        dependencies.sort();
        dependencies.dedup();
        for dependency in dependencies {
            self.require_item(&dependency)?;
            self.visit(&dependency, state)?;
        }
        state.path.pop();
        state.visiting.remove(address);
        state.resolved.insert(address.clone());
        state.items.push(ResolvedRegistryItem {
            address: address.clone(),
            item: item.clone(),
            location: registry.location.clone(),
            manifest_digest: registry.manifest_digest.clone(),
            registry_compatibility: registry.manifest.compatibility.clone(),
        });
        Ok(())
    }

    fn dependency_address(
        &self,
        declaring_namespace: &RegistryNamespace,
        dependency: &str,
    ) -> Result<RegistryItemAddress, RegistryError> {
        Ok(match RegistryAddress::parse(dependency)? {
            RegistryAddress::Bare(item) => RegistryItemAddress {
                namespace: declaring_namespace.clone(),
                item,
            },
            RegistryAddress::Namespaced { namespace, item } => {
                RegistryItemAddress { namespace, item }
            }
        })
    }

    fn require_registry(
        &self,
        namespace: &RegistryNamespace,
    ) -> Result<&LoadedRegistry, RegistryError> {
        self.registries
            .get(namespace)
            .ok_or_else(|| RegistryError::UnknownRegistrySource {
                namespace: namespace.to_string(),
            })
    }

    fn require_item(&self, address: &RegistryItemAddress) -> Result<(), RegistryError> {
        let registry = self.require_registry(&address.namespace)?;
        if registry
            .manifest
            .items
            .iter()
            .any(|item| item.name == address.item)
        {
            Ok(())
        } else {
            Err(RegistryError::UnknownRegistryItem {
                address: address.to_string(),
            })
        }
    }
}

#[derive(Default)]
struct ResolutionState {
    visiting: BTreeSet<RegistryItemAddress>,
    resolved: BTreeSet<RegistryItemAddress>,
    path: Vec<RegistryItemAddress>,
    items: Vec<ResolvedRegistryItem>,
}

/// A Cargo dependency requirement merged from one or more registry items.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnifiedCargoDependency {
    /// Dependency key written to Cargo.toml.
    pub crate_name: String,
    /// Underlying crates.io package when the key is an alias.
    pub package: Option<String>,
    /// A Cargo-compatible conjunction of all source version requirements.
    pub version: String,
    /// Deterministically sorted union of requested features.
    pub features: Vec<String>,
    /// Whether any installed source requires package default features.
    pub default_features: bool,
    /// Optional Cargo target predicate. Different predicates remain distinct.
    pub target: Option<String>,
    /// Source-owned items that contributed this requirement.
    pub origins: Vec<RegistryItemAddress>,
}

/// Unifies Cargo requirements declared by a resolved registry install plan.
///
/// Requirements sharing a manifest key and target predicate are combined only
/// when they name the same package and have a satisfiable semver intersection.
/// Cargo features are additive, so their union is deterministic. Default
/// features are retained when any copied source requires them.
pub fn unify_cargo_dependencies(
    plan: &RegistryInstallPlan,
) -> Result<Vec<UnifiedCargoDependency>, RegistryError> {
    let mut merged = BTreeMap::<(String, Option<String>), UnifiedCargoDependency>::new();
    for resolved in &plan.items {
        for requirement in &resolved.item.cargo_dependencies {
            validate_cargo_version_requirement(requirement, &resolved.address)?;
            let key = (requirement.crate_name.clone(), requirement.target.clone());
            if let Some(existing) = merged.get_mut(&key) {
                if existing.package != requirement.package {
                    return Err(RegistryError::CargoPackageConflict {
                        crate_name: requirement.crate_name.clone(),
                        target: requirement.target.clone(),
                        existing_package: existing.package.clone(),
                        requested_package: requirement.package.clone(),
                        origins: origin_strings(
                            existing
                                .origins
                                .iter()
                                .chain(std::iter::once(&resolved.address)),
                        ),
                    });
                }
                // Repeating an identical requirement must not make the
                // consumer-facing declaration needlessly different. This is
                // common when several installed components pin the Dioxus
                // runtime to the same exact release.
                let combined = if existing.version == requirement.version {
                    existing.version.clone()
                } else {
                    format!("{}, {}", existing.version, requirement.version)
                };
                let combined_requirement = VersionReq::parse(&combined).map_err(|error| {
                    RegistryError::InvalidCargoVersionRequirement {
                        crate_name: requirement.crate_name.clone(),
                        requirement: combined.clone(),
                        origin: resolved.address.to_string(),
                        reason: error.to_string(),
                    }
                })?;
                if !has_semver_witness(&combined_requirement) {
                    return Err(RegistryError::IncompatibleCargoRequirements {
                        crate_name: requirement.crate_name.clone(),
                        target: requirement.target.clone(),
                        requirements: vec![existing.version.clone(), requirement.version.clone()],
                        origins: origin_strings(
                            existing
                                .origins
                                .iter()
                                .chain(std::iter::once(&resolved.address)),
                        ),
                    });
                }
                existing.version = combined;
                let mut features: BTreeSet<_> = existing.features.iter().cloned().collect();
                features.extend(requirement.features.iter().cloned());
                existing.features = features.into_iter().collect();
                existing.default_features |= requirement.default_features;
                if !existing.origins.contains(&resolved.address) {
                    existing.origins.push(resolved.address.clone());
                    existing.origins.sort();
                }
            } else {
                merged.insert(
                    key,
                    UnifiedCargoDependency {
                        crate_name: requirement.crate_name.clone(),
                        package: requirement.package.clone(),
                        version: requirement.version.clone(),
                        features: {
                            let mut features: Vec<_> = requirement.features.clone();
                            features.sort();
                            features.dedup();
                            features
                        },
                        default_features: requirement.default_features,
                        target: requirement.target.clone(),
                        origins: vec![resolved.address.clone()],
                    },
                );
            }
        }
    }
    Ok(merged.into_values().collect())
}

fn validate_cargo_version_requirement(
    requirement: &CargoDependency,
    origin: &RegistryItemAddress,
) -> Result<(), RegistryError> {
    let parsed = VersionReq::parse(&requirement.version).map_err(|error| {
        RegistryError::InvalidCargoVersionRequirement {
            crate_name: requirement.crate_name.clone(),
            requirement: requirement.version.clone(),
            origin: origin.to_string(),
            reason: error.to_string(),
        }
    })?;
    if has_semver_witness(&parsed) {
        Ok(())
    } else {
        Err(RegistryError::IncompatibleCargoRequirements {
            crate_name: requirement.crate_name.clone(),
            target: requirement.target.clone(),
            requirements: vec![requirement.version.clone()],
            origins: vec![origin.to_string()],
        })
    }
}

fn origin_strings<'a>(origins: impl Iterator<Item = &'a RegistryItemAddress>) -> Vec<String> {
    origins.map(ToString::to_string).collect()
}

/// Checks whether a Cargo-style semver conjunction has a practical stable
/// release witness. The candidates are comparator boundaries plus their next
/// patch/minor releases, which covers all ordinary Cargo requirements while
/// avoiding an unbounded version search.
fn has_semver_witness(requirement: &VersionReq) -> bool {
    let mut candidates = BTreeSet::from([
        Version::new(0, 0, 0),
        Version::new(0, 1, 0),
        Version::new(1, 0, 0),
    ]);
    for comparator in &requirement.comparators {
        let minor = comparator.minor.unwrap_or(0);
        let patch = comparator.patch.unwrap_or(0);
        let boundary = Version {
            major: comparator.major,
            minor,
            patch,
            pre: comparator.pre.clone(),
            build: Default::default(),
        };
        candidates.insert(boundary.clone());
        if boundary.pre.is_empty() {
            if let Some(next_patch) = boundary.patch.checked_add(1) {
                candidates.insert(Version::new(boundary.major, boundary.minor, next_patch));
            }
            if let Some(next_minor) = boundary.minor.checked_add(1) {
                candidates.insert(Version::new(boundary.major, next_minor, 0));
            }
            if let Some(next_major) = boundary.major.checked_add(1) {
                candidates.insert(Version::new(next_major, 0, 0));
            }
        }
    }
    candidates
        .into_iter()
        .any(|candidate| requirement.matches(&candidate))
}

/// Minimal transport boundary used by static HTTPS registry sources.
///
/// The trait keeps registry validation testable without making CI contact a
/// live organization endpoint. The default implementation performs ordinary
/// unauthenticated static HTTPS GET requests only.
pub trait RegistryHttpClient {
    /// Fetches bytes from an already validated HTTPS URL.
    fn get(&self, url: &Url) -> Result<Vec<u8>, RegistryError>;
}

/// Blocking client for public static HTTPS registry documents.
#[derive(Clone, Debug, Default)]
pub struct StaticHttpsClient;

impl RegistryHttpClient for StaticHttpsClient {
    fn get(&self, url: &Url) -> Result<Vec<u8>, RegistryError> {
        ensure_https_url(url)?;
        let client = reqwest::blocking::Client::builder()
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.url().scheme() == "https" {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .build()
            .map_err(|error| RegistryError::NetworkRequest {
                url: url.to_string(),
                message: error.to_string(),
            })?;
        let response = client
            .get(url.as_str())
            .send()
            .map_err(|error| RegistryError::NetworkRequest {
                url: url.to_string(),
                message: error.to_string(),
            })?
            .error_for_status()
            .map_err(|error| RegistryError::NetworkRequest {
                url: url.to_string(),
                message: error.to_string(),
            })?;
        response
            .bytes()
            .map(|bytes| bytes.to_vec())
            .map_err(|error| RegistryError::NetworkRequest {
                url: url.to_string(),
                message: error.to_string(),
            })
    }
}

/// Loads and validates embedded, local, and static HTTPS registry sources.
#[derive(Clone, Debug)]
pub struct RegistrySourceLoader<Client = StaticHttpsClient> {
    embedded: EmbeddedRegistry,
    http: Client,
}

impl RegistrySourceLoader<StaticHttpsClient> {
    /// Creates a loader that uses the built-in unauthenticated HTTPS client.
    pub fn new(embedded: EmbeddedRegistry) -> Self {
        Self::with_client(embedded, StaticHttpsClient)
    }
}

impl<Client> RegistrySourceLoader<Client> {
    /// Creates a loader with an explicit transport implementation.
    pub fn with_client(embedded: EmbeddedRegistry, http: Client) -> Self {
        Self { embedded, http }
    }
}

impl<Client: RegistryHttpClient> RegistrySourceLoader<Client> {
    /// Loads a source configured under `namespace` and validates all local
    /// invariants before an installer can construct a project mutation plan.
    pub fn load(
        &self,
        namespace: &RegistryNamespace,
        source: &RegistrySource,
    ) -> Result<LoadedRegistry, RegistryError> {
        let (manifest_bytes, location) = match source {
            RegistrySource::Embedded => (
                self.embedded.manifest.clone(),
                RegistryLocation::Embedded {
                    label: self.embedded.label.clone(),
                    source_root: self.embedded.source_root.clone(),
                },
            ),
            RegistrySource::Local { path } => {
                let manifest_path = local_manifest_path(path)?;
                let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
                    RegistryError::UnreadableLocalSource {
                        path: manifest_path.display().to_string(),
                        message: error.to_string(),
                    }
                })?;
                let source_root =
                    manifest_path
                        .parent()
                        .map(Path::to_path_buf)
                        .ok_or_else(|| RegistryError::UnreadableLocalSource {
                            path: manifest_path.display().to_string(),
                            message: "manifest path has no parent directory".to_string(),
                        })?;
                (
                    manifest_bytes,
                    RegistryLocation::Local {
                        manifest_path,
                        source_root,
                    },
                )
            }
            RegistrySource::Https { url } => {
                let manifest_url = parse_https_url(url)?;
                let source_root = https_source_root(&manifest_url)?;
                let manifest_bytes = self.http.get(&manifest_url)?;
                (
                    manifest_bytes,
                    RegistryLocation::Https {
                        manifest_url,
                        source_root,
                    },
                )
            }
        };

        let manifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
            RegistryError::MalformedManifest {
                registry_source: location.to_string(),
                message: error.to_string(),
            }
        })?;
        let loaded = LoadedRegistry {
            manifest,
            location,
            manifest_digest: sha256_hex(&manifest_bytes),
        };
        if &loaded.manifest.namespace != namespace {
            return Err(RegistryError::NamespaceMismatch {
                configured: namespace.to_string(),
                declared: loaded.manifest.namespace.to_string(),
                registry_source: loaded.location.to_string(),
            });
        }
        self.validate(&loaded)?;
        Ok(loaded)
    }

    /// Validates a previously loaded registry and every source file it names.
    pub fn validate(&self, registry: &LoadedRegistry) -> Result<(), RegistryError> {
        registry.manifest.validate()?;
        validate_compatibility(
            &registry.manifest.compatibility,
            &registry.manifest.namespace,
            "registry",
        )?;

        let names: BTreeSet<_> = registry
            .manifest
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        let mut targets = BTreeSet::new();
        for item in &registry.manifest.items {
            if let Some(compatibility) = &item.compatibility {
                validate_compatibility(compatibility, &registry.manifest.namespace, &item.name)?;
            }
            for dependency in &item.registry_dependencies {
                let address = RegistryAddress::parse(dependency)?;
                let local_name = match address {
                    RegistryAddress::Bare(name) => Some(name),
                    RegistryAddress::Namespaced {
                        namespace,
                        item: name,
                    } if namespace == registry.manifest.namespace => Some(name),
                    RegistryAddress::Namespaced { .. } => None,
                };
                if let Some(local_name) = local_name
                    && !names.contains(local_name.as_str())
                {
                    return Err(RegistryError::MissingLocalDependency {
                        item: item.name.clone(),
                        dependency: dependency.clone(),
                        namespace: registry.manifest.namespace.to_string(),
                    });
                }
            }
            for file in &item.files {
                let source = validated_relative_path(&file.source, "source")?;
                let target = validated_relative_path(&file.target, "target")?;
                let target_intent = format!("{:?}:{}", file.target_root, target.display());
                if !targets.insert(target_intent.clone()) {
                    return Err(RegistryError::DuplicateTarget {
                        target: target_intent,
                    });
                }
                validate_checksum(&file.checksum, &item.name, &file.source)?;
                let actual = sha256_hex(&self.read_source(registry, &source)?);
                if actual != file.checksum {
                    return Err(RegistryError::ChecksumMismatch {
                        item: item.name.clone(),
                        file_source: file.source.clone(),
                        expected: file.checksum.clone(),
                        actual,
                    });
                }
            }
        }
        validate_local_dependency_cycles(&registry.manifest, &names)
    }

    /// Reads a registry source file after its containing registry was loaded.
    pub fn read_source(
        &self,
        registry: &LoadedRegistry,
        relative_path: &Path,
    ) -> Result<Vec<u8>, RegistryError> {
        self.read_source_location(&registry.location, relative_path)
    }

    /// Reads a source file for a resolved item while retaining its registry
    /// location. CLI installers use this after dependency resolution, so local
    /// and static-HTTPS organization registries use the same checked source
    /// path as validation.
    pub fn read_resolved_source(
        &self,
        item: &ResolvedRegistryItem,
        source: &str,
    ) -> Result<Vec<u8>, RegistryError> {
        let relative_path = validated_relative_path(source, "source")?;
        self.read_source_location(&item.location, &relative_path)
    }

    fn read_source_location(
        &self,
        location: &RegistryLocation,
        relative_path: &Path,
    ) -> Result<Vec<u8>, RegistryError> {
        match location {
            RegistryLocation::Embedded {
                source_root, label, ..
            } => read_local_source(source_root, relative_path, label),
            RegistryLocation::Local { source_root, .. } => {
                read_local_source(source_root, relative_path, &location.to_string())
            }
            RegistryLocation::Https { source_root, .. } => {
                let url = source_root
                    .join(relative_path.to_str().ok_or_else(|| {
                        RegistryError::InvalidSourcePath {
                            path: relative_path.display().to_string(),
                            reason: "path is not valid UTF-8".to_string(),
                        }
                    })?)
                    .map_err(|error| RegistryError::InvalidSourcePath {
                        path: relative_path.display().to_string(),
                        reason: format!("cannot resolve against HTTPS registry root: {error}"),
                    })?;
                ensure_https_url(&url)?;
                self.http.get(&url)
            }
        }
    }
}

fn local_manifest_path(configured_path: &str) -> Result<PathBuf, RegistryError> {
    let path = PathBuf::from(configured_path);
    let manifest_path = if path.is_dir() {
        path.join("registry.json")
    } else {
        path
    };
    if !manifest_path.is_file() {
        return Err(RegistryError::UnreadableLocalSource {
            path: manifest_path.display().to_string(),
            message: "expected a registry.json file or a path to a manifest file".to_string(),
        });
    }
    Ok(manifest_path)
}

fn parse_https_url(value: &str) -> Result<Url, RegistryError> {
    let url = Url::parse(value).map_err(|error| RegistryError::InvalidHttpsUrl {
        url: value.to_string(),
        reason: error.to_string(),
    })?;
    ensure_https_url(&url)?;
    Ok(url)
}

fn ensure_https_url(url: &Url) -> Result<(), RegistryError> {
    if url.scheme() != "https" {
        return Err(RegistryError::InvalidHttpsUrl {
            url: url.to_string(),
            reason: "only static HTTPS registry endpoints are supported".to_string(),
        });
    }
    if url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() {
        return Err(RegistryError::InvalidHttpsUrl {
            url: url.to_string(),
            reason: "an HTTPS registry endpoint must have a host and no embedded credentials"
                .to_string(),
        });
    }
    Ok(())
}

fn https_source_root(manifest_url: &Url) -> Result<Url, RegistryError> {
    let mut root = manifest_url.clone();
    let directory = {
        let path = root.path();
        let directory_end = path.rfind('/').unwrap_or(0) + 1;
        path[..directory_end].to_string()
    };
    root.set_path(&directory);
    root.set_query(None);
    root.set_fragment(None);
    ensure_https_url(&root)?;
    Ok(root)
}

fn read_local_source(
    source_root: &Path,
    relative_path: &Path,
    source: &str,
) -> Result<Vec<u8>, RegistryError> {
    let root =
        fs::canonicalize(source_root).map_err(|error| RegistryError::UnreadableRegistryFile {
            registry_source: source.to_string(),
            path: source_root.display().to_string(),
            message: error.to_string(),
        })?;
    let path = fs::canonicalize(source_root.join(relative_path)).map_err(|error| {
        RegistryError::UnreadableRegistryFile {
            registry_source: source.to_string(),
            path: source_root.join(relative_path).display().to_string(),
            message: error.to_string(),
        }
    })?;
    if !path.starts_with(&root) {
        return Err(RegistryError::InvalidSourcePath {
            path: relative_path.display().to_string(),
            reason: "source file resolves outside the configured registry root".to_string(),
        });
    }
    fs::read(&path).map_err(|error| RegistryError::UnreadableRegistryFile {
        registry_source: source.to_string(),
        path: path.display().to_string(),
        message: error.to_string(),
    })
}

fn validated_relative_path(value: &str, kind: &str) -> Result<PathBuf, RegistryError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
                    | Component::CurDir
            )
        })
    {
        return Err(RegistryError::InvalidSourcePath {
            path: value.to_string(),
            reason: format!("{kind} paths must be non-empty, relative paths without '.' or '..'"),
        });
    }
    Ok(path.to_path_buf())
}

fn validate_checksum(checksum: &str, item: &str, source: &str) -> Result<(), RegistryError> {
    if checksum.len() == 64
        && checksum
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(RegistryError::InvalidChecksum {
            item: item.to_string(),
            file_source: source.to_string(),
            checksum: checksum.to_string(),
        })
    }
}

fn validate_compatibility(
    compatibility: &RegistryCompatibility,
    namespace: &RegistryNamespace,
    subject: &str,
) -> Result<(), RegistryError> {
    validate_version_requirement(
        &compatibility.cli,
        ADICO_CLI_VERSION,
        namespace,
        subject,
        "CLI",
    )?;
    if let Some(runtime) = &compatibility.runtime {
        validate_version_requirement(
            runtime,
            ADICO_PRIMITIVES_VERSION,
            namespace,
            subject,
            "adico-primitives",
        )?;
    }
    Ok(())
}

fn validate_version_requirement(
    requirement: &str,
    supported: &str,
    namespace: &RegistryNamespace,
    subject: &str,
    dependency: &str,
) -> Result<(), RegistryError> {
    let requirement = VersionReq::parse(requirement).map_err(|error| {
        RegistryError::InvalidCompatibilityRange {
            namespace: namespace.to_string(),
            subject: subject.to_string(),
            dependency: dependency.to_string(),
            range: requirement.to_string(),
            reason: error.to_string(),
        }
    })?;
    let supported = Version::parse(supported).expect("adico compatibility constants are valid");
    if !requirement.matches(&supported) {
        return Err(RegistryError::IncompatibleVersion {
            namespace: namespace.to_string(),
            subject: subject.to_string(),
            dependency: dependency.to_string(),
            required: requirement.to_string(),
            supported: supported.to_string(),
        });
    }
    Ok(())
}

fn validate_local_dependency_cycles(
    manifest: &RegistryManifest,
    names: &BTreeSet<&str>,
) -> Result<(), RegistryError> {
    let mut dependencies = BTreeMap::<String, Vec<String>>::new();
    for item in &manifest.items {
        let mut local_dependencies = Vec::new();
        for dependency in &item.registry_dependencies {
            match RegistryAddress::parse(dependency)? {
                RegistryAddress::Bare(name) => local_dependencies.push(name),
                RegistryAddress::Namespaced { namespace, item }
                    if namespace == manifest.namespace =>
                {
                    local_dependencies.push(item);
                }
                RegistryAddress::Namespaced { .. } => {}
            }
        }
        dependencies.insert(
            item.name.clone(),
            local_dependencies
                .iter()
                .filter(|dependency| names.contains(dependency.as_str()))
                .cloned()
                .collect(),
        );
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut path = Vec::new();
    for item in dependencies.keys() {
        visit_dependency(item, &dependencies, &mut visiting, &mut visited, &mut path)?;
    }
    Ok(())
}

fn visit_dependency(
    item: &str,
    dependencies: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    path: &mut Vec<String>,
) -> Result<(), RegistryError> {
    if visited.contains(item) {
        return Ok(());
    }
    if !visiting.insert(item.to_string()) {
        let start = path.iter().position(|entry| *entry == item).unwrap_or(0);
        let mut cycle = path[start..].to_vec();
        cycle.push(item.to_string());
        return Err(RegistryError::DependencyCycle { cycle });
    }
    path.push(item.to_string());
    for dependency in dependencies.get(item).into_iter().flatten() {
        visit_dependency(dependency, dependencies, visiting, visited, path)?;
    }
    path.pop();
    visiting.remove(item);
    visited.insert(item.to_string());
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
    /// `components.json` could not be parsed as the supported configuration shape.
    #[error("components.json is malformed: {message}")]
    MalformedComponentsConfiguration {
        /// JSON/schema parsing reason.
        message: String,
    },
    /// A consumer configuration version has no supported automatic migration.
    #[error(
        "components.json version {actual} is unsupported; this adico build supports version {supported}. Run `adico init` to create or explicitly migrate a v{supported} configuration."
    )]
    UnsupportedComponentsConfigurationVersion {
        /// Version declared in the consumer file.
        actual: u32,
        /// Version supported by this build.
        supported: u32,
    },
    /// A consumer configuration field has an unsafe or non-portable path.
    #[error("components.json {field} path {path:?} is invalid: {reason}")]
    InvalidComponentsConfigurationPath {
        /// Configuration field name.
        field: String,
        /// Invalid configured path.
        path: String,
        /// Validation reason.
        reason: String,
    },
    /// A consumer configuration field is missing a required meaningful value.
    #[error("components.json {field} is invalid: {reason}")]
    InvalidComponentsConfigurationValue {
        /// Configuration field name.
        field: String,
        /// Validation reason.
        reason: String,
    },
    /// The default registry was not declared among named sources.
    #[error("components.json defaultRegistry {namespace} is not configured")]
    UnknownDefaultRegistry {
        /// Missing default registry namespace.
        namespace: String,
    },
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
    /// More than one loaded source claimed the same namespace.
    #[error("registry namespace {namespace} was configured more than once")]
    DuplicateRegistrySource {
        /// Namespace configured by duplicate sources.
        namespace: String,
    },
    /// A requested/default namespace is not available in the loaded catalog.
    #[error("registry source {namespace} is not configured or could not be loaded")]
    UnknownRegistrySource {
        /// Missing namespace.
        namespace: String,
    },
    /// A registry source does not provide a requested item.
    #[error("registry item {address} is not available from its selected source")]
    UnknownRegistryItem {
        /// Fully qualified requested address.
        address: String,
    },
    /// Two requirements use one Cargo dependency key for different packages.
    #[error(
        "Cargo dependency {crate_name:?} has conflicting package aliases for target {target:?}: {existing_package:?} versus {requested_package:?} (from {})",
        .origins.join(", ")
    )]
    CargoPackageConflict {
        /// Cargo manifest key.
        crate_name: String,
        /// Optional Cargo target predicate.
        target: Option<String>,
        /// Package selected by the first requirement.
        existing_package: Option<String>,
        /// Package selected by the conflicting requirement.
        requested_package: Option<String>,
        /// Fully-qualified source items that introduced the conflict.
        origins: Vec<String>,
    },
    /// A Cargo semver requirement cannot be parsed.
    #[error(
        "Cargo dependency {crate_name:?} has invalid version requirement {requirement:?} from {origin}: {reason}"
    )]
    InvalidCargoVersionRequirement {
        /// Cargo manifest key.
        crate_name: String,
        /// Invalid requirement text.
        requirement: String,
        /// Fully-qualified item that declared the requirement.
        origin: String,
        /// Semver parser reason.
        reason: String,
    },
    /// Cargo requirements have no shared compatible version.
    #[error(
        "Cargo dependency {crate_name:?} has incompatible requirements {requirements:?} for target {target:?} (from {})",
        .origins.join(", ")
    )]
    IncompatibleCargoRequirements {
        /// Cargo manifest key.
        crate_name: String,
        /// Optional Cargo target predicate.
        target: Option<String>,
        /// Mutually incompatible requirement text.
        requirements: Vec<String>,
        /// Fully-qualified source items that declared the requirements.
        origins: Vec<String>,
    },
    /// The configured source namespace did not match the manifest identity.
    #[error(
        "registry source {registry_source} declares {declared}, but the project configured it as {configured}"
    )]
    NamespaceMismatch {
        /// Namespace selected in consumer configuration.
        configured: String,
        /// Namespace declared by the loaded manifest.
        declared: String,
        /// Source that returned the mismatched manifest.
        registry_source: String,
    },
    /// A local registry directory or manifest is unreadable.
    #[error("cannot read local registry source {path}: {message}")]
    UnreadableLocalSource {
        /// Path configured by the consumer.
        path: String,
        /// Underlying filesystem reason.
        message: String,
    },
    /// A registry manifest cannot be parsed.
    #[error("registry manifest from {registry_source} is malformed: {message}")]
    MalformedManifest {
        /// Source that supplied the manifest.
        registry_source: String,
        /// JSON/schema parsing reason.
        message: String,
    },
    /// A static source endpoint is not a safe HTTPS URL.
    #[error("invalid static registry URL {url}: {reason}")]
    InvalidHttpsUrl {
        /// Configured endpoint.
        url: String,
        /// Validation reason.
        reason: String,
    },
    /// A static HTTPS request failed without applying any consumer changes.
    #[error("cannot fetch static registry source {url}: {message}")]
    NetworkRequest {
        /// Requested HTTPS URL.
        url: String,
        /// Transport reason.
        message: String,
    },
    /// An item source or destination path could escape its intended root.
    #[error("invalid registry path {path:?}: {reason}")]
    InvalidSourcePath {
        /// Invalid declared path.
        path: String,
        /// Validation reason.
        reason: String,
    },
    /// A source file cannot be read from an otherwise valid registry.
    #[error("cannot read registry file {path} from {registry_source}: {message}")]
    UnreadableRegistryFile {
        /// Registry source identity.
        registry_source: String,
        /// Fully resolved file location.
        path: String,
        /// Underlying reason.
        message: String,
    },
    /// A checksum does not use lowercase SHA-256 hexadecimal form.
    #[error(
        "registry item {item:?} source {file_source:?} has invalid SHA-256 checksum {checksum:?}"
    )]
    InvalidChecksum {
        /// Item declaring the file.
        item: String,
        /// Declared source path.
        file_source: String,
        /// Invalid checksum value.
        checksum: String,
    },
    /// A source file no longer matches its registry metadata checksum.
    #[error(
        "registry item {item:?} source {file_source:?} checksum mismatch: expected {expected}, found {actual}"
    )]
    ChecksumMismatch {
        /// Item declaring the file.
        item: String,
        /// Declared source path.
        file_source: String,
        /// Checksum declared in metadata.
        expected: String,
        /// Checksum calculated from source bytes.
        actual: String,
    },
    /// Two registry files declare the same consumer destination intent.
    #[error("registry declares duplicate target intent {target:?}")]
    DuplicateTarget {
        /// Logical target root and relative target path.
        target: String,
    },
    /// An in-registry dependency does not name an available item.
    #[error(
        "registry item {item:?} depends on missing local item {dependency:?} in namespace {namespace}"
    )]
    MissingLocalDependency {
        /// Item declaring the dependency.
        item: String,
        /// Missing dependency reference.
        dependency: String,
        /// Namespace in which it should exist.
        namespace: String,
    },
    /// Local registry dependencies form a cycle.
    #[error("registry dependency cycle: {}", .cycle.join(" -> "))]
    DependencyCycle {
        /// Cycle order, including the repeated start item.
        cycle: Vec<String>,
    },
    /// A dependency cycle crosses one or more configured registry sources.
    #[error("cross-registry dependency cycle: {}", .cycle.join(" -> "))]
    CrossRegistryDependencyCycle {
        /// Fully-qualified cycle order, including the repeated start item.
        cycle: Vec<String>,
    },
    /// A compatibility range is not valid semver syntax.
    #[error(
        "registry {namespace} {subject} has invalid {dependency} compatibility range {range:?}: {reason}"
    )]
    InvalidCompatibilityRange {
        /// Registry namespace.
        namespace: String,
        /// Registry or item identity.
        subject: String,
        /// CLI or runtime dependency name.
        dependency: String,
        /// Declared range.
        range: String,
        /// Semver parser reason.
        reason: String,
    },
    /// A valid compatibility range excludes this build's supported version.
    #[error(
        "registry {namespace} {subject} requires {dependency} {required}, but this adico build supports {supported}"
    )]
    IncompatibleVersion {
        /// Registry namespace.
        namespace: String,
        /// Registry or item identity.
        subject: String,
        /// CLI or runtime dependency name.
        dependency: String,
        /// Required semver range.
        required: String,
        /// Locally supported version.
        supported: String,
    },
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
    use serde_json::json;

    #[derive(Default)]
    struct FixtureHttpClient {
        responses: BTreeMap<String, Vec<u8>>,
    }

    impl RegistryHttpClient for FixtureHttpClient {
        fn get(&self, url: &Url) -> Result<Vec<u8>, RegistryError> {
            self.responses
                .get(url.as_str())
                .cloned()
                .ok_or_else(|| RegistryError::NetworkRequest {
                    url: url.to_string(),
                    message: "fixture response was not configured".to_string(),
                })
        }
    }

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/compile/registry/validation-source")
    }

    fn fixture_bytes(name: &str) -> Vec<u8> {
        fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compile/registry")
                .join(name),
        )
        .expect("fixture should be readable")
    }

    fn components_fixture(name: &str) -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compile/components")
                .join(name),
        )
        .expect("components fixture should be readable")
    }

    fn fixture_loader() -> RegistrySourceLoader<FixtureHttpClient> {
        RegistrySourceLoader::with_client(
            EmbeddedRegistry::new(
                fixture_bytes("validation-source/registry.json"),
                fixture_root(),
            ),
            FixtureHttpClient::default(),
        )
    }

    fn loaded_fixture(name: &str) -> LoadedRegistry {
        let bytes = fixture_bytes(name);
        LoadedRegistry {
            manifest: serde_json::from_slice(&bytes).expect("fixture should deserialize"),
            location: RegistryLocation::Local {
                manifest_path: fixture_root().join(name),
                source_root: fixture_root(),
            },
            manifest_digest: sha256_hex(&bytes),
        }
    }

    fn registry_with_items(namespace: &str, items: &[(&str, &[&str])]) -> LoadedRegistry {
        let items: Vec<_> = items
            .iter()
            .map(|(name, dependencies)| {
                json!({
                    "name": name,
                    "type": "registry:ui",
                    "description": format!("{name} fixture item"),
                    "registryDependencies": dependencies,
                    "files": [{
                        "source": "ui/button.rs",
                        "targetRoot": "ui",
                        "target": format!("{name}.rs"),
                        "checksum": "73058a07c2b84095985ca37efb4d42a7c11680a61dc27670d9b1ec4c64b63f2c"
                    }]
                })
            })
            .collect();
        let manifest = serde_json::from_value(json!({
            "formatVersion": 1,
            "namespace": namespace,
            "name": format!("{namespace} resolver fixture"),
            "compatibility": { "cli": ">=0.1.0" },
            "items": items,
        }))
        .expect("resolver fixture should deserialize");
        LoadedRegistry {
            manifest,
            location: RegistryLocation::Embedded {
                label: format!("{namespace} fixture"),
                source_root: fixture_root(),
            },
            manifest_digest: "fixture-digest".to_string(),
        }
    }

    fn catalog(registries: Vec<LoadedRegistry>) -> RegistryCatalog {
        let mut catalog = RegistryCatalog::new();
        for registry in registries {
            catalog.insert(registry).expect("unique fixture namespace");
        }
        catalog
    }

    fn plan_addresses(plan: &RegistryInstallPlan) -> Vec<String> {
        plan.items
            .iter()
            .map(|item| item.address.to_string())
            .collect()
    }

    fn cargo_requirement(
        crate_name: &str,
        package: Option<&str>,
        version: &str,
        features: &[&str],
        default_features: bool,
        target: Option<&str>,
    ) -> CargoDependency {
        CargoDependency {
            crate_name: crate_name.to_string(),
            package: package.map(str::to_string),
            version: version.to_string(),
            features: features.iter().map(ToString::to_string).collect(),
            default_features,
            target: target.map(str::to_string),
        }
    }

    fn cargo_plan(items: &[(&str, Vec<CargoDependency>)]) -> RegistryInstallPlan {
        RegistryInstallPlan {
            requested: Vec::new(),
            items: items
                .iter()
                .map(|(name, cargo_dependencies)| ResolvedRegistryItem {
                    address: RegistryItemAddress {
                        namespace: "@adico".parse().expect("valid namespace"),
                        item: (*name).to_string(),
                    },
                    item: RegistryItem {
                        name: (*name).to_string(),
                        item_type: RegistryItemType::Lib,
                        description: "Cargo requirement fixture".to_string(),
                        files: Vec::new(),
                        registry_dependencies: Vec::new(),
                        cargo_dependencies: cargo_dependencies.clone(),
                        style: StyleRequirements::default(),
                        module_exports: Vec::new(),
                        documentation: None,
                        compatibility: None,
                        provenance: None,
                    },
                    location: RegistryLocation::Embedded {
                        label: "Cargo fixture".to_string(),
                        source_root: fixture_root(),
                    },
                    manifest_digest: "fixture-digest".to_string(),
                    registry_compatibility: RegistryCompatibility {
                        cli: ">=0.1.0".to_string(),
                        runtime: None,
                    },
                })
                .collect(),
        }
    }

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
    fn components_configuration_accepts_official_and_company_registry_defaults() {
        let official = ComponentsConfiguration::parse(&components_fixture("official-valid.json"))
            .expect("official default should be valid");
        assert_eq!(official.default_registry.as_str(), "@adico");

        let company =
            ComponentsConfiguration::parse(&components_fixture("company-default-valid.json"))
                .expect("company default should be valid");
        assert_eq!(company.default_registry.as_str(), "@awwwkshay");
        assert!(matches!(
            company.registries.get(&company.default_registry),
            Some(RegistrySource::Https { .. })
        ));
    }

    #[test]
    fn components_configuration_rejects_unsafe_paths_http_and_unknown_versions() {
        assert!(matches!(
            ComponentsConfiguration::parse(&components_fixture("invalid-path.json"))
                .expect_err("parent path must fail"),
            RegistryError::InvalidComponentsConfigurationPath { .. }
        ));
        assert!(matches!(
            ComponentsConfiguration::parse(&components_fixture("invalid-url.json"))
                .expect_err("HTTP source must fail"),
            RegistryError::InvalidHttpsUrl { .. }
        ));
        assert!(matches!(
            ComponentsConfiguration::parse(&components_fixture("unsupported-version.json"))
                .expect_err("unknown version must fail without migration"),
            RegistryError::UnsupportedComponentsConfigurationVersion { .. }
        ));
        let invalid_namespace =
            components_fixture("official-valid.json").replace("@adico", "@Awwwkshay");
        assert!(matches!(
            ComponentsConfiguration::parse(&invalid_namespace)
                .expect_err("namespace grammar must be enforced"),
            RegistryError::MalformedComponentsConfiguration { .. }
        ));
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

    #[test]
    fn embedded_and_local_sources_load_with_a_stable_manifest_digest() {
        let loader = fixture_loader();
        let namespace: RegistryNamespace = "@adico".parse().expect("valid namespace");

        let embedded = loader
            .load(&namespace, &RegistrySource::Embedded)
            .expect("embedded fixture should load");
        let local = loader
            .load(
                &namespace,
                &RegistrySource::Local {
                    path: fixture_root().display().to_string(),
                },
            )
            .expect("local fixture should load");

        assert_eq!(embedded.manifest_digest(), local.manifest_digest());
        assert!(matches!(
            embedded.location,
            RegistryLocation::Embedded { .. }
        ));
        assert!(matches!(local.location, RegistryLocation::Local { .. }));
    }

    #[test]
    fn static_https_sources_resolve_manifest_and_source_files_from_the_same_endpoint() {
        let manifest_url = "https://registry.awwwkshay.example/registry.json";
        let source_url = "https://registry.awwwkshay.example/ui/button.rs";
        let mut client = FixtureHttpClient::default();
        client.responses.insert(
            manifest_url.to_string(),
            fixture_bytes("validation-source/registry.json"),
        );
        client.responses.insert(
            source_url.to_string(),
            fs::read(fixture_root().join("ui/button.rs")).expect("source fixture should exist"),
        );
        let loader = RegistrySourceLoader::with_client(
            EmbeddedRegistry::new(
                fixture_bytes("validation-source/registry.json"),
                fixture_root(),
            ),
            client,
        );

        let loaded = loader
            .load(
                &"@adico".parse().expect("valid namespace"),
                &RegistrySource::Https {
                    url: manifest_url.to_string(),
                },
            )
            .expect("HTTPS fixture should load");

        assert!(matches!(loaded.location, RegistryLocation::Https { .. }));
    }

    fn awwwkshay_fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/installation/awwwkshay-consumer/awwwkshay-registry")
    }

    fn awwwkshay_fixture_bytes(name: &str) -> Vec<u8> {
        fs::read(awwwkshay_fixture_root().join(name)).expect("awwwkshay fixture should be readable")
    }

    /// The checked-in Awwwkshay organization registry fixture (also exercised
    /// end-to-end by the `adico` binary against
    /// `tests/installation/awwwkshay-consumer`) resolves identically whether a
    /// consumer configures it as a local path or a static HTTPS endpoint, and
    /// its explicit cross-registry dependency on `@adico/cn` is preserved.
    #[test]
    fn awwwkshay_registry_fixture_resolves_identically_over_local_and_https_sources() {
        let manifest_bytes = awwwkshay_fixture_bytes("registry.json");
        let card_bytes = awwwkshay_fixture_bytes("ui/card.rs");
        let manifest_url = "https://registry.awwwkshay.example/registry.json";
        let source_url = "https://registry.awwwkshay.example/ui/card.rs";
        let mut client = FixtureHttpClient::default();
        client
            .responses
            .insert(manifest_url.to_string(), manifest_bytes.clone());
        client
            .responses
            .insert(source_url.to_string(), card_bytes.clone());
        let loader = RegistrySourceLoader::with_client(
            EmbeddedRegistry::new(
                fixture_bytes("validation-source/registry.json"),
                fixture_root(),
            ),
            client,
        );
        let namespace: RegistryNamespace = "@awwwkshay".parse().expect("valid namespace");

        let local = loader
            .load(
                &namespace,
                &RegistrySource::Local {
                    path: awwwkshay_fixture_root().display().to_string(),
                },
            )
            .expect("local Awwwkshay fixture should load");
        let https = loader
            .load(
                &namespace,
                &RegistrySource::Https {
                    url: manifest_url.to_string(),
                },
            )
            .expect("HTTPS Awwwkshay fixture should load");

        assert!(matches!(local.location, RegistryLocation::Local { .. }));
        assert!(matches!(https.location, RegistryLocation::Https { .. }));
        assert_eq!(local.manifest.items, https.manifest.items);
        assert_eq!(
            local
                .manifest
                .items
                .iter()
                .find(|item| item.name == "card")
                .expect("card item should be present")
                .registry_dependencies,
            vec!["@adico/cn".to_string()]
        );

        let synthetic_official = registry_with_items("@adico", &[("cn", &[])]);
        for awwwkshay in [local, https] {
            let plan = catalog(vec![synthetic_official.clone(), awwwkshay])
                .resolve(
                    &namespace,
                    &[RegistryAddress::parse("card").expect("valid request")],
                )
                .expect("bare card should resolve through its explicit cross-registry dependency");
            assert_eq!(plan_addresses(&plan), ["@adico/cn", "@awwwkshay/card"]);
            let card = plan
                .items
                .iter()
                .find(|item| item.address.to_string() == "@awwwkshay/card")
                .expect("resolved card item should be present");
            assert_eq!(card.item.files[0].checksum, sha256_hex(&card_bytes));
        }
    }

    #[test]
    fn non_https_endpoints_are_rejected_before_any_network_request() {
        let error = fixture_loader()
            .load(
                &"@adico".parse().expect("valid namespace"),
                &RegistrySource::Https {
                    url: "http://registry.awwwkshay.example/registry.json".to_string(),
                },
            )
            .expect_err("HTTP must not be loaded");
        assert!(matches!(error, RegistryError::InvalidHttpsUrl { .. }));
    }

    #[test]
    fn negative_manifest_fixtures_fail_before_an_install_plan_exists() {
        let loader = fixture_loader();
        assert!(matches!(
            loader
                .validate(&loaded_fixture("checksum-mismatch.json"))
                .expect_err("checksum must fail"),
            RegistryError::ChecksumMismatch { .. }
        ));
        assert!(matches!(
            loader
                .validate(&loaded_fixture("duplicate-target.json"))
                .expect_err("duplicate target must fail"),
            RegistryError::DuplicateTarget { .. }
        ));
        assert!(matches!(
            loader
                .validate(&loaded_fixture("missing-dependency.json"))
                .expect_err("missing dependency must fail"),
            RegistryError::MissingLocalDependency { .. }
        ));
        assert!(matches!(
            loader
                .validate(&loaded_fixture("dependency-cycle.json"))
                .expect_err("cycle must fail"),
            RegistryError::DependencyCycle { .. }
        ));
        assert!(matches!(
            loader
                .validate(&loaded_fixture("incompatible-cli.json"))
                .expect_err("incompatible CLI must fail"),
            RegistryError::IncompatibleVersion { .. }
        ));
    }

    #[test]
    fn unsafe_source_paths_and_namespace_mismatches_have_specific_diagnostics() {
        let mut unsafe_manifest = fixture_bytes("validation-source/registry.json");
        unsafe_manifest = String::from_utf8(unsafe_manifest)
            .expect("fixture is UTF-8")
            .replace("ui/button.rs", "../button.rs")
            .into_bytes();
        let loader = RegistrySourceLoader::with_client(
            EmbeddedRegistry::new(unsafe_manifest, fixture_root()),
            FixtureHttpClient::default(),
        );
        let namespace: RegistryNamespace = "@adico".parse().expect("valid namespace");
        assert!(matches!(
            loader
                .load(&namespace, &RegistrySource::Embedded)
                .expect_err("parent path must fail"),
            RegistryError::InvalidSourcePath { .. }
        ));

        let error = fixture_loader()
            .load(
                &"@awwwkshay".parse().expect("valid namespace"),
                &RegistrySource::Embedded,
            )
            .expect_err("configured namespace must match manifest namespace");
        assert!(matches!(error, RegistryError::NamespaceMismatch { .. }));
    }

    #[test]
    fn resolver_deduplicates_shared_dependencies_in_a_stable_dependency_first_order() {
        let catalog = catalog(vec![registry_with_items(
            "@adico",
            &[
                ("utility", &[]),
                ("button", &["utility"]),
                ("card", &["utility"]),
            ],
        )]);
        let default: RegistryNamespace = "@adico".parse().expect("valid namespace");
        let plan = catalog
            .resolve(
                &default,
                &[
                    RegistryAddress::parse("card").expect("valid request"),
                    RegistryAddress::parse("button").expect("valid request"),
                ],
            )
            .expect("dependencies should resolve");

        assert_eq!(
            plan_addresses(&plan),
            ["@adico/utility", "@adico/button", "@adico/card"]
        );
        assert_eq!(
            plan.requested
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["@adico/button", "@adico/card"]
        );
        let reversed = catalog
            .resolve(
                &default,
                &[
                    RegistryAddress::parse("button").expect("valid request"),
                    RegistryAddress::parse("card").expect("valid request"),
                ],
            )
            .expect("reversed request order should resolve");
        assert_eq!(plan_addresses(&plan), plan_addresses(&reversed));
    }

    #[test]
    fn resolver_all_uses_the_selected_registry_once_in_stable_order() {
        let catalog = catalog(vec![registry_with_items(
            "@awwwkshay",
            &[
                ("utility", &[]),
                ("button", &["utility"]),
                ("card", &["utility"]),
            ],
        )]);
        let selected: RegistryNamespace = "@awwwkshay".parse().expect("valid namespace");
        let plan = catalog
            .resolve_all(&selected)
            .expect("all configured company items should resolve");
        assert_eq!(
            plan_addresses(&plan),
            ["@awwwkshay/utility", "@awwwkshay/button", "@awwwkshay/card"]
        );
        assert_eq!(
            plan.requested
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["@awwwkshay/button", "@awwwkshay/card", "@awwwkshay/utility"]
        );
    }

    #[test]
    fn resolver_keeps_company_defaults_and_explicit_official_items_separate() {
        let catalog = catalog(vec![
            registry_with_items("@adico", &[("button", &[])]),
            registry_with_items("@awwwkshay", &[("company-card", &["@adico/button"])]),
        ]);
        let default: RegistryNamespace = "@awwwkshay".parse().expect("valid namespace");
        let plan = catalog
            .resolve(
                &default,
                &[
                    RegistryAddress::parse("company-card").expect("valid request"),
                    RegistryAddress::parse("@adico/button").expect("valid request"),
                ],
            )
            .expect("explicit cross-registry dependency should resolve");

        assert_eq!(
            plan_addresses(&plan),
            ["@adico/button", "@awwwkshay/company-card"]
        );
        assert_eq!(plan.items[1].manifest_digest, "fixture-digest");
    }

    #[test]
    fn resolver_never_falls_back_to_another_registry_for_a_bare_dependency() {
        let catalog = catalog(vec![
            registry_with_items("@adico", &[("button", &[])]),
            registry_with_items("@awwwkshay", &[("company-card", &["button"])]),
        ]);
        let error = catalog
            .resolve(
                &"@awwwkshay".parse().expect("valid namespace"),
                &[RegistryAddress::parse("company-card").expect("valid request")],
            )
            .expect_err("bare dependency cannot cross registry boundaries");
        assert!(
            matches!(error, RegistryError::UnknownRegistryItem { address } if address == "@awwwkshay/button")
        );
    }

    #[test]
    fn resolver_reports_missing_items_and_cross_registry_cycles() {
        let missing_catalog = catalog(vec![registry_with_items("@adico", &[])]);
        assert!(matches!(
            missing_catalog
                .resolve(
                    &"@adico".parse().expect("valid namespace"),
                    &[RegistryAddress::parse("button").expect("valid request")],
                )
                .expect_err("missing item must fail"),
            RegistryError::UnknownRegistryItem { .. }
        ));

        let cyclic_catalog = catalog(vec![
            registry_with_items("@adico", &[("button", &["@awwwkshay/card"])]),
            registry_with_items("@awwwkshay", &[("card", &["@adico/button"])]),
        ]);
        assert!(matches!(
            cyclic_catalog
                .resolve(
                    &"@adico".parse().expect("valid namespace"),
                    &[RegistryAddress::parse("button").expect("valid request")],
                )
                .expect_err("cross-registry cycle must fail"),
            RegistryError::CrossRegistryDependencyCycle { .. }
        ));
    }

    #[test]
    fn cargo_requirements_merge_features_versions_and_default_feature_policy() {
        let plan = cargo_plan(&[
            (
                "button",
                vec![cargo_requirement(
                    "dioxus",
                    None,
                    ">=0.7.0, <0.8.0",
                    &["web"],
                    false,
                    None,
                )],
            ),
            (
                "dialog",
                vec![cargo_requirement(
                    "dioxus",
                    None,
                    "^0.7.2",
                    &["desktop", "web"],
                    true,
                    None,
                )],
            ),
            (
                "web-only",
                vec![cargo_requirement(
                    "dioxus",
                    None,
                    "^0.7.2",
                    &[],
                    false,
                    Some("cfg(target_arch = \"wasm32\")"),
                )],
            ),
        ]);

        let dependencies = unify_cargo_dependencies(&plan).expect("requirements should merge");
        assert_eq!(dependencies.len(), 2, "target predicates remain distinct");
        let general = dependencies
            .iter()
            .find(|dependency| dependency.target.is_none())
            .expect("general requirement should exist");
        assert_eq!(general.features, ["desktop", "web"]);
        assert!(general.default_features);
        assert_eq!(general.origins.len(), 2);
        assert!(
            VersionReq::parse(&general.version)
                .expect("merged version is valid Cargo semver")
                .matches(&Version::parse("0.7.9").expect("valid version"))
        );
    }

    #[test]
    fn identical_cargo_requirements_remain_a_single_consumer_facing_requirement() {
        let plan = cargo_plan(&[
            (
                "button",
                vec![cargo_requirement("dioxus", None, "=0.7.9", &[], true, None)],
            ),
            (
                "dialog",
                vec![cargo_requirement("dioxus", None, "=0.7.9", &[], true, None)],
            ),
        ]);

        let dependencies = unify_cargo_dependencies(&plan).expect("requirements should merge");
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].version, "=0.7.9");
        assert_eq!(dependencies[0].origins.len(), 2);
    }

    #[test]
    fn incompatible_cargo_requirements_identify_their_registry_item_origins() {
        let plan = cargo_plan(&[
            (
                "button",
                vec![cargo_requirement("dioxus", None, "^0.7.0", &[], true, None)],
            ),
            (
                "dialog",
                vec![cargo_requirement("dioxus", None, "^1.0.0", &[], true, None)],
            ),
        ]);
        let error = unify_cargo_dependencies(&plan).expect_err("ranges cannot overlap");
        assert!(matches!(
            error,
            RegistryError::IncompatibleCargoRequirements { origins, .. }
                if origins == ["@adico/button", "@adico/dialog"]
        ));
    }

    #[test]
    fn conflicting_package_aliases_are_rejected_before_manifest_editing() {
        let plan = cargo_plan(&[
            (
                "button",
                vec![cargo_requirement(
                    "icons",
                    Some("dioxus-lucide-icons"),
                    "^0.1.0",
                    &[],
                    true,
                    None,
                )],
            ),
            (
                "dialog",
                vec![cargo_requirement(
                    "icons",
                    Some("lucide-icons"),
                    "^0.1.0",
                    &[],
                    true,
                    None,
                )],
            ),
        ]);
        assert!(matches!(
            unify_cargo_dependencies(&plan).expect_err("different alias packages must fail"),
            RegistryError::CargoPackageConflict { .. }
        ));
    }
}
