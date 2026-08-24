//! Dioxus consumer-project discovery for `adico init` and `adico add`.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use thiserror::Error;

/// A Dioxus package selected safely from Cargo metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DioxusProject {
    /// Nearest Cargo manifest found from the invocation directory.
    pub invocation_manifest_path: PathBuf,
    /// Manifest belonging to the selected Dioxus package.
    pub package_manifest_path: PathBuf,
    /// Cargo package identifier.
    pub package_id: String,
    /// Human-readable Cargo package name.
    pub package_name: String,
    /// Existing Dioxus Rust entrypoint used by the selected package.
    pub entrypoint: PathBuf,
}

/// Finds the nearest Cargo manifest, executes offline `cargo metadata`, and
/// selects one unambiguous Dioxus package with a normal Rust entrypoint.
pub fn discover_dioxus_project(start: &Path) -> Result<DioxusProject, ProjectDiscoveryError> {
    let invocation_directory = invocation_directory(start)?;
    let invocation_manifest_path = nearest_manifest(&invocation_directory)?;
    let metadata = cargo_metadata(&invocation_manifest_path)?;
    select_dioxus_project(&invocation_directory, &invocation_manifest_path, &metadata)
}

fn invocation_directory(start: &Path) -> Result<PathBuf, ProjectDiscoveryError> {
    let start = fs::canonicalize(start).map_err(|error| ProjectDiscoveryError::UnreadablePath {
        path: start.display().to_string(),
        message: error.to_string(),
    })?;
    Ok(if start.is_file() {
        start
            .parent()
            .expect("files have parent directories")
            .to_path_buf()
    } else {
        start
    })
}

fn nearest_manifest(start: &Path) -> Result<PathBuf, ProjectDiscoveryError> {
    for directory in start.ancestors() {
        let manifest = directory.join("Cargo.toml");
        if manifest.is_file() {
            return Ok(manifest);
        }
    }
    Err(ProjectDiscoveryError::NoCargoManifest {
        start: start.display().to_string(),
    })
}

fn cargo_metadata(manifest_path: &Path) -> Result<CargoMetadata, ProjectDiscoveryError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(manifest_path)
        .output()
        .map_err(|error| ProjectDiscoveryError::CargoMetadataFailed {
            manifest: manifest_path.display().to_string(),
            message: error.to_string(),
        })?;
    if !output.status.success() {
        return Err(ProjectDiscoveryError::CargoMetadataFailed {
            manifest: manifest_path.display().to_string(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|error| {
        ProjectDiscoveryError::CargoMetadataFailed {
            manifest: manifest_path.display().to_string(),
            message: format!("Cargo returned invalid metadata JSON: {error}"),
        }
    })
}

fn select_dioxus_project(
    invocation_directory: &Path,
    invocation_manifest_path: &Path,
    metadata: &CargoMetadata,
) -> Result<DioxusProject, ProjectDiscoveryError> {
    let workspace_members: BTreeSet<_> = metadata.workspace_members.iter().collect();
    let mut candidates: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| workspace_members.contains(&package.id) && package.depends_on_dioxus())
        .collect();
    candidates.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));

    let selected = if candidates.is_empty() {
        return Err(ProjectDiscoveryError::NoDioxusPackage {
            manifest: invocation_manifest_path.display().to_string(),
        });
    } else if candidates.len() == 1 {
        candidates[0]
    } else if let Some(candidate) = candidates
        .iter()
        .copied()
        .find(|candidate| Path::new(&candidate.manifest_path) == invocation_manifest_path)
    {
        candidate
    } else {
        let containing: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|candidate| {
                Path::new(&candidate.manifest_path)
                    .parent()
                    .is_some_and(|directory| invocation_directory.starts_with(directory))
            })
            .collect();
        if containing.len() == 1 {
            containing[0]
        } else {
            return Err(ProjectDiscoveryError::AmbiguousDioxusPackages {
                manifest: invocation_manifest_path.display().to_string(),
                packages: candidates
                    .into_iter()
                    .map(|candidate| candidate.manifest_path.clone())
                    .collect(),
            });
        }
    };

    let package_manifest_path = PathBuf::from(&selected.manifest_path);
    let package_root = package_manifest_path
        .parent()
        .expect("Cargo manifests always have a parent directory");
    let entrypoint = [
        package_root.join("src/main.rs"),
        package_root.join("src/lib.rs"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .ok_or_else(|| ProjectDiscoveryError::MissingEntrypoint {
        package: selected.name.clone(),
        manifest: selected.manifest_path.clone(),
    })?;
    Ok(DioxusProject {
        invocation_manifest_path: invocation_manifest_path.to_path_buf(),
        package_manifest_path,
        package_id: selected.id.clone(),
        package_name: selected.name.clone(),
        entrypoint,
    })
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    manifest_path: String,
    dependencies: Vec<CargoDependencyMetadata>,
}

impl CargoPackage {
    fn depends_on_dioxus(&self) -> bool {
        self.dependencies.iter().any(|dependency| {
            dependency.name == "dioxus" || dependency.rename.as_deref() == Some("dioxus")
        })
    }
}

#[derive(Debug, Deserialize)]
struct CargoDependencyMetadata {
    name: String,
    rename: Option<String>,
}

/// Discovery failures that leave consumer projects untouched.
#[derive(Debug, Error, PartialEq)]
pub enum ProjectDiscoveryError {
    /// Invocation path could not be inspected.
    #[error("cannot inspect {path}: {message}")]
    UnreadablePath {
        /// Input path.
        path: String,
        /// Filesystem error.
        message: String,
    },
    /// No ancestor declares a Cargo project.
    #[error("no Cargo.toml found from {start} upward")]
    NoCargoManifest {
        /// Canonical invocation directory.
        start: String,
    },
    /// Cargo metadata was unavailable or malformed.
    #[error("cannot inspect Cargo metadata for {manifest}: {message}")]
    CargoMetadataFailed {
        /// Nearest manifest path.
        manifest: String,
        /// Cargo/JSON error.
        message: String,
    },
    /// No workspace package declares Dioxus.
    #[error("no Dioxus package found from {manifest}")]
    NoDioxusPackage {
        /// Nearest manifest path.
        manifest: String,
    },
    /// More than one package could reasonably be selected.
    #[error("multiple Dioxus packages found from {manifest}: {}", .packages.join(", "))]
    AmbiguousDioxusPackages {
        /// Nearest manifest path.
        manifest: String,
        /// Candidate package manifests.
        packages: Vec<String>,
    },
    /// The selected Dioxus package has no supported Rust source entrypoint.
    #[error("Dioxus package {package} at {manifest} has no src/main.rs or src/lib.rs entrypoint")]
    MissingEntrypoint {
        /// Selected package name.
        package: String,
        /// Selected package manifest.
        manifest: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        fs::canonicalize(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/compile/project-discovery")
                .join(name),
        )
        .expect("fixture root should exist")
    }

    #[test]
    fn discovers_a_single_dioxus_package_from_a_nested_source_directory() {
        let root = fixture("single");
        let project = discover_dioxus_project(&root.join("src"))
            .expect("single Dioxus fixture should be discovered");
        assert_eq!(project.package_name, "single-dioxus-app");
        assert_eq!(project.package_manifest_path, root.join("Cargo.toml"));
        assert_eq!(project.entrypoint, root.join("src/main.rs"));
    }

    #[test]
    fn discovers_the_containing_workspace_package_without_guessing() {
        let root = fixture("ambiguous");
        let project = discover_dioxus_project(&root.join("web/src"))
            .expect("the invocation directory identifies one workspace package");
        assert_eq!(project.package_name, "web-app");
        assert_eq!(project.package_manifest_path, root.join("web/Cargo.toml"));
    }

    #[test]
    fn rejects_non_dioxus_and_ambiguous_workspaces() {
        let non_dioxus = discover_dioxus_project(&fixture("non-dioxus"))
            .expect_err("non-Dioxus package must not be accepted");
        assert!(matches!(
            non_dioxus,
            ProjectDiscoveryError::NoDioxusPackage { .. }
        ));

        let ambiguous = discover_dioxus_project(&fixture("ambiguous"))
            .expect_err("workspace root with two Dioxus packages is ambiguous");
        assert!(matches!(
            ambiguous,
            ProjectDiscoveryError::AmbiguousDioxusPackages { packages, .. }
                if packages.len() == 2
        ));
    }
}
