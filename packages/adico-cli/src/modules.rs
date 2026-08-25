//! Marker-owned `mod.rs` management for installed registry source.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use thiserror::Error;

/// Start marker that delimits adico-owned module declarations and re-exports.
pub const MANAGED_REGION_START: &str = "// adico:start";
/// End marker that delimits adico-owned module declarations and re-exports.
pub const MANAGED_REGION_END: &str = "// adico:end";

/// One module declaration required by installed source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleExportRequest {
    /// Rust module identifier.
    pub module: String,
    /// Whether to generate a public glob re-export.
    pub reexport: bool,
}

/// A reviewable file update generated without writing to the consumer project.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleUpdatePlan {
    /// Managed module file path.
    pub path: PathBuf,
    /// Complete new contents when a change is required.
    pub contents: Option<String>,
}

impl ModuleUpdatePlan {
    /// Returns whether applying this plan changes the module file.
    pub fn has_changes(&self) -> bool {
        self.contents.is_some()
    }

    /// Applies only the planned module-file content.
    pub fn apply(&self) -> Result<(), ModuleError> {
        let Some(contents) = &self.contents else {
            return Ok(());
        };
        let parent = self.path.parent().ok_or_else(|| ModuleError::WriteFailed {
            path: self.path.display().to_string(),
            message: "module file has no parent directory".to_string(),
        })?;
        fs::create_dir_all(parent).map_err(|error| ModuleError::WriteFailed {
            path: parent.display().to_string(),
            message: error.to_string(),
        })?;
        fs::write(&self.path, contents).map_err(|error| ModuleError::WriteFailed {
            path: self.path.display().to_string(),
            message: error.to_string(),
        })
    }
}

/// Plans a deterministic update to one adico-managed `mod.rs` region.
///
/// Each `adico add` invocation only knows the registry items it was asked to
/// install, not every item a prior invocation already declared here. Merging
/// `requests` into whatever the managed region already lists (rather than
/// replacing it outright) is what keeps `adico add <new-item>` from silently
/// dropping every previously installed module's `pub mod`/`pub use` lines.
pub fn plan_module_update(
    path: impl Into<PathBuf>,
    requests: &[ModuleExportRequest],
) -> Result<ModuleUpdatePlan, ModuleError> {
    let path = path.into();
    let new_entries = normalized_entries(requests)?;
    let contents = match path.try_exists().map_err(|error| ModuleError::ReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })? {
        false => {
            let managed_body = managed_body(&new_entries);
            Some(format!(
                "{MANAGED_REGION_START}\n{managed_body}{MANAGED_REGION_END}\n"
            ))
        }
        true => {
            let existing = fs::read_to_string(&path).map_err(|error| ModuleError::ReadFailed {
                path: path.display().to_string(),
                message: error.to_string(),
            })?;
            let mut merged = managed_region_entries(&existing)?;
            for (module, reexport) in new_entries {
                merged
                    .entry(module)
                    .and_modify(|existing_reexport| *existing_reexport |= reexport)
                    .or_insert(reexport);
            }
            let managed_body = managed_body(&merged);
            let updated = replace_managed_region(&existing, &managed_body)?;
            (updated != existing).then_some(updated)
        }
    };
    Ok(ModuleUpdatePlan { path, contents })
}

/// Extracts the module entries already declared in a file's managed region,
/// using the same marker validation `replace_managed_region` applies so a
/// missing/malformed region is still rejected rather than silently merged
/// against nothing.
fn managed_region_entries(existing: &str) -> Result<BTreeMap<String, bool>, ModuleError> {
    let starts = marker_positions(existing, MANAGED_REGION_START);
    let ends = marker_positions(existing, MANAGED_REGION_END);
    let (start, end) = match (starts.as_slice(), ends.as_slice()) {
        ([], []) => return Err(ModuleError::MissingManagedRegion),
        ([start], [end]) if start < end => (*start, *end),
        _ => return Err(ModuleError::MalformedManagedRegion),
    };
    let body = &existing[start + MANAGED_REGION_START.len()..end];
    let mut entries = BTreeMap::new();
    for line in body.lines() {
        let line = line.trim();
        if let Some(module) = line
            .strip_prefix("pub mod ")
            .and_then(|rest| rest.strip_suffix(';'))
        {
            entries.entry(module.to_string()).or_insert(false);
        } else if let Some(module) = line
            .strip_prefix("pub use ")
            .and_then(|rest| rest.strip_suffix("::*;"))
        {
            entries.insert(module.to_string(), true);
        }
    }
    Ok(entries)
}

/// Plans an entrypoint-owned module region. Unlike a nested `mod.rs`, a Rust
/// application entrypoint is allowed to gain one new marker region at EOF so
/// `adico init` can expose its generated top-level module trees without
/// touching user code. Once present, the region follows the ordinary strict
/// marker rules.
pub fn plan_entrypoint_module_update(
    path: impl Into<PathBuf>,
    requests: &[ModuleExportRequest],
) -> Result<ModuleUpdatePlan, ModuleError> {
    let path = path.into();
    let new_entries = normalized_entries(requests)?;
    let existing = fs::read_to_string(&path).map_err(|error| ModuleError::ReadFailed {
        path: path.display().to_string(),
        message: error.to_string(),
    })?;
    let updated = match (
        marker_positions(&existing, MANAGED_REGION_START),
        marker_positions(&existing, MANAGED_REGION_END),
    ) {
        (starts, ends) if starts.is_empty() && ends.is_empty() => {
            let managed_body = managed_body(&new_entries);
            format!(
                "{}\n\n{MANAGED_REGION_START}\n{managed_body}{MANAGED_REGION_END}\n",
                existing.trim_end()
            )
        }
        _ => {
            let mut merged = managed_region_entries(&existing)?;
            for (module, reexport) in new_entries {
                merged
                    .entry(module)
                    .and_modify(|existing_reexport| *existing_reexport |= reexport)
                    .or_insert(reexport);
            }
            let managed_body = managed_body(&merged);
            replace_managed_region(&existing, &managed_body)?
        }
    };
    Ok(ModuleUpdatePlan {
        path,
        contents: (updated != existing).then_some(updated),
    })
}

fn normalized_entries(
    requests: &[ModuleExportRequest],
) -> Result<BTreeMap<String, bool>, ModuleError> {
    let mut entries = BTreeMap::new();
    for request in requests {
        if !is_rust_identifier(&request.module) {
            return Err(ModuleError::InvalidModuleName {
                module: request.module.clone(),
            });
        }
        entries
            .entry(request.module.clone())
            .and_modify(|reexport| *reexport |= request.reexport)
            .or_insert(request.reexport);
    }
    Ok(entries)
}

fn is_rust_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(character) if character == '_' || character.is_ascii_lowercase())
        && characters.all(|character| {
            character == '_' || character.is_ascii_lowercase() || character.is_ascii_digit()
        })
}

fn managed_body(entries: &BTreeMap<String, bool>) -> String {
    let declarations = entries
        .keys()
        .map(|module| format!("pub mod {module};"))
        .collect::<Vec<_>>()
        .join("\n");
    let reexports = entries
        .iter()
        .filter(|(_, reexport)| **reexport)
        .map(|(module, _)| format!("pub use {module}::*;"))
        .collect::<Vec<_>>()
        .join("\n");
    match (declarations.is_empty(), reexports.is_empty()) {
        (true, _) => String::new(),
        (false, true) => format!("{declarations}\n"),
        (false, false) => format!("{declarations}\n\n{reexports}\n"),
    }
}

fn replace_managed_region(existing: &str, body: &str) -> Result<String, ModuleError> {
    let starts = marker_positions(existing, MANAGED_REGION_START);
    let ends = marker_positions(existing, MANAGED_REGION_END);
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => Err(ModuleError::MissingManagedRegion),
        ([start], [end]) if start < end => Ok(format!(
            "{}\n{}{}{}",
            &existing[..start + MANAGED_REGION_START.len()],
            body,
            MANAGED_REGION_END,
            &existing[end + MANAGED_REGION_END.len()..]
        )),
        _ => Err(ModuleError::MalformedManagedRegion),
    }
}

fn marker_positions(contents: &str, marker: &str) -> Vec<usize> {
    contents
        .match_indices(marker)
        .map(|(position, _)| position)
        .collect()
}

/// Module-management failures that preserve consumer code by refusing unsafe edits.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ModuleError {
    /// Existing code lacks explicit adico ownership markers.
    #[error(
        "existing mod.rs has no adico-managed region; add one explicitly or resolve the conflict before installation"
    )]
    MissingManagedRegion,
    /// Markers are duplicated, reversed, or otherwise not a single well-formed region.
    #[error("existing mod.rs has malformed or duplicate adico-managed markers")]
    MalformedManagedRegion,
    /// Registry metadata requested an invalid Rust module identifier.
    #[error("invalid Rust module name {module:?}")]
    InvalidModuleName {
        /// Invalid module identifier.
        module: String,
    },
    /// Existing module source could not be read.
    #[error("cannot read {path}: {message}")]
    ReadFailed {
        /// Module path.
        path: String,
        /// Filesystem reason.
        message: String,
    },
    /// Planned module source could not be written.
    #[error("cannot write {path}: {message}")]
    WriteFailed {
        /// Module path.
        path: String,
        /// Filesystem reason.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_module_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after epoch")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("adico-modules-test-{}-{nonce}", std::process::id()))
            .join("src/components/ui/mod.rs")
    }

    fn requests() -> Vec<ModuleExportRequest> {
        vec![
            ModuleExportRequest {
                module: "dialog".to_string(),
                reexport: true,
            },
            ModuleExportRequest {
                module: "button".to_string(),
                reexport: true,
            },
        ]
    }

    #[test]
    fn creates_sorted_marker_owned_module_content_and_is_idempotent() {
        let path = temporary_module_path();
        let plan = plan_module_update(&path, &requests()).expect("new module should be planable");
        assert_eq!(
            plan.contents.as_deref(),
            Some(
                "// adico:start\npub mod button;\npub mod dialog;\n\npub use button::*;\npub use dialog::*;\n// adico:end\n"
            )
        );
        plan.apply().expect("new module should be created");
        assert!(
            !plan_module_update(&path, &requests())
                .expect("same request should be planable")
                .has_changes()
        );
        fs::remove_dir_all(path.ancestors().nth(4).expect("temp root should exist"))
            .expect("test directory should be removable");
    }

    #[test]
    fn a_later_add_merges_with_rather_than_replaces_earlier_declarations() {
        let path = temporary_module_path();
        // "dialog" and "button" installed first, matching a real `adico add`.
        plan_module_update(&path, &requests())
            .expect("first module update should plan")
            .apply()
            .expect("first module update should apply");

        // A later, unrelated `adico add card` only knows about "card" -- it
        // must not drop "button"/"dialog" from the managed region.
        let later_request = vec![ModuleExportRequest {
            module: "card".to_string(),
            reexport: true,
        }];
        let plan =
            plan_module_update(&path, &later_request).expect("later module update should plan");
        assert_eq!(
            plan.contents.as_deref(),
            Some(
                "// adico:start\npub mod button;\npub mod card;\npub mod dialog;\n\npub use button::*;\npub use card::*;\npub use dialog::*;\n// adico:end\n"
            )
        );
        plan.apply().expect("later module update should apply");
        assert!(
            !plan_module_update(&path, &later_request)
                .expect("repeated later request should plan")
                .has_changes()
        );

        fs::remove_dir_all(path.ancestors().nth(4).expect("temp root should exist"))
            .expect("test directory should be removable");
    }

    #[test]
    fn preserves_all_bytes_outside_the_managed_region() {
        let path = temporary_module_path();
        let original = include_str!("../../../tests/compile/modules/preserve-mod.rs");
        fs::create_dir_all(path.parent().expect("module parent should exist"))
            .expect("module parent should be created");
        fs::write(&path, original).expect("fixture module should be written");
        let start = original
            .find(MANAGED_REGION_START)
            .expect("start marker should exist");
        let end = original
            .find(MANAGED_REGION_END)
            .expect("end marker should exist");
        let prefix = &original[..start];
        let suffix = &original[end + MANAGED_REGION_END.len()..];

        let plan = plan_module_update(&path, &requests()).expect("managed fixture should update");
        plan.apply().expect("managed fixture should apply");
        let updated = fs::read_to_string(&path).expect("updated module should be readable");
        assert!(updated.starts_with(prefix));
        assert!(updated.ends_with(suffix));
        assert!(updated.contains("pub mod button;\npub mod dialog;"));
        fs::remove_dir_all(path.ancestors().nth(4).expect("temp root should exist"))
            .expect("test directory should be removable");
    }

    #[test]
    fn rejects_missing_duplicate_and_malformed_marker_regions() {
        let path = temporary_module_path();
        fs::create_dir_all(path.parent().expect("module parent should exist"))
            .expect("module parent should be created");
        fs::write(&path, "pub mod consumer;\n").expect("fixture module should be written");
        assert_eq!(
            plan_module_update(&path, &requests()).expect_err("missing markers must fail"),
            ModuleError::MissingManagedRegion
        );
        fs::write(
            &path,
            "// adico:start\n// adico:end\n// adico:start\n// adico:end\n",
        )
        .expect("fixture module should be written");
        assert_eq!(
            plan_module_update(&path, &requests()).expect_err("duplicate markers must fail"),
            ModuleError::MalformedManagedRegion
        );
        fs::remove_dir_all(path.ancestors().nth(4).expect("temp root should exist"))
            .expect("test directory should be removable");
    }

    #[test]
    fn appends_one_entrypoint_region_and_then_remains_idempotent() {
        let path = temporary_module_path();
        fs::create_dir_all(path.parent().expect("module parent should exist"))
            .expect("module parent should be created");
        fs::write(&path, "fn main() {}\n").expect("entrypoint should be written");
        let plan = plan_entrypoint_module_update(
            &path,
            &[ModuleExportRequest {
                module: "components".to_string(),
                reexport: false,
            }],
        )
        .expect("entrypoint should plan");
        plan.apply().expect("entrypoint plan should apply");
        let updated = fs::read_to_string(&path).expect("entrypoint should be readable");
        assert!(updated.starts_with("fn main() {}\n"));
        assert!(updated.contains("// adico:start\npub mod components;\n// adico:end"));
        assert!(
            !plan_entrypoint_module_update(
                &path,
                &[ModuleExportRequest {
                    module: "components".to_string(),
                    reexport: false,
                }]
            )
            .expect("repeat should plan")
            .has_changes()
        );
        fs::remove_dir_all(path.ancestors().nth(4).expect("temp root should exist"))
            .expect("temporary directory should be removable");
    }
}
