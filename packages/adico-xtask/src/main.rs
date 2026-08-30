//! Repository automation for Awesome Dioxus Components.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use adico_registry_core::{
    EmbeddedRegistry, RegistryCompatibility, RegistryItemType, RegistryManifest, RegistryNamespace,
    RegistrySource, RegistrySourceLoader,
};

/// Registry items with no shadcn-catalog mapping, so they deliberately have
/// no `parity.json` entry: parity only tracks items explicitly mapped to the
/// upstream shadcn catalog (see `docs/adico/parity.md`). Most of these are
/// classified `EXISTING_DIOXUS_EXTRA` in
/// `upstreams/dioxus-components/inventory.md` (the M3 Wave 5 batch, task
/// 4.6); `theme-switcher` is an adico-original component with no upstream
/// source at all (task 4.8g, see design.md §7b) but is classified the same
/// way for the same reason -- ui.shadcn.com's theme customizer is a docs-site
/// feature, not a shipped component. Keep this list, the inventory table,
/// and design.md in sync if a future item needs the same treatment.
const DIOXUS_ONLY_EXTRAS: &[&str] = &[
    "color-picker",
    "drag-and-drop-list",
    "tag-group",
    "theme-builder",
    "theme-switcher",
    "toolbar",
    "virtual-list",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProvenanceRecord {
    id: String,
    revision: String,
    local_paths: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DioxusComponentsSnapshot {
    upstream: String,
    revision: String,
    refreshed_at: String,
    styled_components: Vec<String>,
    primitive_source_paths: Vec<String>,
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
        [command, upstream] if command == "upstream" && upstream == "dioxus-components" => {
            if let Err(error) = report_dioxus_components_snapshot() {
                eprintln!("upstream inventory failed: {error}");
                std::process::exit(1);
            }
        }
        [command, upstream, flag, source, date_flag, refreshed_at]
            if command == "upstream"
                && upstream == "dioxus-components"
                && flag == "--source"
                && date_flag == "--refreshed-at" =>
        {
            if let Err(error) =
                refresh_dioxus_components_snapshot(Path::new(source), refreshed_at, false)
            {
                eprintln!("upstream inventory failed: {error}");
                std::process::exit(1);
            }
        }
        [
            command,
            upstream,
            flag,
            source,
            date_flag,
            refreshed_at,
            write_flag,
        ] if command == "upstream"
            && upstream == "dioxus-components"
            && flag == "--source"
            && date_flag == "--refreshed-at"
            && write_flag == "--write" =>
        {
            if let Err(error) =
                refresh_dioxus_components_snapshot(Path::new(source), refreshed_at, true)
            {
                eprintln!("upstream inventory failed: {error}");
                std::process::exit(1);
            }
        }
        [command] if command == "parity" => {
            if let Err(error) = check_parity() {
                eprintln!("parity check failed: {error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!(
                "usage:\n  cargo xtask provenance check\n  cargo xtask registry build\n  cargo xtask registry validate [--source <registry-directory-or-manifest>]\n  cargo xtask upstream dioxus-components\n  cargo xtask upstream dioxus-components --source <local-clone> --refreshed-at <YYYY-MM-DD> [--write]\n  cargo xtask parity"
            );
            std::process::exit(2);
        }
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

fn report_dioxus_components_snapshot() -> Result<(), String> {
    let root = repository_root()?;
    let path = dioxus_components_snapshot_path(&root);
    let snapshot = read_dioxus_components_snapshot(&path)?;
    validate_sorted_unique(&snapshot.styled_components, "styled components")?;
    validate_sorted_unique(&snapshot.primitive_source_paths, "primitive source paths")?;
    println!(
        "Dioxus Components snapshot: {} styled component(s), {} primitive source unit(s), revision {} (refreshed {})",
        snapshot.styled_components.len(),
        snapshot.primitive_source_paths.len(),
        snapshot.revision,
        snapshot.refreshed_at
    );
    Ok(())
}

fn refresh_dioxus_components_snapshot(
    source: &Path,
    refreshed_at: &str,
    write: bool,
) -> Result<(), String> {
    let root = repository_root()?;
    let path = dioxus_components_snapshot_path(&root);
    let previous = path
        .exists()
        .then(|| read_dioxus_components_snapshot(&path))
        .transpose()?;
    let next = inspect_dioxus_components_source(source, refreshed_at)?;

    if let Some(previous) = previous {
        report_inventory_diff(
            "styled components",
            &previous.styled_components,
            &next.styled_components,
        );
        report_inventory_diff(
            "primitive source units",
            &previous.primitive_source_paths,
            &next.primitive_source_paths,
        );
    } else {
        println!(
            "Dioxus Components initial snapshot: {} styled component(s), {} primitive source unit(s)",
            next.styled_components.len(),
            next.primitive_source_paths.len()
        );
    }

    if write {
        let serialized = serde_json::to_string_pretty(&next)
            .map_err(|error| format!("cannot serialize upstream snapshot: {error}"))?;
        fs::write(&path, format!("{serialized}\n"))
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        println!("wrote {}", path.display());
    } else {
        println!("dry run only; pass --write to update the checked-in snapshot");
    }
    Ok(())
}

fn inspect_dioxus_components_source(
    source: &Path,
    refreshed_at: &str,
) -> Result<DioxusComponentsSnapshot, String> {
    if !refreshed_at.bytes().enumerate().all(|(index, byte)| {
        matches!(index, 4 | 7) && byte == b'-' || !matches!(index, 4 | 7) && byte.is_ascii_digit()
    }) || refreshed_at.len() != 10
    {
        return Err("--refreshed-at must use YYYY-MM-DD".to_string());
    }
    let revision = Command::new("git")
        .args(["-C"])
        .arg(source)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("cannot inspect git revision: {error}"))?;
    if !revision.status.success() {
        return Err(format!("cannot read revision from {}", source.display()));
    }
    let revision = String::from_utf8(revision.stdout)
        .map_err(|error| format!("git revision was not UTF-8: {error}"))?
        .trim()
        .to_string();
    let styled_components = names_in_directory(&source.join("preview/src/components"))?;
    let mut primitive_source_paths = Vec::new();
    collect_relative_source_paths(
        &source.join("primitives/src"),
        &source.join("primitives"),
        &mut primitive_source_paths,
    )?;
    primitive_source_paths.sort();

    Ok(DioxusComponentsSnapshot {
        upstream: "https://github.com/DioxusLabs/dioxus-components".to_string(),
        revision,
        refreshed_at: refreshed_at.to_string(),
        styled_components,
        primitive_source_paths,
    })
}

fn dioxus_components_snapshot_path(root: &Path) -> PathBuf {
    root.join("upstreams/dioxus-components/catalog.json")
}

fn read_dioxus_components_snapshot(path: &Path) -> Result<DioxusComponentsSnapshot, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("{} is invalid: {error}", path.display()))
}

fn names_in_directory(directory: &Path) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        if entry.path().is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

fn collect_relative_source_paths(
    directory: &Path,
    base: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read source entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_source_paths(&path, base, paths)?;
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("rs") | Some("js") | Some("ts")
        ) {
            let relative = path
                .strip_prefix(base)
                .map_err(|error| format!("cannot relativize {}: {error}", path.display()))?;
            paths.push(relative.to_string_lossy().into_owned());
        }
    }
    Ok(())
}

fn validate_sorted_unique(values: &[String], label: &str) -> Result<(), String> {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        Ok(())
    } else {
        Err(format!("snapshot {label} must be sorted and unique"))
    }
}

fn report_inventory_diff(label: &str, previous: &[String], next: &[String]) {
    let previous: BTreeSet<_> = previous.iter().collect();
    let next: BTreeSet<_> = next.iter().collect();
    let added: Vec<_> = next.difference(&previous).collect();
    let removed: Vec<_> = previous.difference(&next).collect();
    println!(
        "{label}: {} added, {} removed{}{}",
        added.len(),
        removed.len(),
        added
            .is_empty()
            .then(String::new)
            .unwrap_or_else(|| format!(
                " (added: {})",
                added
                    .iter()
                    .map(|value| value.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        removed
            .is_empty()
            .then(String::new)
            .unwrap_or_else(|| format!(
                " (removed: {})",
                removed
                    .iter()
                    .map(|value| value.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
    );
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

/// Mirrors `parity.schema.json`. Hand-rolled validation (rather than a JSON
/// Schema crate) matches this repository's existing `registry validate`/
/// `provenance check` style: serde enforces shape, and the functions below
/// enforce the business rules the schema expresses as conditionals.
#[derive(Debug, Deserialize)]
struct ParityManifest {
    #[serde(rename = "schemaVersion")]
    #[allow(dead_code)]
    schema_version: u32,
    catalog: ParityCatalog,
    components: BTreeMap<String, ParityComponent>,
}

#[derive(Debug, Deserialize)]
struct ParityCatalog {
    status: String,
    source: Option<String>,
    revision: Option<String>,
    #[serde(rename = "refreshedAt")]
    refreshed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ParityComponent {
    status: String,
    #[serde(rename = "registryItem")]
    registry_item: Option<String>,
    dimensions: ParityDimensions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParityDimensions {
    source: ParityDimension,
    api: ParityDimension,
    visual: ParityDimension,
    variants: ParityDimension,
    states: ParityDimension,
    keyboard: ParityDimension,
    accessibility: ParityDimension,
    dark_mode: ParityDimension,
    rtl: ParityDimension,
    responsive: ParityDimension,
    examples: ParityDimension,
    cli: ParityDimension,
    cargo: ParityDimension,
    web: ParityDimension,
    desktop: ParityDimension,
    ssr_hydration: ParityDimension,
    docs: ParityDimension,
}

impl ParityDimensions {
    fn named(&self) -> [(&'static str, &ParityDimension); 17] {
        [
            ("source", &self.source),
            ("api", &self.api),
            ("visual", &self.visual),
            ("variants", &self.variants),
            ("states", &self.states),
            ("keyboard", &self.keyboard),
            ("accessibility", &self.accessibility),
            ("darkMode", &self.dark_mode),
            ("rtl", &self.rtl),
            ("responsive", &self.responsive),
            ("examples", &self.examples),
            ("cli", &self.cli),
            ("cargo", &self.cargo),
            ("web", &self.web),
            ("desktop", &self.desktop),
            ("ssrHydration", &self.ssr_hydration),
            ("docs", &self.docs),
        ]
    }
}

#[derive(Debug, Deserialize)]
struct ParityDimension {
    applicable: bool,
    passed: bool,
    #[serde(default)]
    evidence: Vec<String>,
    note: Option<String>,
}

fn check_parity() -> Result<(), String> {
    let root = repository_root()?;
    check_parity_at(
        &root.join("registry/registry.json"),
        &root.join("parity.json"),
    )
}

fn check_parity_at(registry_path: &Path, parity_path: &Path) -> Result<(), String> {
    let registry_contents = fs::read(registry_path)
        .map_err(|error| format!("cannot read {}: {error}", registry_path.display()))?;
    let manifest: RegistryManifest = serde_json::from_slice(&registry_contents)
        .map_err(|error| format!("registry manifest is invalid: {error}"))?;

    let parity_contents = fs::read_to_string(parity_path)
        .map_err(|error| format!("cannot read {}: {error}", parity_path.display()))?;
    let parity: ParityManifest = serde_json::from_str(&parity_contents)
        .map_err(|error| format!("parity manifest is invalid: {error}"))?;

    validate_parity_manifest(&parity)?;

    let required = classify_registry_items(&manifest);
    let unclassified = find_unclassified(&required, &parity);
    if !unclassified.is_empty() {
        return Err(format!(
            "unclassified migrated entry(ies) with no parity.json record: {}",
            unclassified.join(", ")
        ));
    }

    let extras_present = manifest
        .items
        .iter()
        .filter(|item| DIOXUS_ONLY_EXTRAS.contains(&item.name.as_str()))
        .count();

    println!(
        "parity check passed: {}/{} registry items classified ({extras_present} extras excluded)",
        required.len(),
        required.len()
    );
    Ok(())
}

/// Returns the name of every registry item that is required to carry a
/// `parity.json` entry: `registry:ui`/`registry:component` items, excluding
/// [`DIOXUS_ONLY_EXTRAS`] (which are intentionally unmapped to shadcn).
fn classify_registry_items(manifest: &RegistryManifest) -> Vec<String> {
    manifest
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.item_type,
                RegistryItemType::Ui | RegistryItemType::Component
            )
        })
        .map(|item| item.name.clone())
        .filter(|name| !DIOXUS_ONLY_EXTRAS.contains(&name.as_str()))
        .collect()
}

/// Returns every required item name with no corresponding `parity.json`
/// component entry.
fn find_unclassified<'a>(required: &'a [String], parity: &ParityManifest) -> Vec<&'a str> {
    required
        .iter()
        .filter(|name| !parity.components.contains_key(name.as_str()))
        .map(String::as_str)
        .collect()
}

/// Enforces the conditional rules `parity.schema.json` expresses via
/// `if`/`then` (which plain serde deserialization cannot check on its own).
fn validate_parity_manifest(parity: &ParityManifest) -> Result<(), String> {
    if parity.catalog.status == "tracked"
        && (parity.catalog.source.is_none()
            || parity.catalog.revision.is_none()
            || parity.catalog.refreshed_at.is_none())
    {
        return Err(
            "catalog.status is \"tracked\" but source/revision/refreshedAt is missing".into(),
        );
    }

    for (name, component) in &parity.components {
        for (dimension_name, dimension) in component.dimensions.named() {
            if !dimension.applicable {
                if dimension.passed {
                    return Err(format!(
                        "{name}.{dimension_name} is not applicable but marked passed"
                    ));
                }
                if dimension
                    .note
                    .as_ref()
                    .is_none_or(|note| note.trim().is_empty())
                {
                    return Err(format!(
                        "{name}.{dimension_name} is not applicable and has no note"
                    ));
                }
            }
            if dimension.passed && dimension.evidence.is_empty() {
                return Err(format!(
                    "{name}.{dimension_name} is marked passed with no evidence"
                ));
            }
        }

        if component.status == "complete" {
            if component
                .registry_item
                .as_ref()
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(format!("{name} is marked complete but has no registryItem"));
            }
            for (dimension_name, dimension) in component.dimensions.named() {
                if !dimension.applicable || !dimension.passed {
                    return Err(format!(
                        "{name} is marked complete but {dimension_name} is not applicable+passed"
                    ));
                }
            }
        }
    }

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
mod parity_tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/compile/parity")
            .join(name)
    }

    fn schema_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/compile")
            .join(name)
    }

    fn load_parity(path: &Path) -> ParityManifest {
        let contents = fs::read_to_string(path).expect("fixture should be readable");
        serde_json::from_str(&contents).expect("fixture should be valid JSON")
    }

    #[test]
    fn schema_valid_fixture_passes_business_rule_validation() {
        let parity = load_parity(&schema_fixture_path("parity-valid.json"));
        validate_parity_manifest(&parity).expect("fixture is a valid parity manifest");
    }

    #[test]
    fn schema_invalid_complete_fixture_fails_business_rule_validation() {
        let parity = load_parity(&schema_fixture_path("parity-invalid-complete.json"));
        let error = validate_parity_manifest(&parity)
            .expect_err("complete status requires all dimensions passed");
        assert!(error.contains("complete"), "unexpected error: {error}");
    }

    #[test]
    fn parity_check_reports_unclassified_registry_item() {
        let error = check_parity_at(
            &fixture_path("registry-with-unclassified-item.json"),
            &fixture_path("parity-with-missing-entry.json"),
        )
        .expect_err("gadget has no parity.json entry and is not a Dioxus-only extra");
        assert!(
            error.contains("gadget"),
            "expected the unclassified 'gadget' item to be named: {error}"
        );
        assert!(
            !error.contains("widget"),
            "widget has a parity.json entry and must not be reported: {error}"
        );
        assert!(
            !error.contains("toolbar") && !error.contains("cn"),
            "toolbar (extra) and cn (lib) must not be required to have entries: {error}"
        );
    }

    #[test]
    fn classify_registry_items_excludes_extras_and_lib_items() {
        let manifest: RegistryManifest = serde_json::from_slice(
            &fs::read(fixture_path("registry-with-unclassified-item.json"))
                .expect("fixture should be readable"),
        )
        .expect("fixture should be a valid registry manifest");
        let required = classify_registry_items(&manifest);
        assert_eq!(
            required,
            vec!["widget".to_string(), "gadget".to_string()],
            "only registry:ui/registry:component items outside DIOXUS_ONLY_EXTRAS are required"
        );
    }
}
