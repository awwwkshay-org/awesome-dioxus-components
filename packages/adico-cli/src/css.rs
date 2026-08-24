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

/// Plans idempotent shadcn-style semantic color, dark-mode, and radius tokens.
pub fn plan_theme_install(path: impl Into<PathBuf>) -> Result<CssThemePlan, CssThemeError> {
    let path = path.into();
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
                token_region
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

    fn temporary_css_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        std::env::temp_dir()
            .join(format!("adico-css-test-{}-{nonce}", std::process::id()))
            .join("assets/tailwind.css")
    }

    #[test]
    fn installs_light_dark_and_radius_tokens_once_without_touching_user_css() {
        let path = temporary_css_path();
        fs::create_dir_all(path.parent().expect("parent should exist"))
            .expect("parent should be created");
        fs::write(
            &path,
            "@import \"tailwindcss\";\n\n.consumer { color: red; }\n",
        )
        .expect("fixture CSS should be written");
        let plan = plan_theme_install(&path).expect("theme should plan");
        plan.apply().expect("theme should apply");
        let updated = fs::read_to_string(&path).expect("updated CSS should be readable");
        assert!(updated.contains(".consumer { color: red; }"));
        assert!(updated.contains("--radius: 0.5rem;"));
        assert!(updated.contains(".dark"));
        assert!(
            !plan_theme_install(&path)
                .expect("repeated install should plan")
                .has_changes()
        );
        fs::remove_dir_all(
            path.ancestors()
                .nth(2)
                .expect("temporary root should exist"),
        )
        .expect("temporary directory should be removable");
    }

    #[test]
    fn rejects_duplicate_markers_before_writing_css() {
        let path = temporary_css_path();
        fs::create_dir_all(path.parent().expect("parent should exist"))
            .expect("parent should be created");
        fs::write(&path, format!("{THEME_REGION_START}\n{THEME_REGION_END}\n{THEME_REGION_START}\n{THEME_REGION_END}\n")).expect("fixture CSS should be written");
        assert_eq!(
            plan_theme_install(&path).expect_err("duplicate markers must fail"),
            CssThemeError::MalformedThemeRegion
        );
        fs::remove_dir_all(
            path.ancestors()
                .nth(2)
                .expect("temporary root should exist"),
        )
        .expect("temporary directory should be removable");
    }
}
