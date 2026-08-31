//! Shared fetch machinery for the two Dioxus axes (`dioxus-components`,
//! `dioxus-primitives`), which both read subdirectories of the same
//! `DioxusLabs/dioxus-components` repository. Props on this axis live in
//! Rust `#[derive(Props)]` structs, not on any web page, so "fetch" means:
//! resolve a pinned commit sha, download a tarball at that sha, extract it,
//! then hand the tree to [`crate::rust_introspect`].

use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;

pub const REPO: &str = "dioxus-components";
pub const OWNER: &str = "DioxusLabs";

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent("adico-xtask/1.0")
        .build()
        .map_err(|error| format!("cannot build HTTP client: {error}"))
}

fn github_api_get(url: &str) -> Result<serde_json::Value, String> {
    let client = http_client()?;
    let mut request = client
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    let response = request
        .send()
        .map_err(|error| format!("cannot reach GitHub API at {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned {} for {url}",
            response.status()
        ));
    }
    response
        .json()
        .map_err(|error| format!("invalid GitHub API response from {url}: {error}"))
}

/// Resolves the commit sha to fetch: the explicit `--revision` override if
/// given, otherwise the current HEAD of the repository's default branch.
pub fn resolve_revision(revision: Option<&str>) -> Result<String, String> {
    if let Some(revision) = revision {
        return Ok(revision.to_string());
    }
    let repo_info = github_api_get(&format!("https://api.github.com/repos/{OWNER}/{REPO}"))?;
    let default_branch = repo_info
        .get("default_branch")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "GitHub API response missing default_branch".to_string())?;
    let commit = github_api_get(&format!(
        "https://api.github.com/repos/{OWNER}/{REPO}/commits/{default_branch}"
    ))?;
    commit
        .get("sha")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| "GitHub API response missing sha".to_string())
}

/// Downloads and extracts the pinned-sha tarball into a fresh temp dir,
/// returning the dir (kept alive by the caller) and the path to the single
/// top-level directory GitHub's legacy tarball format wraps everything in.
pub fn fetch_tarball(sha: &str) -> Result<(TempDir, PathBuf), String> {
    let url = format!("https://codeload.github.com/{OWNER}/{REPO}/legacy.tar.gz/{sha}");
    let client = http_client()?;
    let response = client
        .get(&url)
        .send()
        .map_err(|error| format!("cannot fetch {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "tarball fetch returned {} for {url}",
            response.status()
        ));
    }
    let bytes = response
        .bytes()
        .map_err(|error| format!("cannot read tarball body from {url}: {error}"))?;

    let dir = tempfile::tempdir().map_err(|error| format!("cannot create temp dir: {error}"))?;
    let decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dir.path())
        .map_err(|error| format!("cannot extract tarball from {url}: {error}"))?;

    let extracted_root = find_single_child_dir(dir.path()).ok_or_else(|| {
        format!("tarball from {url} did not extract to a single top-level directory")
    })?;
    Ok((dir, extracted_root))
}

fn find_single_child_dir(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.is_dir())
}

/// Resolve the revision and fetch its tarball in one call -- what every
/// axis fetcher on this repo needs.
pub fn resolve_and_fetch(revision: Option<&str>) -> Result<(String, TempDir, PathBuf), String> {
    let sha = resolve_revision(revision)?;
    let (temp_dir, extracted_root) = fetch_tarball(&sha)?;
    Ok((sha, temp_dir, extracted_root))
}

pub fn source_url() -> String {
    format!("https://github.com/{OWNER}/{REPO}")
}

/// Splits a module's [`FileIntrospection`] into per-part catalog entries by
/// matching each `pub fn Foo(...)`/`FooProps` pair against the module's own
/// PascalCase prefix (e.g. module `dialog` + component `DialogRoot` ->
/// part `root`, sourced from struct `DialogRootProps`). Best-effort: a
/// component name that doesn't start with the module's prefix still gets a
/// part, just keyed by its own full kebab-cased name.
pub fn parts_from_introspection(
    module: &str,
    introspection: &crate::rust_introspect::FileIntrospection,
) -> Vec<super::schema::PartEntry> {
    use super::case::part_id_for;
    use super::schema::{PartEntry, Prop, PropsSource};

    introspection
        .components
        .iter()
        .map(|component| {
            let part_id = part_id_for(module, component);
            let props_struct = format!("{component}Props");
            let props_source = match introspection.props.get(&props_struct) {
                Some(fields) => PropsSource::Explicit {
                    props: fields
                        .iter()
                        .map(|field| Prop {
                            name: field.name.clone(),
                            type_name: field.type_name.clone(),
                            default: field.default.clone(),
                            description: None,
                        })
                        .collect(),
                },
                None => PropsSource::Unavailable,
            };
            PartEntry {
                id: part_id,
                composition: Vec::new(),
                props_source,
            }
        })
        .collect()
}

pub use super::case::part_id_for;

#[cfg(test)]
mod tests {
    use super::*;

    /// Network-gated: fetches the real tarball at a pinned, known-good sha
    /// and asserts the extracted tree contains an expected file. Run with
    /// `cargo test -p adico-xtask -- --ignored`.
    #[test]
    #[ignore = "hits the real network (codeload.github.com)"]
    fn fetches_and_extracts_a_pinned_tarball() {
        let sha = "bf007c15d0cf4d04d3181cc46cf12325aa773955";
        let (_temp_dir, extracted_root) = fetch_tarball(sha).expect("tarball fetch succeeds");
        assert!(
            extracted_root.join("primitives/src/dialog.rs").is_file(),
            "expected primitives/src/dialog.rs under {}",
            extracted_root.display()
        );
        assert!(
            extracted_root
                .join("preview/src/components/dialog")
                .is_dir(),
            "expected preview/src/components/dialog under {}",
            extracted_root.display()
        );
    }
}
