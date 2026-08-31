//! Repository automation for Awesome Dioxus Components.

mod catalog;
mod component_compat;
mod primitive_compat;
mod rust_introspect;

pub(crate) fn today() -> String {
    let output = Command::new("date").arg("+%Y-%m-%d").output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

pub(crate) fn now_utc() -> String {
    let output = Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use adico_registry_core::{
    EmbeddedRegistry, RegistryCompatibility, RegistryManifest, RegistryNamespace, RegistrySource,
    RegistrySourceLoader,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProvenanceRecord {
    id: String,
    revision: String,
    local_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedRegistryIndex {
    format_version: u32,
    namespace: RegistryNamespace,
    name: String,
    description: Option<String>,
    compatibility: RegistryCompatibility,
    items: BTreeMap<String, String>,
}

fn main() {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.as_slice() {
        [command, subcommand] if command == "provenance" && subcommand == "check" => {
            if let Err(error) = check_provenance() {
                eprintln!("provenance check failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand] if command == "registry" && subcommand == "build" => {
            if let Err(error) = build_registry() {
                eprintln!("registry build failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand] if command == "registry" && subcommand == "validate" => {
            if let Err(error) = validate_registry(None) {
                eprintln!("registry validation failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand, flag, source]
            if command == "registry" && subcommand == "validate" && flag == "--source" =>
        {
            if let Err(error) = validate_registry(Some(Path::new(source))) {
                eprintln!("registry validation failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand] if command == "primitive-compat" && subcommand == "sync" => {
            run_compat(primitive_compat::sync);
        }
        [command, subcommand] if command == "primitive-compat" && subcommand == "check" => {
            run_compat(primitive_compat::check);
        }
        [command, subcommand] if command == "primitive-compat" && subcommand == "diff" => {
            run_compat(primitive_compat::diff);
        }
        [command, subcommand] if command == "component-compat" && subcommand == "sync" => {
            run_compat(component_compat::sync);
        }
        [command, subcommand] if command == "component-compat" && subcommand == "check" => {
            run_compat(component_compat::check);
        }
        [command, subcommand, axis] if command == "catalog" && subcommand == "fetch" => {
            if let Err(error) = catalog_fetch(axis, None) {
                eprintln!("catalog fetch failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand, axis, flag, revision]
            if command == "catalog" && subcommand == "fetch" && flag == "--revision" =>
        {
            if let Err(error) = catalog_fetch(axis, Some(revision.as_str())) {
                eprintln!("catalog fetch failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand] if command == "catalog" && subcommand == "fetch" => {
            eprintln!(
                "usage: cargo xtask catalog fetch <axis|all> [--revision <sha>]\nknown axes:\n{}",
                catalog::usage_lines()
            );
            std::process::exit(2);
        }
        _ => {
            eprintln!(
                "usage:\n  cargo xtask provenance check\n  cargo xtask registry build\n  cargo xtask registry validate [--source <registry-directory-or-manifest>]\n  cargo xtask catalog fetch <axis|all> [--revision <sha>]\n  cargo xtask primitive-compat sync|check|diff\n  cargo xtask component-compat sync|check"
            );
            std::process::exit(2);
        }
    }
}

fn catalog_fetch(axis: &str, revision: Option<&str>) -> Result<(), String> {
    let root = repository_root()?;
    let axis_ids: Vec<&str> = if axis == "all" {
        catalog::AXES.iter().map(|axis| axis.id).collect()
    } else {
        vec![axis]
    };

    for axis_id in axis_ids {
        let Some(axis_def) = catalog::find(axis_id) else {
            return Err(format!(
                "unknown axis '{axis_id}'; known axes:\n{}",
                catalog::usage_lines()
            ));
        };
        let snapshot = (axis_def.fetch)(revision)?;
        let dir = catalog::statics_dir(&root);
        fs::create_dir_all(&dir)
            .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
        let path = catalog::statics_path(&root, axis_id);
        let payload = serde_json::to_string_pretty(&snapshot)
            .map_err(|error| format!("cannot serialize {axis_id} catalog: {error}"))?;
        write_if_changed(&path, &format!("{payload}\n"))?;
        println!(
            "wrote {} ({} entries, revision {})",
            path.display(),
            snapshot.entries.len(),
            snapshot.revision
        );
    }
    Ok(())
}

fn run_compat(action: impl FnOnce(&Path) -> Result<(), String>) {
    let result = repository_root().and_then(|root| action(&root));
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn build_registry() -> Result<(), String> {
    let root = repository_root()?;
    let manifest_path = root.join("registry/registry.json");
    let contents = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest = load_registry_manifest(
        &root.join("registry"),
        contents.as_bytes(),
        RegistrySource::Embedded,
    )?;

    let generated_root = root.join("registry/generated");
    let payload_root = generated_root.join("items");
    fs::create_dir_all(&payload_root)
        .map_err(|error| format!("cannot create {}: {error}", payload_root.display()))?;

    let mut item_paths = BTreeMap::new();
    let mut items = manifest.items.clone();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    for item in items {
        let relative_path = format!("items/{}.json", item.name);
        let payload = serde_json::to_string_pretty(&item)
            .map_err(|error| format!("cannot serialize item {}: {error}", item.name))?;
        write_if_changed(
            &generated_root.join(&relative_path),
            &format!("{payload}\n"),
        )?;
        item_paths.insert(item.name, relative_path);
    }

    let index = GeneratedRegistryIndex {
        format_version: manifest.format_version,
        namespace: manifest.namespace,
        name: manifest.name,
        description: manifest.description,
        compatibility: manifest.compatibility,
        items: item_paths,
    };
    let item_count = index.items.len();
    let index = serde_json::to_string_pretty(&index)
        .map_err(|error| format!("cannot serialize generated registry index: {error}"))?;
    let index_path = generated_root.join("index.json");
    write_if_changed(&index_path, &format!("{index}\n"))?;
    println!(
        "registry build passed: {} item payload(s) at {}",
        item_count,
        generated_root.display()
    );
    Ok(())
}

fn validate_registry(source: Option<&Path>) -> Result<(), String> {
    let root = repository_root()?;
    let official_root = root.join("registry");
    let official_manifest = fs::read(official_root.join("registry.json")).map_err(|error| {
        format!(
            "cannot read {}: {error}",
            official_root.join("registry.json").display()
        )
    })?;
    check_registry_source_formatting(&official_root, &official_manifest)?;
    let source = source.map_or(RegistrySource::Embedded, |path| RegistrySource::Local {
        path: path.display().to_string(),
    });
    let manifest = load_registry_manifest(&official_root, &official_manifest, source)?;
    println!(
        "registry validation passed: {} item payload(s) in {}",
        manifest.items.len(),
        manifest.namespace
    );
    Ok(())
}

/// Guards against the drift found in M4 task 5.3b: `registry/ui/*.rs` (and
/// `registry/lib/*.rs`) are not Cargo workspace members, so `cargo fmt --all`
/// never touches them, and a non-canonically-formatted registry source file
/// silently fails every consumer's own `cargo fmt --all --check` the moment
/// it's installed. Runs `rustfmt --edition 2024 --check` directly against
/// each `.rs` file the official registry manifest declares, so drift is
/// caught here instead of downstream in an installed consumer project.
fn check_registry_source_formatting(
    official_root: &Path,
    official_manifest: &[u8],
) -> Result<(), String> {
    let declared: RegistryManifest = serde_json::from_slice(official_manifest)
        .map_err(|error| format!("registry manifest is invalid: {error}"))?;

    let mut rust_sources: BTreeSet<PathBuf> = BTreeSet::new();
    for item in &declared.items {
        for file in &item.files {
            if file.source.ends_with(".rs") {
                rust_sources.insert(official_root.join(&file.source));
            }
        }
    }

    let mut non_canonical = Vec::new();
    for path in &rust_sources {
        let status = Command::new("rustfmt")
            .args(["--edition", "2024", "--check"])
            .arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|error| format!("failed to run rustfmt on {}: {error}", path.display()))?;
        if !status.success() {
            non_canonical.push(path.display().to_string());
        }
    }

    if non_canonical.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "registry source is not canonically rustfmt-formatted (run `rustfmt --edition 2024 <path>` to fix): {}",
            non_canonical.join(", ")
        ))
    }
}

fn load_registry_manifest(
    official_root: &Path,
    official_manifest: &[u8],
    source: RegistrySource,
) -> Result<RegistryManifest, String> {
    let configured_manifest = match &source {
        RegistrySource::Embedded => official_manifest.to_vec(),
        RegistrySource::Local { path } => {
            let candidate = PathBuf::from(path);
            let manifest_path = if candidate.is_dir() {
                candidate.join("registry.json")
            } else {
                candidate
            };
            fs::read(&manifest_path)
                .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?
        }
        RegistrySource::Https { .. } => {
            return Err(
                "cargo xtask registry validate only accepts embedded or local sources".into(),
            );
        }
    };
    let declared: RegistryManifest = serde_json::from_slice(&configured_manifest)
        .map_err(|error| format!("registry manifest is invalid: {error}"))?;
    let loader = RegistrySourceLoader::new(EmbeddedRegistry::new(
        official_manifest.to_vec(),
        official_root,
    ));
    loader
        .load(&declared.namespace, &source)
        .map(|loaded| loaded.manifest)
        .map_err(|error| error.to_string())
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), String> {
    let existing = fs::read_to_string(path).ok();
    if existing.as_deref() != Some(contents) {
        fs::write(path, contents)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    }
    Ok(())
}

fn check_provenance() -> Result<(), String> {
    let root = repository_root()?;
    let records_dir = root.join("provenance/records");
    let mut recorded_paths = BTreeSet::new();
    let mut checked_records = 0usize;

    for entry in fs::read_dir(&records_dir)
        .map_err(|error| format!("cannot read {}: {error}", records_dir.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read provenance entry: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let record: ProvenanceRecord = serde_json::from_str(&contents)
            .map_err(|error| format!("{} is not valid JSON: {error}", path.display()))?;

        // The M0 schema fixture deliberately uses an all-zero placeholder revision
        // and does not represent imported code.
        if record.revision == "0".repeat(40) {
            continue;
        }
        if record.revision.len() != 40
            || !record.revision.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "{} has an invalid immutable revision",
                path.display()
            ));
        }
        if record.local_paths.is_empty() {
            return Err(format!("{} has no local paths", path.display()));
        }

        checked_records += 1;
        for local_path in record.local_paths {
            let absolute_path = root.join(&local_path);
            let contents = fs::read_to_string(&absolute_path).map_err(|error| {
                format!(
                    "record {} references unreadable {}: {error}",
                    record.id,
                    absolute_path.display()
                )
            })?;
            if !contents.contains(&record.revision) {
                return Err(format!(
                    "record {} references {} without its revision header",
                    record.id, local_path
                ));
            }
            recorded_paths.insert(local_path);
        }
    }

    let primitive_source = root.join("packages/adico-primitives/src");
    let mut imported_paths = Vec::new();
    collect_imported_paths(&root, &primitive_source, &mut imported_paths)?;
    for imported_path in imported_paths {
        if !recorded_paths.contains(&imported_path) {
            return Err(format!(
                "imported source {imported_path} has no provenance record"
            ));
        }
    }

    println!(
        "provenance check passed: {checked_records} imported record(s), {} source unit(s)",
        recorded_paths.len()
    );
    Ok(())
}

fn repository_root() -> Result<PathBuf, String> {
    let mut directory = env::current_dir().map_err(|error| error.to_string())?;
    loop {
        if directory.join("Cargo.toml").is_file() && directory.join("provenance").is_dir() {
            return Ok(directory);
        }
        if !directory.pop() {
            return Err("could not find repository root".to_string());
        }
    }
}

fn collect_imported_paths(
    root: &Path,
    directory: &Path,
    imported_paths: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read source entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_imported_paths(root, &path, imported_paths)?;
            continue;
        }
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if contents.contains("DioxusLabs/dioxus-components at ") {
            let relative_path = path
                .strip_prefix(root)
                .map_err(|error| format!("cannot relativize {}: {error}", path.display()))?;
            imported_paths.push(relative_path.to_string_lossy().into_owned());
        }
    }
    Ok(())
}
