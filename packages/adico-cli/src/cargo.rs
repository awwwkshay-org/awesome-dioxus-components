//! Structured, conflict-safe `Cargo.toml` dependency editing.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use adico_registry_core::UnifiedCargoDependency;
use thiserror::Error;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value, value};

/// A reviewable edit to one Cargo manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CargoEditPlan {
    /// Consumer package manifest to modify.
    pub manifest_path: PathBuf,
    /// Serialized manifest when a write is necessary.
    pub contents: Option<String>,
}

impl CargoEditPlan {
    /// Returns whether this plan writes Cargo.toml.
    pub fn has_changes(&self) -> bool {
        self.contents.is_some()
    }

    /// Applies the planned complete manifest replacement.
    pub fn apply(&self) -> Result<(), CargoEditError> {
        if let Some(contents) = &self.contents {
            fs::write(&self.manifest_path, contents).map_err(|error| {
                CargoEditError::WriteFailed {
                    path: self.manifest_path.display().to_string(),
                    message: error.to_string(),
                }
            })?;
        }
        Ok(())
    }
}

/// Plans Cargo dependency edits for one selected consumer package manifest.
pub fn plan_cargo_dependency_edits(
    manifest_path: impl Into<PathBuf>,
    dependencies: &[UnifiedCargoDependency],
) -> Result<CargoEditPlan, CargoEditError> {
    let manifest_path = manifest_path.into();
    let original =
        fs::read_to_string(&manifest_path).map_err(|error| CargoEditError::ReadFailed {
            path: manifest_path.display().to_string(),
            message: error.to_string(),
        })?;
    let mut document =
        original
            .parse::<DocumentMut>()
            .map_err(|error| CargoEditError::MalformedManifest {
                path: manifest_path.display().to_string(),
                message: error.to_string(),
            })?;
    for dependency in dependencies {
        plan_one_dependency(&mut document, dependency)?;
    }
    let updated = document.to_string();
    Ok(CargoEditPlan {
        manifest_path,
        contents: (updated != original).then_some(updated),
    })
}

fn plan_one_dependency(
    document: &mut DocumentMut,
    dependency: &UnifiedCargoDependency,
) -> Result<(), CargoEditError> {
    let table = dependency_table(document, dependency)?;
    match table.get(&dependency.crate_name) {
        None => {
            table[&dependency.crate_name] = dependency_item(dependency);
            Ok(())
        }
        Some(existing) if is_workspace_inheritance(existing) => {
            let workspace = document
                .get("workspace")
                .and_then(Item::as_table)
                .and_then(|workspace| workspace.get("dependencies"))
                .and_then(Item::as_table)
                .ok_or_else(|| CargoEditError::AmbiguousWorkspaceDependency {
                    crate_name: dependency.crate_name.clone(),
                })?;
            let inherited = workspace.get(&dependency.crate_name).ok_or_else(|| {
                CargoEditError::AmbiguousWorkspaceDependency {
                    crate_name: dependency.crate_name.clone(),
                }
            })?;
            verify_existing_dependency(inherited, dependency)
        }
        Some(existing) => verify_existing_dependency(existing, dependency),
    }
}

fn dependency_table<'a>(
    document: &'a mut DocumentMut,
    dependency: &UnifiedCargoDependency,
) -> Result<&'a mut Table, CargoEditError> {
    match &dependency.target {
        None => {
            if !document.as_table().contains_key("dependencies") {
                document["dependencies"] = Item::Table(Table::new());
            }
            document["dependencies"].as_table_mut().ok_or_else(|| {
                CargoEditError::UnsupportedDependencyTable {
                    crate_name: dependency.crate_name.clone(),
                    target: None,
                }
            })
        }
        Some(target) => {
            if !document.as_table().contains_key("target") {
                document["target"] = Item::Table(Table::new());
            }
            let target_table = document["target"].as_table_mut().ok_or_else(|| {
                CargoEditError::UnsupportedDependencyTable {
                    crate_name: dependency.crate_name.clone(),
                    target: Some(target.clone()),
                }
            })?;
            if !target_table.contains_key(target) {
                target_table[target] = Item::Table(Table::new());
            }
            let target_entry = target_table[target].as_table_mut().ok_or_else(|| {
                CargoEditError::UnsupportedDependencyTable {
                    crate_name: dependency.crate_name.clone(),
                    target: Some(target.clone()),
                }
            })?;
            if !target_entry.contains_key("dependencies") {
                target_entry["dependencies"] = Item::Table(Table::new());
            }
            target_entry["dependencies"].as_table_mut().ok_or_else(|| {
                CargoEditError::UnsupportedDependencyTable {
                    crate_name: dependency.crate_name.clone(),
                    target: Some(target.clone()),
                }
            })
        }
    }
}

fn dependency_item(dependency: &UnifiedCargoDependency) -> Item {
    if dependency.package.is_none()
        && dependency.features.is_empty()
        && dependency.default_features
        && dependency.target.is_none()
    {
        return value(&dependency.version);
    }
    let mut table = InlineTable::new();
    table.insert("version", Value::from(dependency.version.as_str()));
    if let Some(package) = &dependency.package {
        table.insert("package", Value::from(package.as_str()));
    }
    if !dependency.features.is_empty() {
        let mut features = Array::new();
        for feature in &dependency.features {
            features.push(feature.as_str());
        }
        table.insert("features", Value::Array(features));
    }
    if !dependency.default_features {
        table.insert("default-features", Value::from(false));
    }
    Item::Value(Value::InlineTable(table))
}

fn is_workspace_inheritance(item: &Item) -> bool {
    item.as_inline_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        == Some(true)
}

fn verify_existing_dependency(
    existing: &Item,
    requested: &UnifiedCargoDependency,
) -> Result<(), CargoEditError> {
    let existing = dependency_shape(existing).ok_or_else(|| {
        CargoEditError::UnsupportedExistingDependency {
            crate_name: requested.crate_name.clone(),
        }
    })?;
    let requested_features = requested.features.iter().cloned().collect::<BTreeSet<_>>();
    // A consumer may deliberately enable additional features or disable
    // defaults on a dependency already needed by a registry item. Registry
    // items name every feature they require explicitly, so only a missing
    // requested feature is incompatible.
    if existing.version.as_deref() != Some(requested.version.as_str())
        || existing.package != requested.package
        || !requested_features.is_subset(&existing.features)
    {
        return Err(CargoEditError::DependencyConflict {
            crate_name: requested.crate_name.clone(),
            existing_version: existing.version,
            requested_version: requested.version.clone(),
            origins: requested.origins.iter().map(ToString::to_string).collect(),
        });
    }
    Ok(())
}

#[derive(Debug)]
struct DependencyShape {
    version: Option<String>,
    package: Option<String>,
    features: BTreeSet<String>,
}

fn dependency_shape(item: &Item) -> Option<DependencyShape> {
    if let Some(version) = item.as_str() {
        return Some(DependencyShape {
            version: Some(version.to_string()),
            package: None,
            features: BTreeSet::new(),
        });
    }
    let table = item.as_inline_table()?;
    let version = table
        .get("version")
        .and_then(Value::as_str)
        .map(str::to_string);
    let package = table
        .get("package")
        .and_then(Value::as_str)
        .map(str::to_string);
    let features = table
        .get("features")
        .and_then(Value::as_array)
        .map(|features| {
            features
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    Some(DependencyShape {
        version,
        package,
        features,
    })
}

/// Structured Cargo-manifest planning errors. No error returns a partial plan.
#[derive(Debug, Error)]
pub enum CargoEditError {
    /// Manifest could not be read.
    #[error("cannot read Cargo manifest {path}: {message}")]
    ReadFailed { path: String, message: String },
    /// Manifest was not valid TOML.
    #[error("Cargo manifest {path} is malformed: {message}")]
    MalformedManifest { path: String, message: String },
    /// Existing manifest table is not a normal dependency table.
    #[error("Cargo dependency table for {crate_name:?} target {target:?} is unsupported")]
    UnsupportedDependencyTable {
        crate_name: String,
        target: Option<String>,
    },
    /// Existing dependency is in a TOML shape adico cannot safely reconcile.
    #[error("existing Cargo dependency {crate_name:?} is not a supported string or inline table")]
    UnsupportedExistingDependency { crate_name: String },
    /// Workspace inheritance cannot be mapped to a same-manifest workspace dependency.
    #[error(
        "workspace dependency {crate_name:?} is ambiguous; adico will not edit another manifest implicitly"
    )]
    AmbiguousWorkspaceDependency { crate_name: String },
    /// Existing dependency requirements would change copied-source behavior.
    #[error("Cargo dependency {crate_name:?} conflicts: existing {existing_version:?}, requested {requested_version:?} (from {})", .origins.join(", "))]
    DependencyConflict {
        crate_name: String,
        existing_version: Option<String>,
        requested_version: String,
        origins: Vec<String>,
    },
    /// Planned manifest could not be written.
    #[error("cannot write Cargo manifest {path}: {message}")]
    WriteFailed { path: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use adico_registry_core::RegistryItemAddress;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMPORARY_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temporary_manifest(contents: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        let sequence = TEMPORARY_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "adico-cargo-test-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary directory should be created");
        let path = root.join("Cargo.toml");
        fs::write(&path, contents).expect("temporary manifest should be written");
        path
    }

    fn dependency(name: &str, version: &str) -> UnifiedCargoDependency {
        UnifiedCargoDependency {
            crate_name: name.to_string(),
            package: None,
            version: version.to_string(),
            features: Vec::new(),
            default_features: true,
            target: None,
            origins: vec![RegistryItemAddress {
                namespace: "@adico".parse().expect("valid namespace"),
                item: "fixture".to_string(),
            }],
        }
    }

    #[test]
    fn adds_dependencies_without_reformatting_existing_comments() {
        let path = temporary_manifest(
            "[dependencies]\n# keep this comment\ndioxus = \"=0.7.9\" # preserve this too\n",
        );
        let plan = plan_cargo_dependency_edits(
            &path,
            &[
                dependency("dioxus", "=0.7.9"),
                dependency("adico-primitives", "^0.1.0"),
            ],
        )
        .expect("compatible dependencies should plan");
        let contents = plan
            .contents
            .as_deref()
            .expect("dependency should be added");
        assert!(contents.contains("# keep this comment\ndioxus = \"=0.7.9\" # preserve this too"));
        assert!(contents.contains("adico-primitives = \"^0.1.0\""));
        plan.apply().expect("plan should apply");
        fs::remove_dir_all(path.parent().expect("temporary root should exist"))
            .expect("temporary directory should be removable");
    }

    #[test]
    fn accepts_a_consumer_dependency_with_additional_features() {
        let path = temporary_manifest(
            "[dependencies]\ndioxus = { version = \"=0.7.9\", features = [\"web\", \"router\"] }\n",
        );
        let plan = plan_cargo_dependency_edits(&path, &[dependency("dioxus", "=0.7.9")])
            .expect("extra consumer features should remain compatible");
        assert!(!plan.has_changes());
        fs::remove_dir_all(path.parent().expect("temporary root should exist"))
            .expect("temporary directory should be removable");
    }

    #[test]
    fn conflicts_and_ambiguous_workspace_dependencies_produce_no_edit_plan() {
        let path = temporary_manifest("[dependencies]\ndioxus = \"=0.7.9\"\n");
        assert!(matches!(
            plan_cargo_dependency_edits(&path, &[dependency("dioxus", "^1.0.0")])
                .expect_err("version conflict must fail"),
            CargoEditError::DependencyConflict { .. }
        ));
        fs::remove_dir_all(path.parent().expect("temporary root should exist"))
            .expect("temporary directory should be removable");
        let path = temporary_manifest("[dependencies]\ndioxus = { workspace = true }\n");
        assert!(matches!(
            plan_cargo_dependency_edits(&path, &[dependency("dioxus", "=0.7.9")])
                .expect_err("external workspace manifest must not be guessed"),
            CargoEditError::AmbiguousWorkspaceDependency { .. }
        ));
        fs::remove_dir_all(path.parent().expect("temporary root should exist"))
            .expect("temporary directory should be removable");
    }

    #[test]
    fn same_manifest_workspace_dependency_is_verified_without_rewrite() {
        let path = temporary_manifest(
            "[workspace]\n\n[workspace.dependencies]\ndioxus = \"=0.7.9\" # comment\n\n[dependencies]\ndioxus = { workspace = true }\n",
        );
        let plan = plan_cargo_dependency_edits(&path, &[dependency("dioxus", "=0.7.9")])
            .expect("workspace dependency should verify");
        assert!(!plan.has_changes());
        fs::remove_dir_all(path.parent().expect("temporary root should exist"))
            .expect("temporary directory should be removable");
    }
}
