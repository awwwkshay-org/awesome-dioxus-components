//! End-to-end `adico` binary coverage: multi-item add, repeated add, shared
//! dependencies, Cargo conflicts, file conflicts, malformed modules, dry-plan
//! output, incompatible registry sources, and source-lock refresh behavior.
//!
//! Every fixture project depends on a local path-based `dioxus` stub so
//! `cargo metadata --offline` never touches the network; only the CLI's own
//! embedded official registry (`@adico`) is exercised for real component
//! source.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMPORARY_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A standalone consumer-style project usable by `adico` without network
/// access. Removed on drop so failed assertions still clean up.
struct FixtureProject {
    root: PathBuf,
}

impl Drop for FixtureProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl FixtureProject {
    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path(relative))
            .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
    }

    fn exists(&self, relative: &str) -> bool {
        self.path(relative).exists()
    }

    fn adico(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_adico"))
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("adico should execute")
    }
}

/// Creates an isolated project whose `dioxus` dependency is a local path
/// crate, so discovery and Cargo edits never require network access. The
/// dependency's declared `version` requirement is configurable so tests can
/// produce an offline-resolvable Cargo conflict against the official
/// registry's `=0.7.9` requirement.
fn fixture_project(dioxus_version_requirement: &str) -> FixtureProject {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid system time")
        .as_nanos();
    let sequence = TEMPORARY_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "adico-cli-integration-{}-{nonce}-{sequence}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("src")).expect("source directory should be created");
    fs::create_dir_all(root.join("dioxus_stub/src")).expect("stub directory should be created");
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"consumer\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n\n[dependencies]\ndioxus = {{ path = \"dioxus_stub\", version = \"{dioxus_version_requirement}\" }}\n"
        ),
    )
    .expect("consumer manifest should be written");
    fs::write(root.join("src/main.rs"), "fn main() {}\n")
        .expect("consumer entrypoint should be written");
    fs::write(
        root.join("dioxus_stub/Cargo.toml"),
        "[package]\nname = \"dioxus\"\nversion = \"0.7.9\"\nedition = \"2024\"\n",
    )
    .expect("stub manifest should be written");
    fs::write(
        root.join("dioxus_stub/src/lib.rs"),
        "//! offline dioxus stub\n",
    )
    .expect("stub source should be written");
    FixtureProject { root }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn init(project: &FixtureProject) {
    let output = project.adico(&["init"]);
    assert!(
        output.status.success(),
        "adico init should succeed: {}",
        stderr(&output)
    );
}

#[test]
fn multi_item_add_installs_shared_dependencies_once() {
    let project = fixture_project("=0.7.9");
    init(&project);

    let output = project.adico(&["add", "button", "dialog"]);
    assert!(
        output.status.success(),
        "multi-item add should succeed: {}",
        stderr(&output)
    );
    let report = stdout(&output);
    assert!(report.contains("@adico/button"));
    assert!(report.contains("@adico/dialog"));
    assert!(report.contains("@adico/cn"));
    assert!(report.contains("adico add complete."));

    assert!(project.exists("src/adico_lib/cn.rs"));
    assert!(project.exists("src/components/ui/button.rs"));
    assert!(project.exists("src/components/ui/dialog.rs"));

    let lock = project.read("adico.lock");
    assert!(lock.contains("\"address\": \"@adico/button\""));
    assert!(lock.contains("\"address\": \"@adico/cn\""));
    assert!(lock.contains("\"address\": \"@adico/dialog\""));
    // The shared "cn" dependency is a single lock entry, not one per consumer.
    assert_eq!(lock.matches("\"address\": \"@adico/cn\"").count(), 1);

    let manifest = project.read("Cargo.toml");
    assert!(
        manifest.contains("dioxus_stub"),
        "the original dioxus dependency must survive"
    );
    assert!(manifest.contains("adico-primitives"));
    assert!(manifest.contains("=0.1.0"));
}

/// The same registry fixture the `adico` binary installs end-to-end into
/// `tests/installation/awwwkshay-consumer`, embedded here so an offline
/// company-registry consumer can be configured without a second copy.
const AWWWKSHAY_REGISTRY_MANIFEST: &str =
    include_str!("../../../tests/installation/awwwkshay-consumer/awwwkshay-registry/registry.json");
const AWWWKSHAY_CARD_SOURCE: &str =
    include_str!("../../../tests/installation/awwwkshay-consumer/awwwkshay-registry/ui/card.rs");

fn write_awwwkshay_registry(project: &FixtureProject) {
    fs::create_dir_all(project.path("awwwkshay-registry/ui"))
        .expect("awwwkshay registry directory should be created");
    fs::write(
        project.path("awwwkshay-registry/registry.json"),
        AWWWKSHAY_REGISTRY_MANIFEST,
    )
    .expect("awwwkshay registry manifest should be written");
    fs::write(
        project.path("awwwkshay-registry/ui/card.rs"),
        AWWWKSHAY_CARD_SOURCE,
    )
    .expect("awwwkshay registry source should be written");
}

#[test]
fn bare_company_default_and_explicit_official_items_install_together() {
    let project = fixture_project("=0.7.9");
    write_awwwkshay_registry(&project);

    let init_output = project.adico(&[
        "init",
        "--default-registry",
        "@awwwkshay",
        "--registry",
        "@awwwkshay=awwwkshay-registry",
    ]);
    assert!(
        init_output.status.success(),
        "company-default init should succeed: {}",
        stderr(&init_output)
    );

    // "card" is bare and resolves through the configured @awwwkshay default;
    // "@adico/button" is explicit and resolves through the official registry
    // regardless of that default. Card also carries an explicit
    // "@adico/cn" cross-registry dependency, so a single add exercises both
    // namespace-selection rules at once.
    let output = project.adico(&["add", "card", "@adico/button"]);
    assert!(
        output.status.success(),
        "mixed bare/explicit add should succeed: {}",
        stderr(&output)
    );
    let report = stdout(&output);
    assert!(report.contains("@awwwkshay/card"));
    assert!(report.contains("@adico/button"));
    assert!(report.contains("@adico/cn"));

    assert!(project.exists("src/components/ui/card.rs"));
    assert!(project.exists("src/components/ui/button.rs"));
    assert!(project.exists("src/adico_lib/cn.rs"));

    let lock = project.read("adico.lock");
    assert!(lock.contains("\"address\": \"@awwwkshay/card\""));
    assert!(lock.contains("\"address\": \"@adico/button\""));
    assert!(lock.contains("\"address\": \"@adico/cn\""));
}

#[test]
fn repeated_add_is_idempotent() {
    let project = fixture_project("=0.7.9");
    init(&project);

    let first = project.adico(&["add", "button"]);
    assert!(
        first.status.success(),
        "first add should succeed: {}",
        stderr(&first)
    );

    let button_after_first = project.read("src/components/ui/button.rs");
    let lock_after_first = project.read("adico.lock");
    let manifest_after_first = project.read("Cargo.toml");

    let second = project.adico(&["add", "button"]);
    assert!(
        second.status.success(),
        "repeated add of an unchanged component should succeed: {}",
        stderr(&second)
    );

    assert_eq!(
        project.read("src/components/ui/button.rs"),
        button_after_first
    );
    assert_eq!(project.read("adico.lock"), lock_after_first);
    assert_eq!(project.read("Cargo.toml"), manifest_after_first);
}

#[test]
fn cargo_dependency_conflict_reports_without_mutating_project() {
    // The stub's own declared requirement ("0.7.9") is a valid literal string
    // for the path dependency, but it differs from the official Button
    // item's exact "=0.7.9" requirement string, which the installer refuses
    // to silently rewrite.
    let project = fixture_project("0.7.9");
    init(&project);

    let manifest_before = project.read("Cargo.toml");
    let output = project.adico(&["add", "button"]);
    assert!(
        !output.status.success(),
        "a Cargo version conflict must fail the add"
    );
    assert!(stderr(&output).contains("conflicts"), "{}", stderr(&output));

    assert_eq!(
        project.read("Cargo.toml"),
        manifest_before,
        "Cargo.toml must not change"
    );
    assert!(!project.exists("src/components/ui/button.rs"));
    assert!(!project.exists("adico.lock"));
}

#[test]
fn file_conflict_is_reported_without_overwrite() {
    let project = fixture_project("=0.7.9");
    init(&project);

    let first = project.adico(&["add", "button"]);
    assert!(
        first.status.success(),
        "initial add should succeed: {}",
        stderr(&first)
    );

    let button_path = "src/components/ui/button.rs";
    fs::write(project.path(button_path), "// consumer-modified button\n")
        .expect("consumer edit should be written");
    let lock_before = project.read("adico.lock");

    let second = project.adico(&["add", "button"]);
    assert!(
        !second.status.success(),
        "a modified installed file must block re-install"
    );
    assert!(
        stderr(&second).contains("refusing to overwrite"),
        "{}",
        stderr(&second)
    );

    assert_eq!(project.read(button_path), "// consumer-modified button\n");
    assert_eq!(
        project.read("adico.lock"),
        lock_before,
        "lock must not change on failure"
    );
}

#[test]
fn malformed_module_marker_is_rejected() {
    let project = fixture_project("=0.7.9");
    init(&project);

    let first = project.adico(&["add", "button"]);
    assert!(
        first.status.success(),
        "initial add should succeed: {}",
        stderr(&first)
    );

    let mod_path = "src/components/ui/mod.rs";
    let mut corrupted = project.read(mod_path);
    corrupted.push_str("// adico:start\n// adico:end\n");
    fs::write(project.path(mod_path), &corrupted).expect("corrupted module should be written");

    let output = project.adico(&["add", "button"]);
    assert!(
        !output.status.success(),
        "duplicate markers must block installation"
    );
    assert!(stderr(&output).contains("malformed"), "{}", stderr(&output));
    assert_eq!(
        project.read(mod_path),
        corrupted,
        "the corrupted module must be left untouched"
    );
}

#[test]
fn dry_run_add_reports_plan_without_writing_files() {
    let project = fixture_project("=0.7.9");
    init(&project);

    let output = project.adico(&["add", "button", "--dry-run"]);
    assert!(
        output.status.success(),
        "dry-run should succeed: {}",
        stderr(&output)
    );
    let report = stdout(&output);
    assert!(report.contains("adico add plan:"));
    assert!(report.contains("@adico/button"));
    assert!(!report.contains("adico add complete."));

    assert!(!project.exists("src/components/ui/button.rs"));
    assert!(!project.exists("adico.lock"));
}

#[test]
fn incompatible_registry_source_is_rejected() {
    let project = fixture_project("=0.7.9");

    let registry_root = project.path("bad-registry");
    fs::create_dir_all(&registry_root).expect("registry directory should be created");
    fs::write(
        registry_root.join("registry.json"),
        r#"{"formatVersion":1,"namespace":"@badcorp","name":"incompatible fixture","compatibility":{"cli":">=99.0.0"},"items":[]}"#,
    )
    .expect("incompatible registry manifest should be written");

    let output = project.adico(&["init", "--registry", "@badcorp=bad-registry"]);
    assert!(
        output.status.success(),
        "init only records configuration and must not validate registries: {}",
        stderr(&output)
    );

    let list = project.adico(&["list"]);
    assert!(
        !list.status.success(),
        "an incompatible configured registry must block commands"
    );
    let message = stderr(&list);
    assert!(message.contains("@badcorp"), "{message}");
    assert!(message.contains(">=99.0.0"), "{message}");
    assert!(message.contains("this adico build supports"), "{message}");
}

#[test]
fn source_lock_refresh_merges_items_across_separate_add_invocations() {
    let project = fixture_project("=0.7.9");
    init(&project);

    let first = project.adico(&["add", "button"]);
    assert!(
        first.status.success(),
        "first add should succeed: {}",
        stderr(&first)
    );
    let lock_after_button = project.read("adico.lock");
    assert!(lock_after_button.contains("\"address\": \"@adico/button\""));
    assert!(lock_after_button.contains("\"address\": \"@adico/cn\""));
    assert!(!lock_after_button.contains("\"address\": \"@adico/dialog\""));

    let second = project.adico(&["add", "dialog"]);
    assert!(
        second.status.success(),
        "second add should succeed: {}",
        stderr(&second)
    );
    let lock_after_dialog = project.read("adico.lock");

    // The refreshed lock retains items from the first invocation and adds the
    // second invocation's items, rather than overwriting the file.
    assert!(lock_after_dialog.contains("\"address\": \"@adico/button\""));
    assert!(lock_after_dialog.contains("\"address\": \"@adico/dialog\""));
    assert_eq!(
        lock_after_dialog
            .matches("\"address\": \"@adico/cn\"")
            .count(),
        1
    );
    assert!(project.exists("src/components/ui/button.rs"));
    assert!(project.exists("src/components/ui/dialog.rs"));
}

/// Sanity check on the fixture helper itself: two independent fixtures never
/// collide on disk.
#[test]
fn fixtures_are_isolated_per_test() {
    let a = fixture_project("=0.7.9");
    let b = fixture_project("=0.7.9");
    assert_ne!(a.root, b.root);
    let _ = Path::new(&a.root);
    let _ = Path::new(&b.root);
}
