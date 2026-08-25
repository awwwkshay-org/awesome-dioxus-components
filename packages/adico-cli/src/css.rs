//! Marker-owned Tailwind/shadcn-style theme token installation.

use std::fs;
use std::path::PathBuf;

use thiserror::Error;

/// Start marker for the adico-managed semantic theme token region.
pub const THEME_REGION_START: &str = "/* adico:theme:start */";
/// End marker for the adico-managed semantic theme token region.
pub const THEME_REGION_END: &str = "/* adico:theme:end */";

/// A reviewable CSS file update that never touches content outside markers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssThemePlan {
    /// Consumer CSS entry point.
    pub path: PathBuf,
    /// Complete new contents when an edit is required.
    pub contents: Option<String>,
}

impl CssThemePlan {
    /// Returns whether applying this plan changes the CSS entry.
    pub fn has_changes(&self) -> bool {
        self.contents.is_some()
    }

    /// Applies the already reviewed token-only update.
    pub fn apply(&self) -> Result<(), CssThemeError> {
        let Some(contents) = &self.contents else {
            return Ok(());
        };
        let parent = self
            .path
            .parent()
            .ok_or_else(|| CssThemeError::WriteFailed {
                path: self.path.display().to_string(),
                message: "CSS entry has no parent directory".to_string(),
            })?;
        fs::create_dir_all(parent).map_err(|error| CssThemeError::WriteFailed {
            path: parent.display().to_string(),
            message: error.to_string(),
        })?;
        fs::write(&self.path, contents).map_err(|error| CssThemeError::WriteFailed {
            path: self.path.display().to_string(),
            message: error.to_string(),
        })
    }
}

/// Computes the relative `@source` scan path from a CSS entry's directory
/// back to the project's `src/` tree, e.g. `assets/tailwind.css` under a
/// project root yields `../src`.
fn relative_source_directive(project_root: &std::path::Path, css_path: &std::path::Path) -> String {
    let css_dir = css_path.parent().unwrap_or(project_root);
    let depth = css_dir
        .strip_prefix(project_root)
        .map(|relative| relative.components().count())
        .unwrap_or(1);
    format!("{}src", "../".repeat(depth))
}

/// Plans idempotent shadcn-style semantic color, dark-mode, and radius tokens.
///
/// When the CSS entry does not exist yet (the common case: `adico init`
/// only reserves the path, and the first `adico add` that needs tokens is
/// what actually creates the file), this also bootstraps the Tailwind v4
/// `@import`/`@source` lines the entry needs to compile at all. An existing,
/// non-empty file is assumed to already have its own bootstrap and is left
/// untouched outside the managed marker region, matching the tokens
/// themselves.
pub fn plan_theme_install(
    path: impl Into<PathBuf>,
    project_root: impl AsRef<std::path::Path>,
) -> Result<CssThemePlan, CssThemeError> {
    let path = path.into();
    let source_directive = relative_source_directive(project_root.as_ref(), &path);
    let existing = match path
        .try_exists()
        .map_err(|error| CssThemeError::ReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })? {
        false => String::new(),
        true => fs::read_to_string(&path).map_err(|error| CssThemeError::ReadFailed {
            path: path.display().to_string(),
            message: error.to_string(),
        })?,
    };
    let token_region = theme_region();
    let updated = match (
        positions(&existing, THEME_REGION_START),
        positions(&existing, THEME_REGION_END),
    ) {
        (starts, ends) if starts.is_empty() && ends.is_empty() => {
            if existing.is_empty() {
                format!(
                    "@import \"tailwindcss\";\n@source \"{source_directive}\";\n\n{token_region}"
                )
            } else {
                format!("{existing}\n{token_region}")
            }
        }
        (starts, ends) if starts.len() == 1 && ends.len() == 1 && starts[0] < ends[0] => {
            let after_marker = ends[0] + THEME_REGION_END.len();
            let suffix_start = if existing[after_marker..].starts_with("\r\n") {
                after_marker + 2
            } else if existing[after_marker..].starts_with('\n') {
                after_marker + 1
            } else {
                after_marker
            };
            format!(
                "{}{}{}",
                &existing[..starts[0]],
                token_region,
                &existing[suffix_start..]
            )
        }
        _ => return Err(CssThemeError::MalformedThemeRegion),
    };
    Ok(CssThemePlan {
        path,
        contents: (updated != existing).then_some(updated),
    })
}

fn positions(contents: &str, marker: &str) -> Vec<usize> {
    contents
        .match_indices(marker)
        .map(|(index, _)| index)
        .collect()
}

fn theme_region() -> String {
    format!(
        "{THEME_REGION_START}\n@theme {{\n  --color-background: hsl(var(--background));\n  --color-foreground: hsl(var(--foreground));\n  --color-primary: hsl(var(--primary));\n  --color-primary-foreground: hsl(var(--primary-foreground));\n  --radius-sm: calc(var(--radius) - 4px);\n  --radius-md: calc(var(--radius) - 2px);\n  --radius-lg: var(--radius);\n}}\n\n:root {{\n  --background: 0 0% 100%;\n  --foreground: 222.2 84% 4.9%;\n  --primary: 222.2 47.4% 11.2%;\n  --primary-foreground: 210 40% 98%;\n  --radius: 0.5rem;\n}}\n\n.dark {{\n  --background: 222.2 84% 4.9%;\n  --foreground: 210 40% 98%;\n  --primary: 210 40% 98%;\n  --primary-foreground: 222.2 47.4% 11.2%;\n}}\n{THEME_REGION_END}\n"
    )
}

/// CSS token planning errors that preserve consumer styles on failure.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum CssThemeError {
    #[error("cannot read CSS entry {path}: {message}")]
    ReadFailed { path: String, message: String },
    #[error("CSS entry has malformed or duplicate adico theme markers")]
    MalformedThemeRegion,
    #[error("cannot write CSS entry {path}: {message}")]
    WriteFailed { path: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_project_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        std::env::temp_dir().join(format!("adico-css-test-{}-{nonce}", std::process::id()))
    }

    fn temporary_css_path() -> PathBuf {
        temporary_project_root().join("assets/tailwind.css")
    }

    #[test]
    fn bootstraps_tailwind_import_and_source_directive_for_a_fresh_css_entry() {
        let path = temporary_css_path();
        let project_root = path
            .ancestors()
            .nth(2)
            .expect("temporary root should exist")
            .to_path_buf();
        // No fs::write: the entry does not exist yet, matching `adico init`
        // only reserving the path and the first themed `adico add` creating
        // the file for real.
        let plan = plan_theme_install(&path, &project_root).expect("theme should plan");
        plan.apply().expect("theme should apply");
        let created = fs::read_to_string(&path).expect("created CSS should be readable");
        assert!(created.starts_with("@import \"tailwindcss\";\n@source \"../src\";\n\n"));
        assert!(created.contains("--radius: 0.5rem;"));
        assert!(
            !plan_theme_install(&path, &project_root)
                .expect("repeated install should plan")
                .has_changes()
        );
        fs::remove_dir_all(&project_root).expect("temporary directory should be removable");
    }

    #[test]
    fn installs_light_dark_and_radius_tokens_once_without_touching_user_css() {
        let path = temporary_css_path();
        let project_root = path
            .ancestors()
            .nth(2)
            .expect("temporary root should exist")
            .to_path_buf();
        fs::create_dir_all(path.parent().expect("parent should exist"))
            .expect("parent should be created");
        fs::write(
            &path,
            "@import \"tailwindcss\";\n\n.consumer { color: red; }\n",
        )
        .expect("fixture CSS should be written");
        let plan = plan_theme_install(&path, &project_root).expect("theme should plan");
        plan.apply().expect("theme should apply");
        let updated = fs::read_to_string(&path).expect("updated CSS should be readable");
        assert!(updated.contains(".consumer { color: red; }"));
        assert!(updated.contains("--radius: 0.5rem;"));
        assert!(updated.contains(".dark"));
        assert!(
            !plan_theme_install(&path, &project_root)
                .expect("repeated install should plan")
                .has_changes()
        );
        fs::remove_dir_all(&project_root).expect("temporary directory should be removable");
    }

    #[test]
    fn rejects_duplicate_markers_before_writing_css() {
        let path = temporary_css_path();
        let project_root = path
            .ancestors()
            .nth(2)
            .expect("temporary root should exist")
            .to_path_buf();
        fs::create_dir_all(path.parent().expect("parent should exist"))
            .expect("parent should be created");
        fs::write(&path, format!("{THEME_REGION_START}\n{THEME_REGION_END}\n{THEME_REGION_START}\n{THEME_REGION_END}\n")).expect("fixture CSS should be written");
        assert_eq!(
            plan_theme_install(&path, &project_root).expect_err("duplicate markers must fail"),
            CssThemeError::MalformedThemeRegion
        );
        fs::remove_dir_all(&project_root).expect("temporary directory should be removable");
    }
}
