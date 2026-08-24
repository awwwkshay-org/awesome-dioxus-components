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
    /// Returns the SHA-256 digest of the exact manifest bytes that were loaded.
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }
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
        match &registry.location {
            RegistryLocation::Embedded {
                source_root, label, ..
            } => read_local_source(source_root, relative_path, label),
            RegistryLocation::Local { source_root, .. } => {
                read_local_source(source_root, relative_path, &registry.location.to_string())
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
}
