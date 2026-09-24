//! The site's page tree: the landing page (`index.rs`, `/`), the docs route
//! tree (`docs/`, `/docs/*`), the playground route tree (`playground/`,
//! `/playground/*`), and the two shell-free viewport-harness routes
//! (`responsive_flow.rs`, `responsive_overlay.rs`), which stay at this top
//! level — not under `playground/` — because their routes render outside
//! every layout, `SiteLayout` included (see `routes.rs`).

pub mod docs;
pub mod playground;

mod index;
mod responsive_flow;
mod responsive_overlay;

pub use index::Home;
pub use responsive_flow::ResponsiveFlowPage;
pub use responsive_overlay::ResponsiveOverlayPage;
