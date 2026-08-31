//! Marker-owned Tailwind/shadcn-style theme token installation.

use std::fs;
use std::path::PathBuf;

use thiserror::Error;

/// Start marker for the adico-managed semantic theme token region.
pub const THEME_REGION_START: &str = "/* adico:theme:start */";
/// End marker for the adico-managed semantic theme token region.
pub const THEME_REGION_END: &str = "/* adico:theme:end */";

/// Enter/exit animation primitives (`animate-in`/`animate-out`,
/// `fade-*`/`zoom-*`/`slide-*` state utilities, and the accordion open/close
/// keyframes) that shadcn-parity `data-[state=...]:` classes in
/// `registry/ui/*.rs` depend on. Ported from `tw-animate-css`
/// (MIT, Wombosvideo, revision `1fc30d2b5ad6ab18baf7897828e44ab20a0a62ed`) —
/// see `provenance/records/adico-cli-theme-animation-utilities.json` and
/// `third_party/tw-animate-css/LICENSE-MIT`. Narrowed to only the utilities
/// adico's registry currently references (see that record's `changes` note
/// for what was left out and why).
///
/// Deliberately kept as an attributed port rather than re-derived from
/// Tailwind v4's own `@theme`/`@keyframes`/`@utility` documentation
/// (`reauthor-primitives-from-independent-spec` task 7.1): Tailwind v4 does
/// not itself define these shadcn/Radix-convention enter/exit animation
/// names (`fade-in-0`, `zoom-in-95`, `slide-in-from-top-2`, etc.) -- they
/// are `tw-animate-css`'s (and its `tailwindcss-animate` v3 predecessor's)
/// own codification of that ecosystem convention. "Re-deriving from
/// Tailwind v4 docs" would only mean re-typing the same numeric values
/// (opacity/scale/translate per named utility) from memory of the same
/// external convention, without a live-rendered animation available this
/// session to catch a subtly wrong value -- more execution risk for no
/// reduction in what's actually being attributed. This is this crate's one
/// remaining provenance record by design, not an oversight.
const ANIMATION_UTILITIES_CSS: &str = r#"
@property --tw-animation-delay {
  syntax: "*";
  inherits: false;
  initial-value: 0s;
}

@property --tw-animation-direction {
  syntax: "*";
  inherits: false;
  initial-value: normal;
}

@property --tw-animation-duration {
  syntax: "*";
  inherits: false;
}

@property --tw-animation-fill-mode {
  syntax: "*";
  inherits: false;
  initial-value: none;
}

@property --tw-animation-iteration-count {
  syntax: "*";
  inherits: false;
  initial-value: 1;
}

@property --tw-enter-opacity {
  syntax: "*";
  inherits: false;
  initial-value: 1;
}

@property --tw-enter-scale {
  syntax: "*";
  inherits: false;
  initial-value: 1;
}

@property --tw-enter-translate-x {
  syntax: "*";
  inherits: false;
  initial-value: 0;
}

@property --tw-enter-translate-y {
  syntax: "*";
  inherits: false;
  initial-value: 0;
}

@property --tw-exit-opacity {
  syntax: "*";
  inherits: false;
  initial-value: 1;
}

@property --tw-exit-scale {
  syntax: "*";
  inherits: false;
  initial-value: 1;
}

@property --tw-exit-translate-x {
  syntax: "*";
  inherits: false;
  initial-value: 0;
}

@property --tw-exit-translate-y {
  syntax: "*";
  inherits: false;
  initial-value: 0;
}

@theme inline {
  --percentage-0: 0;
  --percentage-5: 0.05;
  --percentage-10: 0.1;
  --percentage-15: 0.15;
  --percentage-20: 0.2;
  --percentage-25: 0.25;
  --percentage-30: 0.3;
  --percentage-35: 0.35;
  --percentage-40: 0.4;
  --percentage-45: 0.45;
  --percentage-50: 0.5;
  --percentage-55: 0.55;
  --percentage-60: 0.6;
  --percentage-65: 0.65;
  --percentage-70: 0.7;
  --percentage-75: 0.75;
  --percentage-80: 0.8;
  --percentage-85: 0.85;
  --percentage-90: 0.9;
  --percentage-95: 0.95;
  --percentage-100: 1;
  --percentage-translate-full: 1;

  --animate-in: enter var(--tw-animation-duration, var(--tw-duration, 150ms)) var(--tw-ease, ease)
    var(--tw-animation-delay, 0s) var(--tw-animation-iteration-count, 1)
    var(--tw-animation-direction, normal) var(--tw-animation-fill-mode, none);
  --animate-out: exit var(--tw-animation-duration, var(--tw-duration, 150ms)) var(--tw-ease, ease)
    var(--tw-animation-delay, 0s) var(--tw-animation-iteration-count, 1)
    var(--tw-animation-direction, normal) var(--tw-animation-fill-mode, none);

  @keyframes enter {
    from {
      opacity: var(--tw-enter-opacity, 1);
      transform: translate3d(var(--tw-enter-translate-x, 0), var(--tw-enter-translate-y, 0), 0)
        scale3d(var(--tw-enter-scale, 1), var(--tw-enter-scale, 1), var(--tw-enter-scale, 1));
    }
  }

  @keyframes exit {
    to {
      opacity: var(--tw-exit-opacity, 1);
      transform: translate3d(var(--tw-exit-translate-x, 0), var(--tw-exit-translate-y, 0), 0)
        scale3d(var(--tw-exit-scale, 1), var(--tw-exit-scale, 1), var(--tw-exit-scale, 1));
    }
  }

}

/* `--animate-accordion-down`/`-up` (and their keyframes, which animate
   `height: 0` to/from `height: var(--radix-accordion-content-height, auto)`)
   were deliberately left out here after being tried and reverted in M4 task
   5.3a: adico's AccordionContent primitive never sets a measured
   content-height custom property (unlike Radix's own implementation), so the
   keyframe's `auto` fallback does not interpolate in Chromium -- verified
   live via dx serve to leave the content permanently stuck at height: 0
   (invisible) once "open", a regression, not an improvement. Re-adding these
   requires primitive-layer work first: either JS-measured height tracking
   like Radix's, or restructuring to the CSS grid 0fr/1fr animatable-track
   technique. Tracked as a gap for 5.4 (primitive hardening). */

@utility fade-in {
  --tw-enter-opacity: 0;
}
@utility fade-in-* {
  --tw-enter-opacity: calc(--value(number) / 100);
  --tw-enter-opacity: --value(--percentage-*, [*]);
}

@utility fade-out {
  --tw-exit-opacity: 0;
}
@utility fade-out-* {
  --tw-exit-opacity: calc(--value(number) / 100);
  --tw-exit-opacity: --value(--percentage-*, [*]);
}

@utility zoom-in {
  --tw-enter-scale: 0;
}
@utility zoom-in-* {
  --tw-enter-scale: calc(--value(number) * 1%);
  --tw-enter-scale: calc(--value(ratio));
  --tw-enter-scale: --value(--percentage-*, [*]);
}

@utility zoom-out {
  --tw-exit-scale: 0;
}
@utility zoom-out-* {
  --tw-exit-scale: calc(--value(number) * 1%);
  --tw-exit-scale: calc(--value(ratio));
  --tw-exit-scale: --value(--percentage-*, [*]);
}

@utility slide-in-from-top {
  --tw-enter-translate-y: -100%;
}
@utility slide-in-from-top-* {
  --tw-enter-translate-y: calc(--value(integer) * var(--spacing) * -1);
  --tw-enter-translate-y: calc(--value(--percentage-*, --percentage-translate-*) * -100%);
  --tw-enter-translate-y: calc(--value(ratio) * -100%);
  --tw-enter-translate-y: calc(--value(--translate-*, [percentage], [length]) * -1);
}
@utility slide-in-from-bottom {
  --tw-enter-translate-y: 100%;
}
@utility slide-in-from-bottom-* {
  --tw-enter-translate-y: calc(--value(integer) * var(--spacing));
  --tw-enter-translate-y: calc(--value(--percentage-*, --percentage-translate-*) * 100%);
  --tw-enter-translate-y: calc(--value(ratio) * 100%);
  --tw-enter-translate-y: --value(--translate-*, [percentage], [length]);
}
@utility slide-in-from-left {
  --tw-enter-translate-x: -100%;
}
@utility slide-in-from-left-* {
  --tw-enter-translate-x: calc(--value(integer) * var(--spacing) * -1);
  --tw-enter-translate-x: calc(--value(--percentage-*, --percentage-translate-*) * -100%);
  --tw-enter-translate-x: calc(--value(ratio) * -100%);
  --tw-enter-translate-x: calc(--value(--translate-*, [percentage], [length]) * -1);
}
@utility slide-in-from-right {
  --tw-enter-translate-x: 100%;
}
@utility slide-in-from-right-* {
  --tw-enter-translate-x: calc(--value(integer) * var(--spacing));
  --tw-enter-translate-x: calc(--value(--percentage-*, --percentage-translate-*) * 100%);
  --tw-enter-translate-x: calc(--value(ratio) * 100%);
  --tw-enter-translate-x: --value(--translate-*, [percentage], [length]);
}

@utility slide-out-to-top {
  --tw-exit-translate-y: -100%;
}
@utility slide-out-to-top-* {
  --tw-exit-translate-y: calc(--value(integer) * var(--spacing) * -1);
  --tw-exit-translate-y: calc(--value(--percentage-*, --percentage-translate-*) * -100%);
  --tw-exit-translate-y: calc(--value(ratio) * -100%);
  --tw-exit-translate-y: calc(--value(--translate-*, [percentage], [length]) * -1);
}
@utility slide-out-to-bottom {
  --tw-exit-translate-y: 100%;
}
@utility slide-out-to-bottom-* {
  --tw-exit-translate-y: calc(--value(integer) * var(--spacing));
  --tw-exit-translate-y: calc(--value(--percentage-*, --percentage-translate-*) * 100%);
  --tw-exit-translate-y: calc(--value(ratio) * 100%);
  --tw-exit-translate-y: --value(--translate-*, [percentage], [length]);
}
@utility slide-out-to-left {
  --tw-exit-translate-x: -100%;
}
@utility slide-out-to-left-* {
  --tw-exit-translate-x: calc(--value(integer) * var(--spacing) * -1);
  --tw-exit-translate-x: calc(--value(--percentage-*, --percentage-translate-*) * -100%);
  --tw-exit-translate-x: calc(--value(ratio) * -100%);
  --tw-exit-translate-x: calc(--value(--translate-*, [percentage], [length]) * -1);
}
@utility slide-out-to-right {
  --tw-exit-translate-x: 100%;
}
@utility slide-out-to-right-* {
  --tw-exit-translate-x: calc(--value(integer) * var(--spacing));
  --tw-exit-translate-x: calc(--value(--percentage-*, --percentage-translate-*) * 100%);
  --tw-exit-translate-x: calc(--value(ratio) * 100%);
  --tw-exit-translate-x: --value(--translate-*, [percentage], [length]);
}
"#;

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

/// The full shadcn-style semantic token set: every color a registry
/// component's Tailwind classes reference (`bg-accent`, `bg-card`,
/// `bg-popover`, `bg-sidebar*`, `border-input`, `ring-ring`, etc.), not just
/// the background/foreground/primary trio a single Button/Badge needs. A
/// narrower set left those classes resolving to nothing under Tailwind v4
/// (no `@theme` token registered), so components using them rendered
/// unstyled even with the CSS pipeline otherwise wired up correctly.
fn theme_region() -> String {
    format!(
        "{THEME_REGION_START}\n\
@theme {{\n\
\x20 --color-background: hsl(var(--background));\n\
\x20 --color-foreground: hsl(var(--foreground));\n\
\x20 --color-card: hsl(var(--card));\n\
\x20 --color-card-foreground: hsl(var(--card-foreground));\n\
\x20 --color-popover: hsl(var(--popover));\n\
\x20 --color-popover-foreground: hsl(var(--popover-foreground));\n\
\x20 --color-primary: hsl(var(--primary));\n\
\x20 --color-primary-foreground: hsl(var(--primary-foreground));\n\
\x20 --color-secondary: hsl(var(--secondary));\n\
\x20 --color-secondary-foreground: hsl(var(--secondary-foreground));\n\
\x20 --color-muted: hsl(var(--muted));\n\
\x20 --color-muted-foreground: hsl(var(--muted-foreground));\n\
\x20 --color-accent: hsl(var(--accent));\n\
\x20 --color-accent-foreground: hsl(var(--accent-foreground));\n\
\x20 --color-destructive: hsl(var(--destructive));\n\
\x20 --color-destructive-foreground: hsl(var(--destructive-foreground));\n\
\x20 --color-border: hsl(var(--border));\n\
\x20 --color-input: hsl(var(--input));\n\
\x20 --color-ring: hsl(var(--ring));\n\
\x20 --color-sidebar: hsl(var(--sidebar-background));\n\
\x20 --color-sidebar-foreground: hsl(var(--sidebar-foreground));\n\
\x20 --color-sidebar-primary: hsl(var(--sidebar-primary));\n\
\x20 --color-sidebar-primary-foreground: hsl(var(--sidebar-primary-foreground));\n\
\x20 --color-sidebar-accent: hsl(var(--sidebar-accent));\n\
\x20 --color-sidebar-accent-foreground: hsl(var(--sidebar-accent-foreground));\n\
\x20 --color-sidebar-border: hsl(var(--sidebar-border));\n\
\x20 --color-sidebar-ring: hsl(var(--sidebar-ring));\n\
\x20 --radius-sm: calc(var(--radius) - 4px);\n\
\x20 --radius-md: calc(var(--radius) - 2px);\n\
\x20 --radius-lg: var(--radius);\n\
}}\n\
\n\
:root {{\n\
\x20 --background: 0 0% 100%;\n\
\x20 --foreground: 222.2 84% 4.9%;\n\
\x20 --card: 0 0% 100%;\n\
\x20 --card-foreground: 222.2 84% 4.9%;\n\
\x20 --popover: 0 0% 100%;\n\
\x20 --popover-foreground: 222.2 84% 4.9%;\n\
\x20 --primary: 222.2 47.4% 11.2%;\n\
\x20 --primary-foreground: 210 40% 98%;\n\
\x20 --secondary: 210 40% 96.1%;\n\
\x20 --secondary-foreground: 222.2 47.4% 11.2%;\n\
\x20 --muted: 210 40% 96.1%;\n\
\x20 --muted-foreground: 215.4 16.3% 46.9%;\n\
\x20 --accent: 210 40% 96.1%;\n\
\x20 --accent-foreground: 222.2 47.4% 11.2%;\n\
\x20 --destructive: 0 84.2% 60.2%;\n\
\x20 --destructive-foreground: 210 40% 98%;\n\
\x20 --border: 214.3 31.8% 91.4%;\n\
\x20 --input: 214.3 31.8% 91.4%;\n\
\x20 --ring: 222.2 84% 4.9%;\n\
\x20 --radius: 0.5rem;\n\
\x20 --sidebar-background: 0 0% 98%;\n\
\x20 --sidebar-foreground: 240 5.3% 26.1%;\n\
\x20 --sidebar-primary: 240 5.9% 10%;\n\
\x20 --sidebar-primary-foreground: 0 0% 98%;\n\
\x20 --sidebar-accent: 240 4.8% 95.9%;\n\
\x20 --sidebar-accent-foreground: 240 5.9% 10%;\n\
\x20 --sidebar-border: 220 13% 91%;\n\
\x20 --sidebar-ring: 217.2 91.2% 59.8%;\n\
}}\n\
\n\
.dark {{\n\
\x20 --background: 222.2 84% 4.9%;\n\
\x20 --foreground: 210 40% 98%;\n\
\x20 --card: 222.2 84% 4.9%;\n\
\x20 --card-foreground: 210 40% 98%;\n\
\x20 --popover: 222.2 84% 4.9%;\n\
\x20 --popover-foreground: 210 40% 98%;\n\
\x20 --primary: 210 40% 98%;\n\
\x20 --primary-foreground: 222.2 47.4% 11.2%;\n\
\x20 --secondary: 217.2 32.6% 17.5%;\n\
\x20 --secondary-foreground: 210 40% 98%;\n\
\x20 --muted: 217.2 32.6% 17.5%;\n\
\x20 --muted-foreground: 215 20.2% 65.1%;\n\
\x20 --accent: 217.2 32.6% 17.5%;\n\
\x20 --accent-foreground: 210 40% 98%;\n\
\x20 --destructive: 0 62.8% 30.6%;\n\
\x20 --destructive-foreground: 210 40% 98%;\n\
\x20 --border: 217.2 32.6% 17.5%;\n\
\x20 --input: 217.2 32.6% 17.5%;\n\
\x20 --ring: 212.7 26.8% 83.9%;\n\
\x20 --sidebar-background: 240 5.9% 10%;\n\
\x20 --sidebar-foreground: 240 4.8% 95.9%;\n\
\x20 --sidebar-primary: 224.3 76.3% 48%;\n\
\x20 --sidebar-primary-foreground: 0 0% 100%;\n\
\x20 --sidebar-accent: 240 3.7% 15.9%;\n\
\x20 --sidebar-accent-foreground: 240 4.8% 95.9%;\n\
\x20 --sidebar-border: 240 3.7% 15.9%;\n\
\x20 --sidebar-ring: 217.2 91.2% 59.8%;\n\
}}\n\
{ANIMATION_UTILITIES_CSS}\
{THEME_REGION_END}\n"
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_project_root() -> PathBuf {
        // A nanosecond timestamp alone can collide: this repo's tests run
        // with multiple threads, and two calls close together can land in
        // the same clock tick on platforms with coarser-than-nanosecond
        // resolution, so two tests race on the same directory. The counter
        // guarantees every call gets a distinct path regardless of timing.
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("valid time")
            .as_nanos();
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "adico-css-test-{}-{nonce}-{unique}",
            std::process::id()
        ))
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
