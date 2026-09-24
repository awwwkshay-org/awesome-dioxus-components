//! The docs route tree: `/docs` (component list) and
//! `/docs/components/:name` (usage/accessibility/keyboard docs + an
//! introspected props table), styled through the site's Tailwind pipeline
//! and composed from installed registry components (`Card`, `Table`,
//! `Badge`) rather than a hardcoded stylesheet — see
//! `adico-web-structure`'s "The docs route tree renders through Tailwind
//! and installed registry components, not hardcoded CSS" requirement.

mod component;
mod data;
mod index;

pub use component::DocsComponent;
pub use index::DocsIndex;
