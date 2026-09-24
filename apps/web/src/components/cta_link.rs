//! A plain anchor styled to match the landing page's call-to-action
//! treatment. `Button` (installed under `ui/`) renders a native `<button>`
//! and has no link mode, and `NavigationMenuLink` requires a
//! `NavigationMenu` ancestor context, so neither fits a standalone
//! cross-app/external CTA link. This applies ordinary Tailwind utility
//! classes using the project's existing semantic tokens; it does not
//! duplicate any registry component's interactive behavior.

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CtaLinkVariant {
    Primary,
    Outline,
    Ghost,
}

impl CtaLinkVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
            Self::Outline => {
                "border border-border bg-background hover:bg-accent hover:text-accent-foreground"
            }
            Self::Ghost => "hover:bg-accent hover:text-accent-foreground",
        }
    }
}

#[component]
pub fn CtaLink(href: String, variant: CtaLinkVariant, children: Element) -> Element {
    let class = format!(
        "inline-flex h-9 items-center justify-center rounded-md px-4 text-sm font-medium transition-colors {}",
        variant.class(),
    );
    rsx! {
        a { href, class, {children} }
    }
}
