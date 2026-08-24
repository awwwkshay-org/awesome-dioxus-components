//! Repository automation for Awesome Dioxus Components.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

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

fn main() {
    let arguments: Vec<_> = env::args().skip(1).collect();
    match arguments.as_slice() {
        [command, subcommand] if command == "provenance" && subcommand == "check" => {
            if let Err(error) = check_provenance() {
                eprintln!("provenance check failed: {error}");
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
        _ => {
            eprintln!(
                "usage:\n  cargo xtask provenance check\n  cargo xtask upstream dioxus-components\n  cargo xtask upstream dioxus-components --source <local-clone> --refreshed-at <YYYY-MM-DD> [--write]"
            );
            std::process::exit(2);
        }
    }
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
