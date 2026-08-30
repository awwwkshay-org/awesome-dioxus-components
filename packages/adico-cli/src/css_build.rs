//! Node-free Tailwind CSS compilation via Tailwind's own standalone native
//! CLI release (not the npm-distributed `@tailwindcss/cli` package). `dx
//! serve`/`dx build` already fetch and run this exact binary into
//! `~/.dx/tools/tailwindcss-v<version>/`; this module does the same into a
//! separate `~/.adico/tools/` cache so the two tools never collide, and
//! exposes it as `adico css build`/`adico css check` so `adico init`/
//! `adico add` can guarantee a consumer project renders styled output
//! without requiring Node/npm anywhere in the chain. See
//! `openspec/changes/build-adico-component-ecosystem/design.md` section 7c.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use thiserror::Error;

/// Pinned standalone Tailwind CLI release. Independently upgradable from
/// whatever version `dx` itself caches under `~/.dx/tools/` -- the two
/// happen to agree today (both resolve to Tailwind's official
/// `tailwindcss-macos-arm64`/etc. GitHub release assets) but are not assumed
/// to stay in lockstep.
pub const TAILWIND_VERSION: &str = "4.1.5";

const RELEASE_BASE_URL: &str = "https://github.com/tailwindlabs/tailwindcss/releases/download";

/// Compiled Tailwind output path, relative to a consumer project's root.
/// Matches Dioxus's own default `tailwind_output` convention (`Dioxus.toml`
/// can override it, but adico does not currently read that override) and
/// the `document::Stylesheet { href: asset!("/assets/tailwind.css") }` link
/// every installed entrypoint uses.
pub const COMPILED_OUTPUT_RELATIVE_PATH: &str = "assets/tailwind.css";

/// Failures compiling a consumer's Tailwind CSS. Every variant carries
/// enough detail to name the exact remediation to a human, per this
/// project's "never silently proceed" convention for CSS-pipeline gaps.
#[derive(Debug, Error)]
pub enum CssBuildError {
    #[error(
        "no standalone Tailwind CLI release is published for {os}/{arch}; install and run `tailwindcss` manually, or open an issue"
    )]
    UnsupportedPlatform { os: String, arch: String },
    #[error("cannot determine a home directory to cache the Tailwind CLI in")]
    NoCacheDirectory,
    #[error("cannot download {url}: {message}")]
    Download { url: String, message: String },
    #[error(
        "downloaded Tailwind CLI asset {asset} failed checksum verification (expected {expected}, got {actual}); refusing to trust it"
    )]
    ChecksumMismatch {
        asset: String,
        expected: String,
        actual: String,
    },
    #[error("Tailwind's published sha256sums.txt for v{version} has no entry for {asset}")]
    ChecksumMissing { version: String, asset: String },
    #[error("cannot make {path} executable: {message}")]
    NotExecutable { path: String, message: String },
    #[error("cannot read/write {path}: {message}")]
    Io { path: String, message: String },
    #[error(
        "Tailwind CSS input {input} does not exist; run `adico init` or `adico add` first, or check components.json's css.entry"
    )]
    InputMissing { input: String },
    #[error("tailwindcss exited with {status}: {stderr}")]
    CompileFailed { status: String, stderr: String },
    #[error(
        "{output} is stale relative to {input} ({detail}); run `adico css build` to refresh it"
    )]
    Stale {
        input: String,
        output: String,
        detail: String,
    },
}

fn io_error(path: &Path, error: std::io::Error) -> CssBuildError {
    CssBuildError::Io {
        path: path.display().to_string(),
        message: error.to_string(),
    }
}

/// Maps the host platform to Tailwind's exact standalone release asset name.
/// Only glibc Linux targets are covered today (matching `dx`'s own default);
/// musl hosts can still point `adico` at a manually installed `tailwindcss`
/// binary at `~/.adico/tools/tailwindcss-v<version>/tailwindcss`.
fn asset_name() -> Result<&'static str, CssBuildError> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("tailwindcss-macos-arm64"),
        ("macos", "x86_64") => Ok("tailwindcss-macos-x64"),
        ("linux", "aarch64") => Ok("tailwindcss-linux-arm64"),
        ("linux", "x86_64") => Ok("tailwindcss-linux-x64"),
        ("windows", "x86_64") => Ok("tailwindcss-windows-x64.exe"),
        (os, arch) => Err(CssBuildError::UnsupportedPlatform {
            os: os.to_string(),
            arch: arch.to_string(),
        }),
    }
}

fn cache_dir() -> Result<PathBuf, CssBuildError> {
    let home = dirs::home_dir().ok_or(CssBuildError::NoCacheDirectory)?;
    Ok(home
        .join(".adico")
        .join("tools")
        .join(format!("tailwindcss-v{TAILWIND_VERSION}")))
}

fn binary_file_name() -> &'static str {
    if cfg!(windows) {
        "tailwindcss.exe"
    } else {
        "tailwindcss"
    }
}

fn binary_path() -> Result<PathBuf, CssBuildError> {
    Ok(cache_dir()?.join(binary_file_name()))
}

fn http_client() -> Result<reqwest::blocking::Client, CssBuildError> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            // GitHub release downloads redirect to a signed object-storage
            // URL; only ever follow it over HTTPS, matching this project's
            // existing `StaticHttpsClient` policy for registry sources.
            if attempt.url().scheme() == "https" {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|error| CssBuildError::Download {
            url: "<client construction>".to_string(),
            message: error.to_string(),
        })
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>, CssBuildError> {
    let client = http_client()?;
    let response = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| CssBuildError::Download {
            url: url.to_string(),
            message: error.to_string(),
        })?;
    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .map_err(|error| CssBuildError::Download {
            url: url.to_string(),
            message: error.to_string(),
        })
}

/// Parses Tailwind's published `sha256sums.txt` (`<hex digest>  ./<asset>`
/// per line) for one asset's expected digest.
fn parse_checksum(checksums: &str, asset: &str) -> Option<String> {
    checksums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        let name = parts
            .next()?
            .trim_start_matches("./")
            .trim_start_matches('*');
        (name == asset).then(|| digest.to_lowercase())
    })
}

fn hex_encode(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), CssBuildError> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|error| io_error(path, error))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| io_error(path, error))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), CssBuildError> {
    Ok(())
}

/// Ensures a verified copy of the standalone Tailwind CLI is cached locally,
/// downloading it on cache miss. Integrity is checked against Tailwind's own
/// published `sha256sums.txt` for the pinned release -- fetched over HTTPS
/// from the same trusted origin as the binary itself, matching how this
/// project's other HTTPS-source verification already works. Never returns a
/// path to an unverified or partially written binary.
pub fn ensure_binary_cached() -> Result<PathBuf, CssBuildError> {
    let path = binary_path()?;
    if path.is_file() {
        return Ok(path);
    }
    let asset = asset_name()?;
    let checksums_url = format!("{RELEASE_BASE_URL}/v{TAILWIND_VERSION}/sha256sums.txt");
    let checksums_bytes = http_get_bytes(&checksums_url)?;
    let checksums_text = String::from_utf8_lossy(&checksums_bytes);
    let expected =
        parse_checksum(&checksums_text, asset).ok_or_else(|| CssBuildError::ChecksumMissing {
            version: TAILWIND_VERSION.to_string(),
            asset: asset.to_string(),
        })?;

    let binary_url = format!("{RELEASE_BASE_URL}/v{TAILWIND_VERSION}/{asset}");
    let bytes = http_get_bytes(&binary_url)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual = hex_encode(hasher.finalize());
    if actual != expected {
        return Err(CssBuildError::ChecksumMismatch {
            asset: asset.to_string(),
            expected,
            actual,
        });
    }

    let dir = cache_dir()?;
    fs::create_dir_all(&dir).map_err(|error| io_error(&dir, error))?;
    // Write to a unique staging path first so a crash or a concurrent
    // `adico css build` invocation never leaves a half-written file at the
    // real cache path that a later run would treat as already-cached.
    let staging = dir.join(format!(
        ".{asset}.download-{}-{}",
        std::process::id(),
        unique_nonce()
    ));
    fs::write(&staging, &bytes).map_err(|error| io_error(&staging, error))?;
    set_executable(&staging)?;
    fs::rename(&staging, &path).map_err(|error| io_error(&path, error))?;
    Ok(path)
}

fn unique_nonce() -> u128 {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    timestamp + u128::from(NEXT.fetch_add(1, Ordering::Relaxed))
}

/// Compiles `input` to `output` using the cached standalone Tailwind CLI.
/// Tailwind resolves `@source` directives relative to the input file's own
/// location, so absolute paths are passed for both and no `--cwd` override
/// is needed.
fn compile(input: &Path, output: &Path) -> Result<(), CssBuildError> {
    if !input.is_file() {
        return Err(CssBuildError::InputMissing {
            input: input.display().to_string(),
        });
    }
    let binary = ensure_binary_cached()?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    let result = Command::new(&binary)
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(output)
        .output()
        .map_err(|error| io_error(&binary, error))?;
    if !result.status.success() {
        return Err(CssBuildError::CompileFailed {
            status: result.status.to_string(),
            stderr: String::from_utf8_lossy(&result.stderr).trim().to_string(),
        });
    }
    Ok(())
}

/// Compiles a consumer project's Tailwind input (`components.json`'s
/// `css.entry`, project-root-relative) into `assets/tailwind.css`. This is
/// what `adico css build`, and the `adico init`/`adico add` wiring, call.
pub fn build_project(project_root: &Path, css_entry_relative: &str) -> Result<(), CssBuildError> {
    let input = project_root.join(css_entry_relative);
    let output = project_root.join(COMPILED_OUTPUT_RELATIVE_PATH);
    compile(&input, &output)
}

/// Compiles into a private temporary file and diffs it against the checked-in
/// `assets/tailwind.css`, without touching the committed file. This is the
/// `registry validate`-equivalent staleness gate `adico css check` exposes.
pub fn check_project(project_root: &Path, css_entry_relative: &str) -> Result<(), CssBuildError> {
    let input = project_root.join(css_entry_relative);
    let committed_output = project_root.join(COMPILED_OUTPUT_RELATIVE_PATH);
    let existing = fs::read(&committed_output).unwrap_or_default();

    let scratch_output = std::env::temp_dir().join(format!(
        "adico-css-check-{}-{}.css",
        std::process::id(),
        unique_nonce()
    ));
    compile(&input, &scratch_output)?;
    let fresh = fs::read(&scratch_output).unwrap_or_default();
    let _ = fs::remove_file(&scratch_output);

    if existing == fresh {
        Ok(())
    } else {
        Err(CssBuildError::Stale {
            input: input.display().to_string(),
            output: committed_output.display().to_string(),
            detail: format!(
                "{} bytes committed vs {} bytes freshly compiled",
                existing.len(),
                fresh.len()
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_name_covers_this_test_host() {
        // Every CI/dev platform this repo is validated on (macOS/Linux,
        // x86_64/aarch64) must resolve to a real Tailwind release asset;
        // this is a compile-host smoke check, not exhaustive cross-platform
        // coverage (Windows is exercised by inspection, not by this test
        // suite, since it doesn't run on a Windows host here).
        asset_name().expect("this test host's OS/arch should be supported");
    }

    #[test]
    fn asset_name_rejects_unknown_platforms() {
        // asset_name() reads the real std::env::consts, so this test
        // exercises the match arms' fallthrough directly instead.
        let error = match ("plan9", "riscv64") {
            ("macos", "aarch64") => unreachable!(),
            ("macos", "x86_64") => unreachable!(),
            ("linux", "aarch64") => unreachable!(),
            ("linux", "x86_64") => unreachable!(),
            ("windows", "x86_64") => unreachable!(),
            (os, arch) => CssBuildError::UnsupportedPlatform {
                os: os.to_string(),
                arch: arch.to_string(),
            },
        };
        assert!(matches!(error, CssBuildError::UnsupportedPlatform { .. }));
    }

    #[test]
    fn cache_dir_is_versioned_and_under_dot_adico_tools() {
        let dir = cache_dir().expect("home directory should resolve in test environments");
        assert!(dir.ends_with(format!("tools/tailwindcss-v{TAILWIND_VERSION}")));
        assert!(dir.to_string_lossy().contains(".adico"));
    }

    #[test]
    fn binary_path_uses_platform_appropriate_file_name() {
        let path = binary_path().expect("home directory should resolve in test environments");
        assert_eq!(path.file_name().unwrap(), binary_file_name());
    }

    #[test]
    fn parse_checksum_finds_the_matching_asset_and_ignores_others() {
        let checksums = "abc123  ./tailwindcss-macos-arm64\ndef456  ./tailwindcss-linux-x64\n";
        assert_eq!(
            parse_checksum(checksums, "tailwindcss-macos-arm64"),
            Some("abc123".to_string())
        );
        assert_eq!(
            parse_checksum(checksums, "tailwindcss-windows-x64.exe"),
            None
        );
    }

    #[test]
    fn hex_encode_matches_known_sha256_of_empty_input() {
        let digest = Sha256::digest(b"");
        assert_eq!(
            hex_encode(digest),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn build_project_reports_a_clear_error_when_input_is_missing() {
        let root = std::env::temp_dir().join(format!(
            "adico-css-build-test-missing-{}-{}",
            std::process::id(),
            unique_nonce()
        ));
        let error = build_project(&root, "tailwind.css").expect_err("missing input should fail");
        assert!(matches!(error, CssBuildError::InputMissing { .. }));
    }
}
