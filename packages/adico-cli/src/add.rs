//! Plan-first source installation and lockfile management for `adico add`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use adico_registry_core::{
    ComponentsConfiguration, RegistryAddress, RegistryCatalog, RegistryError, RegistryInstallPlan,
    RegistryItemAddress, ResolvedRegistryItem, TargetRoot, unify_cargo_dependencies,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::cargo::{CargoEditPlan, plan_cargo_dependency_edits};
use crate::css::{CssThemePlan, plan_theme_install};
use crate::modules::{ModuleExportRequest, ModuleUpdatePlan, plan_module_update};

/// Source-byte boundary used by local, embedded, and HTTPS registry transports.
pub trait RegistryFileReader {
    /// Returns immutable authored bytes for an already resolved source path.
    fn read(&self, item: &ResolvedRegistryItem, source: &str) -> Result<Vec<u8>, AddError>;
}

/// One registry source write that passed all checksum and conflict preconditions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFilePlan {
    /// Fully qualified item source, retained for CLI reporting.
    pub address: RegistryItemAddress,
    /// Consumer-owned destination.
    pub path: PathBuf,
    /// Exact authored source bytes.
    pub bytes: Vec<u8>,
    /// SHA-256 checksum supplied by registry metadata.
    pub checksum: String,
    expected_existing_checksum: Option<String>,
}

/// Lockfile state retained after a successful source installation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdicoLock {
    /// Lockfile format version.
    pub version: u32,
    /// Resolved registry source facts sorted by address.
    pub items: Vec<LockedItem>,
}

/// Immutable registry source facts pinned by one installed item.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedItem {
    /// Fully qualified `@namespace/item` address.
    pub address: String,
    /// SHA-256 digest of the loaded manifest bytes.
    pub manifest_digest: String,
    /// Authored file checksums keyed by consumer-relative destination.
    pub files: Vec<LockedFile>,
}

/// Installed source checksum recorded by target path.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockedFile {
    /// Consumer-project-relative destination path.
    pub path: String,
    /// SHA-256 checksum of the installed authored bytes.
    pub checksum: String,
}

/// A complete, preflighted registry-source and lockfile installation.
#[derive(Clone, Debug)]
pub struct AddPlan {
    /// Source-owned file writes. An empty collection represents an idempotent add.
    pub files: Vec<SourceFilePlan>,
    /// Project lockfile destination.
    pub lock_path: PathBuf,
    /// Complete replacement lock contents when it changed.
    pub lock_contents: Option<String>,
    source_namespaces: Vec<String>,
    lock_expected_existing_checksum: Option<String>,
}

impl AddPlan {
    /// Returns whether source files or the lockfile will change.
    pub fn has_changes(&self) -> bool {
        !self.files.is_empty() || self.lock_contents.is_some()
    }

    /// Fully-qualified registries contributing this plan, in stable order.
    pub fn source_namespaces(&self) -> Vec<String> {
        self.source_namespaces.clone()
    }

    /// Applies the plan through same-directory temporary files and atomic
    /// renames. Every destination is rechecked first. If a later rename fails,
    /// already-replaced destinations are restored from their retained backups.
    pub fn apply(&self) -> Result<(), AddError> {
        let writes = self.writes()?;
        for write in &writes {
            verify_precondition(&write.path, write.expected_existing_checksum.as_deref())?;
        }

        let mut staged = Vec::new();
        for write in &writes {
            match stage(&write.path, &write.bytes) {
                Ok(temporary) => staged.push((temporary, write)),
                Err(error) => {
                    cleanup_paths(staged.iter().map(|(temporary, _)| temporary));
                    return Err(error);
                }
            }
        }

        let mut applied = Vec::new();
        for (temporary, write) in staged {
            let backup = match backup_existing(&write.path) {
                Ok(backup) => backup,
                Err(error) => {
                    cleanup_paths(std::iter::once(&temporary));
                    rollback(&applied);
                    return Err(error);
                }
            };
            if let Err(error) = fs::rename(&temporary, &write.path) {
                if let Some(backup) = &backup {
                    let _ = fs::rename(backup, &write.path);
                }
                cleanup_paths(std::iter::once(&temporary));
                rollback(&applied);
                return Err(AddError::WriteFailed {
                    path: write.path.display().to_string(),
                    message: error.to_string(),
                });
            }
            applied.push(AppliedWrite {
                path: write.path.clone(),
                backup,
            });
        }
        cleanup_paths(applied.iter().filter_map(|write| write.backup.as_ref()));
        Ok(())
    }

    fn writes(&self) -> Result<Vec<PlannedWrite>, AddError> {
        let mut writes = self
            .files
            .iter()
            .map(|file| PlannedWrite {
                path: file.path.clone(),
                bytes: file.bytes.clone(),
                expected_existing_checksum: file.expected_existing_checksum.clone(),
            })
            .collect::<Vec<_>>();
        if let Some(lock_contents) = &self.lock_contents {
            writes.push(PlannedWrite {
                path: self.lock_path.clone(),
                bytes: lock_contents.as_bytes().to_vec(),
                expected_existing_checksum: self.lock_expected_existing_checksum.clone(),
            });
        }
        writes.sort_by(|left, right| left.path.cmp(&right.path));
        if writes.windows(2).any(|pair| pair[0].path == pair[1].path) {
            return Err(AddError::DuplicatePlannedTarget);
        }
        Ok(writes)
    }
}

/// Plans all source-owned file writes and an updated `adico.lock` before any
/// consumer-project mutation. Higher-level CLI wiring combines this with the
/// independently preflighted Cargo/module/CSS plans.
pub fn plan_source_install<R: RegistryFileReader>(
    project_root: &Path,
    configuration: &ComponentsConfiguration,
    install: &RegistryInstallPlan,
    reader: &R,
) -> Result<AddPlan, AddError> {
    let mut files = Vec::new();
    let mut locked_items = Vec::new();
    let mut source_namespaces = Vec::new();
    for resolved in &install.items {
        source_namespaces.push(resolved.address.namespace.to_string());
        let mut locked_files = Vec::new();
        for file in &resolved.item.files {
            let path = project_root
                .join(target_root_path(configuration, &file.target_root)?)
                .join(&file.target);
            let bytes = reader.read(resolved, &file.source)?;
            let actual = checksum(&bytes);
            if actual != file.checksum {
                return Err(AddError::RegistryChecksumMismatch {
                    address: resolved.address.to_string(),
                    source_path: file.source.clone(),
                    expected: file.checksum.clone(),
                    actual,
                });
            }
            let existing_checksum = existing_checksum(&path)?;
            if let Some(existing_checksum) = &existing_checksum {
                if existing_checksum != &file.checksum {
                    return Err(AddError::ModifiedConsumerFile {
                        path: path.display().to_string(),
                        address: resolved.address.to_string(),
                    });
                }
            } else {
                files.push(SourceFilePlan {
                    address: resolved.address.clone(),
                    path: path.clone(),
                    bytes,
                    checksum: file.checksum.clone(),
                    expected_existing_checksum: None,
                });
            }
            locked_files.push(LockedFile {
                path: relative_display(project_root, &path),
                checksum: file.checksum.clone(),
            });
        }
        locked_items.push(LockedItem {
            address: resolved.address.to_string(),
            manifest_digest: resolved.manifest_digest.clone(),
            files: locked_files,
        });
    }

    let lock_path = project_root.join("adico.lock");
    let lock_existing_checksum = existing_checksum(&lock_path)?;
    let mut lock = read_lock(&lock_path)?;
    for item in locked_items {
        lock.items
            .retain(|existing| existing.address != item.address);
        lock.items.push(item);
    }
    lock.items
        .sort_by(|left, right| left.address.cmp(&right.address));
    let lock_contents =
        serde_json::to_string_pretty(&lock).map_err(|error| AddError::LockSerialization {
            message: error.to_string(),
        })? + "\n";
    let existing_lock_contents = fs::read_to_string(&lock_path).ok();
    source_namespaces.sort();
    source_namespaces.dedup();
    Ok(AddPlan {
        files,
        lock_path,
        lock_contents: (existing_lock_contents.as_deref() != Some(&lock_contents))
            .then_some(lock_contents),
        source_namespaces,
        lock_expected_existing_checksum: lock_existing_checksum,
    })
}

/// Complete Button-era add plan: all mutable consumer surfaces are validated
/// before any one of them is applied.
#[derive(Clone, Debug)]
pub struct ComponentAddPlan {
    pub install: RegistryInstallPlan,
    pub source: AddPlan,
    pub cargo: CargoEditPlan,
    pub modules: Vec<ModuleUpdatePlan>,
    pub theme: Option<CssThemePlan>,
}

impl ComponentAddPlan {
    pub fn has_changes(&self) -> bool {
        self.source.has_changes()
            || self.cargo.has_changes()
            || self.modules.iter().any(ModuleUpdatePlan::has_changes)
            || self.theme.as_ref().is_some_and(CssThemePlan::has_changes)
    }

    pub fn apply(&self) -> Result<(), ComponentAddError> {
        self.cargo.apply()?;
        if let Some(theme) = &self.theme {
            theme.apply()?;
        }
        for module in &self.modules {
            module.apply()?;
        }
        self.source.apply()?;
        Ok(())
    }
}

/// Resolves named items, then validates source, Cargo, module, and theme edits
/// as one reviewable plan.
pub fn plan_component_add<R: RegistryFileReader>(
    catalog: &RegistryCatalog,
    project_root: &Path,
    manifest_path: &Path,
    configuration: &ComponentsConfiguration,
    requests: &[RegistryAddress],
    reader: &R,
) -> Result<ComponentAddPlan, ComponentAddError> {
    let install = catalog
        .resolve(&configuration.default_registry, requests)
        .map_err(|error| ComponentAddError::Registry(Box::new(error)))?;
    plan_component_install(project_root, manifest_path, configuration, install, reader)
}

/// Resolves every item in the selected default registry and validates every
/// consumer surface required by that complete install set.
pub fn plan_component_add_all<R: RegistryFileReader>(
    catalog: &RegistryCatalog,
    project_root: &Path,
    manifest_path: &Path,
    configuration: &ComponentsConfiguration,
    reader: &R,
) -> Result<ComponentAddPlan, ComponentAddError> {
    let install = catalog
        .resolve_all(&configuration.default_registry)
        .map_err(|error| ComponentAddError::Registry(Box::new(error)))?;
    plan_component_install(project_root, manifest_path, configuration, install, reader)
}

fn plan_component_install<R: RegistryFileReader>(
    project_root: &Path,
    manifest_path: &Path,
    configuration: &ComponentsConfiguration,
    install: RegistryInstallPlan,
    reader: &R,
) -> Result<ComponentAddPlan, ComponentAddError> {
    let source = plan_source_install(project_root, configuration, &install, reader)?;
    let requirements = unify_cargo_dependencies(&install)
        .map_err(|error| ComponentAddError::Registry(Box::new(error)))?;
    let cargo = plan_cargo_dependency_edits(manifest_path, &requirements)?;
    let mut exports = BTreeMap::<TargetRoot, Vec<ModuleExportRequest>>::new();
    let requires_theme = install
        .items
        .iter()
        .any(|item| item.item.style.semantic_tokens || item.item.style.radius_token);
    for item in &install.items {
        for export in &item.item.module_exports {
            exports
                .entry(export.target_root.clone())
                .or_default()
                .push(ModuleExportRequest {
                    module: export.module.clone(),
                    reexport: export.reexport,
                });
        }
    }
    let mut modules = Vec::new();
    for (root, requests) in exports {
        let path = project_root
            .join(target_root_path(configuration, &root)?)
            .join("mod.rs");
        modules.push(plan_module_update(path, &requests)?);
    }
    let theme = requires_theme
        .then(|| plan_theme_install(project_root.join(&configuration.css.entry)))
        .transpose()?;
    Ok(ComponentAddPlan {
        install,
        source,
        cargo,
        modules,
        theme,
    })
}

/// A deterministic `adico add --all` plan. The resolved install sequence is
/// retained for review/output, while its source writes use the same preflight
/// and transactional apply behavior as explicitly requested items.
#[derive(Clone, Debug)]
pub struct AddAllPlan {
    /// Dependency-first resolved items from the selected default registry.
    pub install: RegistryInstallPlan,
    /// Source and lockfile writes for the resolved item set.
    pub source: AddPlan,
}

impl AddAllPlan {
    /// Returns whether applying every selected item changes the project.
    pub fn has_changes(&self) -> bool {
        self.source.has_changes()
    }

    /// Applies every source and lockfile change through the shared installer.
    pub fn apply(&self) -> Result<(), AddError> {
        self.source.apply()
    }
}

/// Resolves every item from the configured default registry and preflights it
/// through the same source installer used by `adico add <component...>`.
pub fn plan_add_all<R: RegistryFileReader>(
    catalog: &RegistryCatalog,
    project_root: &Path,
    configuration: &ComponentsConfiguration,
    reader: &R,
) -> Result<AddAllPlan, AddAllError> {
    let install = catalog
        .resolve_all(&configuration.default_registry)
        .map_err(|error| AddAllError::Registry(Box::new(error)))?;
    let source = plan_source_install(project_root, configuration, &install, reader)?;
    Ok(AddAllPlan { install, source })
}

#[derive(Clone, Debug)]
struct PlannedWrite {
    path: PathBuf,
    bytes: Vec<u8>,
    expected_existing_checksum: Option<String>,
}

#[derive(Clone, Debug)]
struct AppliedWrite {
    path: PathBuf,
    backup: Option<PathBuf>,
}

fn target_root_path<'a>(
    configuration: &'a ComponentsConfiguration,
    root: &TargetRoot,
) -> Result<&'a str, AddError> {
    match root {
        TargetRoot::Ui => Ok(&configuration.paths.ui),
        TargetRoot::Components => Ok(&configuration.paths.components),
        TargetRoot::Lib => Ok(&configuration.paths.lib),
        TargetRoot::Hooks => Ok(&configuration.paths.hooks),
        TargetRoot::Css => Err(AddError::UnsupportedCssFileTarget),
    }
}

fn read_lock(path: &Path) -> Result<AdicoLock, AddError> {
    if !path.try_exists().map_err(read_error(path))? {
        return Ok(AdicoLock {
            version: 1,
            items: Vec::new(),
        });
    }
    serde_json::from_str(&fs::read_to_string(path).map_err(read_error(path))?).map_err(|error| {
        AddError::MalformedLock {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    })
}

fn existing_checksum(path: &Path) -> Result<Option<String>, AddError> {
    if !path.try_exists().map_err(read_error(path))? {
        return Ok(None);
    }
    Ok(Some(checksum(&fs::read(path).map_err(read_error(path))?)))
}

fn verify_precondition(path: &Path, expected: Option<&str>) -> Result<(), AddError> {
    let actual = existing_checksum(path)?;
    if actual.as_deref() != expected {
        return Err(AddError::ConsumerPathChangedDuringApply {
            path: path.display().to_string(),
        });
    }
    Ok(())
}

fn stage(path: &Path, bytes: &[u8]) -> Result<PathBuf, AddError> {
    let parent = path.parent().ok_or_else(|| AddError::WriteFailed {
        path: path.display().to_string(),
        message: "destination has no parent directory".to_string(),
    })?;
    fs::create_dir_all(parent).map_err(write_error(parent))?;
    let temporary = temporary_path(path, "stage");
    fs::write(&temporary, bytes).map_err(write_error(&temporary))?;
    Ok(temporary)
}

fn backup_existing(path: &Path) -> Result<Option<PathBuf>, AddError> {
    if !path.try_exists().map_err(read_error(path))? {
        return Ok(None);
    }
    let backup = temporary_path(path, "backup");
    fs::rename(path, &backup).map_err(write_error(path))?;
    Ok(Some(backup))
}

fn rollback(applied: &[AppliedWrite]) {
    for write in applied.iter().rev() {
        let _ = fs::remove_file(&write.path);
        if let Some(backup) = &write.backup {
            let _ = fs::rename(backup, &write.path);
        }
    }
}

fn cleanup_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

fn temporary_path(path: &Path, kind: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    path.with_file_name(format!(
        ".adico-{kind}-{}-{nonce}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file")
    ))
}

fn relative_display(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_error(path: &Path) -> impl FnOnce(std::io::Error) -> AddError + '_ {
    move |error| AddError::ReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

fn write_error(path: &Path) -> impl FnOnce(std::io::Error) -> AddError + '_ {
    move |error| AddError::WriteFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

/// Preflight or transactional-apply failures that preserve consumer ownership.
#[derive(Debug, Error)]
pub enum AddError {
    /// Consumer path could not be read.
    #[error("cannot read {path}: {message}")]
    ReadFailed { path: String, message: String },
    /// Registry source did not match its signed/recorded source checksum.
    #[error(
        "registry source for {address} returned a checksum mismatch for {source_path}: expected {expected}, found {actual}"
    )]
    RegistryChecksumMismatch {
        address: String,
        source_path: String,
        expected: String,
        actual: String,
    },
    /// Existing consumer source is not identical to the source being installed.
    #[error("refusing to overwrite user-modified installed file {path} from {address}")]
    ModifiedConsumerFile { path: String, address: String },
    /// CSS source-target files require a dedicated future installer.
    #[error("registry:file CSS targets are not supported by source installation yet")]
    UnsupportedCssFileTarget,
    /// Existing lockfile was not parseable.
    #[error("adico lockfile {path} is malformed: {message}")]
    MalformedLock { path: String, message: String },
    /// Lockfile serialization unexpectedly failed.
    #[error("cannot serialize adico.lock: {message}")]
    LockSerialization { message: String },
    /// A target changed after planning, before mutation could start.
    #[error("consumer path changed after adico add was planned: {path}")]
    ConsumerPathChangedDuringApply { path: String },
    /// Multiple planned writes targeted one consumer file.
    #[error("adico add produced duplicate target writes")]
    DuplicatePlannedTarget,
    /// A staged or final consumer write failed.
    #[error("cannot write {path}: {message}")]
    WriteFailed { path: String, message: String },
}

/// `--all` planning failures retain either registry-resolution or source-file
/// context without performing a consumer-project mutation.
#[derive(Debug, Error)]
pub enum AddAllError {
    /// The selected registry could not be resolved.
    #[error(transparent)]
    Registry(Box<RegistryError>),
    /// Source-install preflight failed.
    #[error(transparent)]
    Source(#[from] AddError),
}

#[derive(Debug, Error)]
pub enum ComponentAddError {
    #[error(transparent)]
    Registry(Box<RegistryError>),
    #[error(transparent)]
    Source(#[from] AddError),
    #[error(transparent)]
    Cargo(#[from] crate::cargo::CargoEditError),
    #[error(transparent)]
    Module(#[from] crate::modules::ModuleError),
    #[error(transparent)]
    Theme(#[from] crate::css::CssThemeError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use adico_registry_core::{
        ComponentPaths, EmbeddedRegistry, RegistryItem, RegistryItemType, RegistryLocation,
        RegistryNamespace, RegistrySource, RegistrySourceLoader, StyleRequirements,
    };
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMPORARY_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[derive(Default)]
    struct FixtureReader {
        files: BTreeMap<(String, String), Vec<u8>>,
    }

    impl RegistryFileReader for FixtureReader {
        fn read(&self, item: &ResolvedRegistryItem, source: &str) -> Result<Vec<u8>, AddError> {
            self.files
                .get(&(item.address.to_string(), source.to_string()))
                .cloned()
                .ok_or_else(|| AddError::ReadFailed {
                    path: source.to_string(),
                    message: "fixture source is absent".to_string(),
                })
        }
    }

    fn temporary_project() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid system time")
            .as_nanos();
        let sequence = TEMPORARY_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "adico-add-test-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary project should be created");
        root
    }

    fn configuration() -> ComponentsConfiguration {
        ComponentsConfiguration {
            schema: Some("https://adico.dev/schema/components.json/v1".to_string()),
            version: 1,
            style: "default".to_string(),
            theme: adico_registry_core::ThemeConfiguration {
                tokens: "shadcn".to_string(),
                dark_mode: "class".to_string(),
            },
            paths: ComponentPaths {
                components: "src/components".to_string(),
                ui: "src/components/ui".to_string(),
                lib: "src/adico_lib".to_string(),
                hooks: "src/hooks".to_string(),
            },
            css: adico_registry_core::CssConfiguration {
                entry: "assets/tailwind.css".to_string(),
                framework: "tailwind".to_string(),
            },
            registries: BTreeMap::from([(
                "@adico".parse().expect("valid namespace"),
                adico_registry_core::RegistrySource::Embedded,
            )]),
            default_registry: "@adico".parse().expect("valid namespace"),
        }
    }

    fn install(namespace: &str, item: &str, source: &str, bytes: &[u8]) -> RegistryInstallPlan {
        let namespace = namespace
            .parse::<RegistryNamespace>()
            .expect("valid namespace");
        let address = RegistryItemAddress {
            namespace,
            item: item.to_string(),
        };
        let digest = checksum(bytes);
        RegistryInstallPlan {
            requested: vec![address.clone()],
            items: vec![ResolvedRegistryItem {
                address,
                item: RegistryItem {
                    name: item.to_string(),
                    item_type: RegistryItemType::Ui,
                    description: "fixture".to_string(),
                    files: vec![adico_registry_core::RegistryFile {
                        source: source.to_string(),
                        target_root: TargetRoot::Ui,
                        target: format!("{item}.rs"),
                        checksum: digest,
                    }],
                    registry_dependencies: Vec::new(),
                    cargo_dependencies: Vec::new(),
                    style: StyleRequirements::default(),
                    module_exports: Vec::new(),
                    documentation: None,
                    compatibility: None,
                    provenance: None,
                },
                location: RegistryLocation::Embedded {
                    label: "fixture".to_string(),
                    source_root: PathBuf::new(),
                },
                manifest_digest: "fixture-manifest-digest".to_string(),
                registry_compatibility: adico_registry_core::RegistryCompatibility {
                    cli: ">=0.1.0".to_string(),
                    runtime: None,
                },
            }],
        }
    }

    fn company_catalog(root: &Path) -> (RegistryCatalog, FixtureReader) {
        let registry_root = root.join("company-registry");
        let source_root = registry_root.join("ui");
        fs::create_dir_all(&source_root).expect("registry source directory should be created");
        let sources = [
            ("utility", b"pub fn utility() {}\n".as_slice()),
            ("button", b"pub fn button() {}\n".as_slice()),
            ("card", b"pub fn card() {}\n".as_slice()),
        ];
        for (name, bytes) in &sources {
            fs::write(source_root.join(format!("{name}.rs")), bytes)
                .expect("registry source should be written");
        }
        let manifest = format!(
            r#"{{
  "formatVersion": 1,
  "namespace": "@awwwkshay",
  "name": "Awwwkshay fixtures",
  "compatibility": {{ "cli": ">=0.1.0" }},
  "items": [
    {{ "name": "utility", "type": "registry:lib", "description": "utility", "files": [{{ "source": "ui/utility.rs", "targetRoot": "lib", "target": "utility.rs", "checksum": "{}" }}] }},
    {{ "name": "button", "type": "registry:ui", "description": "button", "registryDependencies": ["utility"], "files": [{{ "source": "ui/button.rs", "targetRoot": "ui", "target": "button.rs", "checksum": "{}" }}] }},
    {{ "name": "card", "type": "registry:ui", "description": "card", "registryDependencies": ["utility"], "files": [{{ "source": "ui/card.rs", "targetRoot": "ui", "target": "card.rs", "checksum": "{}" }}] }}
  ]
}}"#,
            checksum(sources[0].1),
            checksum(sources[1].1),
            checksum(sources[2].1),
        );
        fs::write(registry_root.join("registry.json"), manifest)
            .expect("registry manifest should be written");
        let embedded_manifest = br#"{
            "formatVersion": 1,
            "namespace": "@adico",
            "name": "embedded fixture",
            "compatibility": { "cli": ">=0.1.0" },
            "items": []
        }"#;
        let loader =
            RegistrySourceLoader::new(EmbeddedRegistry::new(embedded_manifest, &registry_root));
        let namespace: RegistryNamespace = "@awwwkshay".parse().expect("valid namespace");
        let loaded = loader
            .load(
                &namespace,
                &RegistrySource::Local {
                    path: registry_root.display().to_string(),
                },
            )
            .expect("company registry should load");
        let mut catalog = RegistryCatalog::new();
        catalog
            .insert(loaded)
            .expect("catalog should accept registry");
        let mut reader = FixtureReader::default();
        for (name, bytes) in sources {
            reader.files.insert(
                (format!("@awwwkshay/{name}"), format!("ui/{name}.rs")),
                bytes.to_vec(),
            );
        }
        (catalog, reader)
    }

    #[test]
    fn installs_source_and_lock_with_namespace_provenance_idempotently() {
        let root = temporary_project();
        let bytes = b"pub fn button() {}\n";
        let install = install("@awwwkshay", "button", "ui/button.rs", bytes);
        let mut reader = FixtureReader::default();
        reader.files.insert(
            ("@awwwkshay/button".to_string(), "ui/button.rs".to_string()),
            bytes.to_vec(),
        );

        let plan = plan_source_install(&root, &configuration(), &install, &reader)
            .expect("install should plan");
        assert_eq!(plan.source_namespaces(), vec!["@awwwkshay"]);
        assert!(plan.has_changes());
        plan.apply().expect("plan should apply");
        assert_eq!(
            fs::read(root.join("src/components/ui/button.rs")).expect("source should install"),
            bytes
        );
        let lock: AdicoLock = serde_json::from_str(
            &fs::read_to_string(root.join("adico.lock")).expect("lock should install"),
        )
        .expect("lock should parse");
        assert_eq!(lock.items[0].address, "@awwwkshay/button");
        assert_eq!(lock.items[0].manifest_digest, "fixture-manifest-digest");
        assert!(
            !plan_source_install(&root, &configuration(), &install, &reader)
                .expect("repeat should plan")
                .has_changes()
        );
        fs::remove_dir_all(root).expect("temporary project should be removable");
    }

    #[test]
    fn modified_source_and_invalid_registry_bytes_fail_before_writing() {
        let root = temporary_project();
        let bytes = b"pub fn dialog() {}\n";
        let install = install("@adico", "dialog", "ui/dialog.rs", bytes);
        let mut reader = FixtureReader::default();
        reader.files.insert(
            ("@adico/dialog".to_string(), "ui/dialog.rs".to_string()),
            bytes.to_vec(),
        );
        let target = root.join("src/components/ui/dialog.rs");
        fs::create_dir_all(target.parent().expect("target parent should exist"))
            .expect("target parent should be created");
        fs::write(&target, "consumer changes\n").expect("consumer source should be written");
        let before = fs::read(&target).expect("source should be readable");
        assert!(matches!(
            plan_source_install(&root, &configuration(), &install, &reader),
            Err(AddError::ModifiedConsumerFile { .. })
        ));
        assert_eq!(
            fs::read(&target).expect("source must remain untouched"),
            before
        );
        assert!(!root.join("adico.lock").exists());

        fs::remove_file(&target).expect("fixture source should be removable");
        reader.files.insert(
            ("@adico/dialog".to_string(), "ui/dialog.rs".to_string()),
            b"incorrect registry source\n".to_vec(),
        );
        assert!(matches!(
            plan_source_install(&root, &configuration(), &install, &reader),
            Err(AddError::RegistryChecksumMismatch { .. })
        ));
        assert!(!root.join("adico.lock").exists());
        fs::remove_dir_all(root).expect("temporary project should be removable");
    }

    #[test]
    fn apply_refuses_a_target_created_after_preflight() {
        let root = temporary_project();
        let bytes = b"pub fn select() {}\n";
        let install = install("@adico", "select", "ui/select.rs", bytes);
        let mut reader = FixtureReader::default();
        reader.files.insert(
            ("@adico/select".to_string(), "ui/select.rs".to_string()),
            bytes.to_vec(),
        );
        let plan = plan_source_install(&root, &configuration(), &install, &reader)
            .expect("install should plan");
        let target = root.join("src/components/ui/select.rs");
        fs::create_dir_all(target.parent().expect("target parent should exist"))
            .expect("target parent should be created");
        fs::write(&target, "consumer added this after planning\n")
            .expect("consumer source should be written");
        assert!(matches!(
            plan.apply(),
            Err(AddError::ConsumerPathChangedDuringApply { .. })
        ));
        assert_eq!(
            fs::read_to_string(&target).expect("consumer source should remain"),
            "consumer added this after planning\n"
        );
        assert!(!root.join("adico.lock").exists());
        fs::remove_dir_all(root).expect("temporary project should be removable");
    }

    #[test]
    fn all_uses_the_same_install_path_for_every_local_registry_item_once() {
        let root = temporary_project();
        let (catalog, reader) = company_catalog(&root);
        let mut configuration = configuration();
        configuration.default_registry = "@awwwkshay".parse().expect("valid namespace");
        configuration.registries = BTreeMap::from([(
            configuration.default_registry.clone(),
            RegistrySource::Local {
                path: "company-registry".to_string(),
            },
        )]);
        let plan = plan_add_all(&catalog, &root, &configuration, &reader)
            .expect("all company items should plan");
        assert_eq!(
            plan.install
                .items
                .iter()
                .map(|item| item.address.to_string())
                .collect::<Vec<_>>(),
            ["@awwwkshay/utility", "@awwwkshay/button", "@awwwkshay/card"]
        );
        plan.apply().expect("all company sources should install");
        assert!(root.join("src/adico_lib/utility.rs").is_file());
        assert!(root.join("src/components/ui/button.rs").is_file());
        assert!(root.join("src/components/ui/card.rs").is_file());
        assert!(
            !plan_add_all(&catalog, &root, &configuration, &reader)
                .expect("repeat all should plan")
                .has_changes()
        );
        fs::remove_dir_all(root).expect("temporary project should be removable");
    }
}
