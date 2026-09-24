//! The docs route tree: `/docs` (component list) and
//! `/docs/components/:name` (usage/accessibility/keyboard docs + an
//! introspected props table), styled through the site's Tailwind pipeline
//! and composed from installed registry components (`Card`, `Table`,
//! `Badge`) rather than a hardcoded stylesheet — see
//! `adico-web-structure`'s "The docs route tree renders through Tailwind
//! and installed registry components, not hardcoded CSS" requirement.

mod component;
mod examples;
// `pub` so the landing page can count `registry:ui` items from the same
// compile-time manifest the docs pages read, rather than repeating a literal
// that silently goes stale as the registry grows.
pub mod data;
pub mod guides;
mod index;
mod tokens;

pub use component::DocsComponent;
pub use index::DocsIndex;
