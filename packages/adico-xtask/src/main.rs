//! Repository automation for Awesome Dioxus Components.

mod catalog;
mod component_compat;
mod component_props;
mod playground_controls;
mod primitive_compat;
mod primitive_usage;
mod prop_parity;
mod registry_checksums;
mod registry_introspect;
mod rust_introspect;
mod styling_usage;

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

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use adico_registry_core::{
    EmbeddedRegistry, REGISTRY_FORMAT_VERSION, RegistryItem, RegistryManifest, RegistrySource,
    RegistrySourceLoader,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProvenanceRecord {
    id: String,
    revision: String,
    local_paths: Vec<String>,
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
        [command, subcommand, flag]
            if command == "registry" && subcommand == "build" && flag == "--check" =>
        {
            if let Err(error) = check_registry_build_drift() {
                eprintln!("registry build --check failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand] if command == "registry" && subcommand == "validate" => {
            if let Err(error) = validate_registry(None) {
                eprintln!("registry validation failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand, flag]
            if command == "registry" && subcommand == "checksums" && flag == "--write" =>
        {
            if let Err(error) = repository_root().and_then(|root| registry_checksums::write(&root))
            {
                eprintln!("registry checksums --write failed: {error}");
                std::process::exit(1);
            }
        }
        [command, subcommand] if command == "registry" && subcommand == "checksums" => {
            eprintln!("usage: cargo xtask registry checksums --write");
            std::process::exit(2);
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
        [command, subcommand] if command == "primitive-usage" && subcommand == "sync" => {
            run_compat(primitive_usage::sync);
        }
        [command, subcommand] if command == "primitive-usage" && subcommand == "check" => {
            run_compat(primitive_usage::check);
        }
        [command, subcommand] if command == "primitive-usage" && subcommand == "diff" => {
            run_compat(primitive_usage::diff);
        }
        [command] if command == "primitive-usage" => {
            eprintln!("usage: cargo xtask primitive-usage sync|check|diff");
            std::process::exit(2);
        }
        [command, subcommand] if command == "styling-usage" && subcommand == "sync" => {
            run_compat(styling_usage::sync);
        }
        [command, subcommand] if command == "styling-usage" && subcommand == "check" => {
            run_compat(styling_usage::check);
        }
        [command, subcommand] if command == "styling-usage" && subcommand == "diff" => {
            run_compat(styling_usage::diff);
        }
        [command] if command == "styling-usage" => {
            eprintln!("usage: cargo xtask styling-usage sync|check|diff");
            std::process::exit(2);
        }
        [command, subcommand] if command == "playground-controls" && subcommand == "sync" => {
            run_compat(playground_controls::sync);
        }
        [command, subcommand] if command == "playground-controls" && subcommand == "check" => {
            run_compat(playground_controls::check);
        }
        [command, subcommand] if command == "playground-controls" && subcommand == "diff" => {
            run_compat(playground_controls::diff);
        }
        [command] if command == "playground-controls" => {
            eprintln!("usage: cargo xtask playground-controls sync|check|diff");
            std::process::exit(2);
        }
        [command, subcommand] if command == "prop-parity" && subcommand == "sync" => {
            run_compat(prop_parity::sync);
        }
        [command, subcommand] if command == "prop-parity" && subcommand == "check" => {
            run_compat(prop_parity::check);
        }
        [command, subcommand] if command == "prop-parity" && subcommand == "diff" => {
            run_compat(prop_parity::diff);
        }
        [command] if command == "prop-parity" => {
            eprintln!("usage: cargo xtask prop-parity sync|check|diff");
            std::process::exit(2);
        }
        [command, subcommand] if command == "component-props" && subcommand == "sync" => {
            run_compat(component_props::sync);
        }
        [command, subcommand] if command == "component-props" && subcommand == "check" => {
            run_compat(component_props::check);
        }
        [command, subcommand] if command == "component-props" && subcommand == "diff" => {
            run_compat(component_props::diff);
        }
        [command] if command == "component-props" => {
            eprintln!("usage: cargo xtask component-props sync|check|diff");
            std::process::exit(2);
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
                "usage:\n  cargo xtask provenance check\n  cargo xtask registry build\n  cargo xtask registry validate [--source <registry-directory-or-manifest>]\n  cargo xtask registry checksums --write\n  cargo xtask catalog fetch <axis|all> [--revision <sha>]\n  cargo xtask primitive-compat sync|check|diff\n  cargo xtask component-compat sync|check\n  cargo xtask primitive-usage sync|check|diff\n  cargo xtask styling-usage sync|check|diff\n  cargo xtask playground-controls sync|check|diff\n  cargo xtask prop-parity sync|check|diff\n  cargo xtask component-props sync|check|diff"
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

/// Returns a clone of `item` with every file's `content` populated from its
/// authored bytes under `source_root` (`registry/`). Used to produce both
/// the served per-item documents and the CLI's embedded fallback payload --
/// see design D4 of `adopt-shadcn-style-registry-serving`.
fn content_bearing_item(source_root: &Path, item: &RegistryItem) -> Result<RegistryItem, String> {
    let mut item = item.clone();
    for file in &mut item.files {
        let path = source_root.join(&file.source);
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        file.content = Some(content);
    }
    Ok(item)
}

/// Everything `cargo xtask registry build` computes from
/// `registry/registry.json`, before any of it is written to disk. Shared by
/// `build_registry` (writes it) and `check_registry_build_drift` (compares
/// the embedded payload against what's already committed, without writing).
struct RegistryBuildOutputs {
    generated_root: PathBuf,
    /// `(file name under generated_root, pretty-printed JSON + trailing newline)`,
    /// one entry per item, plus a final `("index.json", ...)` entry.
    served_tree: Vec<(String, String)>,
    embedded_path: PathBuf,
    embedded_payload: String,
    item_count: usize,
}

fn compute_registry_build_outputs(root: &Path) -> Result<RegistryBuildOutputs, String> {
    let manifest_path = root.join("registry/registry.json");
    let contents = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest = load_registry_manifest(
        &root.join("registry"),
        contents.as_bytes(),
        RegistrySource::Embedded,
    )?;
    let source_root = root.join("registry");

    let mut items = manifest.items.clone();
    items.sort_by(|left, right| left.name.cmp(&right.name));
    let mut content_bearing_items = Vec::with_capacity(items.len());
    for item in &items {
        content_bearing_items.push(content_bearing_item(&source_root, item)?);
    }

    // Served tree: a content-free index (the full manifest, format 2) plus
    // one content-bearing document per item, directly under the registry
    // root (`<item-name>.json`, not nested under `items/`) -- matching the
    // shadcn-style `/r/<name>.json` shape. Not committed to git; it is
    // regenerated fresh wherever the registry is actually served (a
    // separate, dependent infrastructure change). `.gitignore`d.
    let mut served_tree = Vec::with_capacity(content_bearing_items.len() + 1);
    for item in &content_bearing_items {
        let payload = serde_json::to_string_pretty(item)
            .map_err(|error| format!("cannot serialize item {}: {error}", item.name))?;
        served_tree.push((format!("{}.json", item.name), format!("{payload}\n")));
    }

    let mut index_manifest = manifest.clone();
    index_manifest.format_version = REGISTRY_FORMAT_VERSION;
    index_manifest.items = items;
    let item_count = index_manifest.items.len();
    let index = serde_json::to_string_pretty(&index_manifest)
        .map_err(|error| format!("cannot serialize generated registry index: {error}"))?;
    served_tree.push(("index.json".to_string(), format!("{index}\n")));

    // CLI embedded fallback payload: one committed, content-bearing
    // manifest replacing the previous 72 `include_bytes!` arms with one
    // (design D1/D4). Committed because a crates.io build packages only
    // `adico-cli`'s own crate directory and cannot invoke this command.
    let mut embedded_manifest = manifest;
    embedded_manifest.format_version = REGISTRY_FORMAT_VERSION;
    embedded_manifest.items = content_bearing_items;
    let embedded_payload = serde_json::to_string_pretty(&embedded_manifest)
        .map_err(|error| format!("cannot serialize embedded registry payload: {error}"))?;

    Ok(RegistryBuildOutputs {
        generated_root: root.join("registry/generated"),
        served_tree,
        embedded_path: root.join("packages/adico-cli/embedded/registry.json"),
        embedded_payload: format!("{embedded_payload}\n"),
        item_count,
    })
}

fn build_registry() -> Result<(), String> {
    let root = repository_root()?;
    let outputs = compute_registry_build_outputs(&root)?;

    // Remove and recreate rather than writing over the existing tree: a
    // renamed or removed item must not leave an orphaned, stale document
    // behind (this previously left a whole stale `items/` subdirectory from
    // the pre-D4 layout in place indefinitely).
    if outputs.generated_root.is_dir() {
        fs::remove_dir_all(&outputs.generated_root).map_err(|error| {
            format!(
                "cannot remove {}: {error}",
                outputs.generated_root.display()
            )
        })?;
    }
    fs::create_dir_all(&outputs.generated_root).map_err(|error| {
        format!(
            "cannot create {}: {error}",
            outputs.generated_root.display()
        )
    })?;
    for (name, payload) in &outputs.served_tree {
        write_if_changed(&outputs.generated_root.join(name), payload)?;
    }

    if let Some(parent) = outputs.embedded_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    write_if_changed(&outputs.embedded_path, &outputs.embedded_payload)?;

    println!(
        "registry build passed: {} item payload(s) at {} and {}",
        outputs.item_count,
        outputs.generated_root.display(),
        outputs.embedded_path.display()
    );
    Ok(())
}

/// Fails if the committed `packages/adico-cli/embedded/registry.json`
/// disagrees with a fresh regeneration from `registry/registry.json`,
/// without writing anything. This is the CI-gated drift check that closes
/// the exact gap that let a prior generated-payload drift incident (22 of
/// 43 payloads silently missing) go undetected -- see design D8.
fn check_registry_build_drift() -> Result<(), String> {
    check_registry_build_drift_at(&repository_root()?)
}

fn check_registry_build_drift_at(root: &Path) -> Result<(), String> {
    let outputs = compute_registry_build_outputs(root)?;
    let committed = fs::read_to_string(&outputs.embedded_path).map_err(|error| {
        format!(
            "cannot read {}: {error} (has `cargo xtask registry build` ever been run and committed?)",
            outputs.embedded_path.display()
        )
    })?;
    if committed != outputs.embedded_payload {
        return Err(format!(
            "{} is stale relative to registry/registry.json -- run `cargo xtask registry build` and commit the result",
            outputs.embedded_path.display()
        ));
    }
    println!(
        "registry build --check passed: {} matches a fresh regeneration",
        outputs.embedded_path.display()
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
    let loaded = loader
        .load(&declared.namespace, &source)
        .map_err(|error| error.to_string())?;
    // `load` only performs the structural validation that's cheap against a
    // potentially remote/large registry (see design D3 of
    // `adopt-shadcn-style-registry-serving`); xtask always validates a
    // local, free-to-read source, so it explicitly pays for the exhaustive
    // checksum verification `load` no longer does automatically. Without
    // this call, a tampered checksum in `registry/registry.json` would pass
    // `registry validate` silently.
    loader
        .validate_all_content(&loaded)
        .map_err(|error| error.to_string())?;
    Ok(loaded.manifest)
}

pub(crate) fn write_if_changed(path: &Path, contents: &str) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn checksum_mismatch_source_fixture(manifest_name: &str) -> PathBuf {
        repository_root()
            .expect("repository root should resolve in test context")
            .join("tests/compile/registry/checksum-mismatch-source")
            .join(manifest_name)
    }

    #[test]
    fn registry_validate_still_catches_a_tampered_checksum() {
        // Regression test: `validate_registry`/`load_registry_manifest`
        // used to catch this because the old, eager `RegistrySourceLoader
        // ::validate` checksummed every file. That loop moved to
        // `validate_all_content` (design D3) to make runtime CLI resolution
        // proportional to requested items -- but `registry validate` always
        // reads a local, free-to-check source, so it must explicitly keep
        // calling `validate_all_content` or a tampered checksum would pass
        // silently. This was caught manually during implementation by
        // deliberately corrupting `registry/registry.json` and observing
        // `registry validate` wrongly pass; this test pins the fix.
        let correct = validate_registry(Some(&checksum_mismatch_source_fixture("registry.json")));
        assert!(
            correct.is_ok(),
            "correctly checksummed fixture should validate: {correct:?}"
        );

        let tampered = validate_registry(Some(&checksum_mismatch_source_fixture(
            "registry-tampered.json",
        )));
        let error = tampered.expect_err("tampered checksum must fail validation");
        assert!(
            error.contains("checksum mismatch"),
            "expected a checksum-mismatch error, got: {error}"
        );
    }

    fn temporary_registry_build_root() -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("valid system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "adico-xtask-registry-build-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("registry/ui")).expect("fixture registry dir should exist");
        fs::write(
            root.join("registry/ui/button.rs"),
            "pub const DRIFT_TEST_BUTTON: &str = \"xtask registry build drift fixture\";\n",
        )
        .expect("fixture source file should be writable");
        let checksum = {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(
                fs::read(root.join("registry/ui/button.rs")).expect("fixture file should exist"),
            );
            hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        };
        let manifest = serde_json::json!({
            "formatVersion": 1,
            "namespace": "@adico",
            "name": "xtask registry build drift fixture",
            "compatibility": { "cli": ">=0.1.0" },
            "items": [{
                "name": "button",
                "type": "registry:ui",
                "description": "Drift-check fixture button.",
                "files": [{
                    "source": "ui/button.rs",
                    "targetRoot": "ui",
                    "target": "button.rs",
                    "checksum": checksum
                }]
            }]
        });
        fs::write(
            root.join("registry/registry.json"),
            serde_json::to_string_pretty(&manifest).expect("fixture manifest should serialize"),
        )
        .expect("fixture manifest should be writable");
        root
    }

    #[test]
    fn registry_build_check_detects_a_stale_committed_embedded_payload() {
        let root = temporary_registry_build_root();

        // No committed embedded payload yet -- the check must fail loudly
        // (not silently pass), naming that it's never been built.
        let missing = check_registry_build_drift_at(&root);
        assert!(
            missing.is_err(),
            "check must fail when no embedded payload has ever been committed"
        );

        // Build once for real, then check again -- must now pass.
        let outputs = compute_registry_build_outputs(&root).expect("fixture registry should build");
        fs::create_dir_all(
            outputs
                .embedded_path
                .parent()
                .expect("embedded path has a parent"),
        )
        .expect("embedded directory should be created");
        fs::write(&outputs.embedded_path, &outputs.embedded_payload)
            .expect("embedded payload should be written");
        assert!(
            check_registry_build_drift_at(&root).is_ok(),
            "check must pass immediately after a real build"
        );

        // Tamper with the committed payload without rebuilding -- this is
        // exactly the prior drift incident (22/43 payloads silently
        // missing): the check must catch it, not pass silently.
        fs::write(&outputs.embedded_path, "{}").expect("tampered payload should be writable");
        let drifted = check_registry_build_drift_at(&root);
        let error = drifted.expect_err("a hand-edited/stale embedded payload must be rejected");
        assert!(
            error.contains("is stale"),
            "expected a staleness error, got: {error}"
        );

        fs::remove_dir_all(&root).ok();
    }
}
