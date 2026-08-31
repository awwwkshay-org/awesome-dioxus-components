// SPDX-License-Identifier: MIT OR Apache-2.0

//! Text-direction context shared by primitives whose keyboard or layout
//! behavior depends on writing direction (roving focus, drag deltas, grid
//! navigation). Modeled on Base UI's Direction Provider: a component reads
//! [`use_direction`] and gets [`Direction::Ltr`] unless a [`DirectionProvider`]
//! ancestor set otherwise — most consumers never need to render one.

use dioxus::prelude::*;

/// Text direction for keyboard-navigation and layout purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Left-to-right, the default when no [`DirectionProvider`] is present.
    #[default]
    Ltr,
    /// Right-to-left.
    Rtl,
}

impl Direction {
    /// Whether this direction is right-to-left.
    pub fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

/// The props for the [`DirectionProvider`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DirectionProviderProps {
    /// The direction descendant primitives should read via [`use_direction`].
    pub direction: Direction,
    /// The children that can read the provided direction.
    pub children: Element,
}

/// # Direction Provider
///
/// Provides a [`Direction`] to every descendant primitive that calls
/// [`use_direction`]. Nesting a provider inside another overrides the
/// direction for its own subtree only.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::direction::{Direction, DirectionProvider};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         DirectionProvider {
///             direction: Direction::Rtl,
///             "content that reads use_direction() as Rtl"
///         }
///     }
/// }
/// ```
#[component]
pub fn DirectionProvider(props: DirectionProviderProps) -> Element {
    use_context_provider(|| props.direction);

    rsx! {
        {props.children}
    }
}

/// Read the nearest ancestor [`DirectionProvider`]'s direction, or
/// [`Direction::Ltr`] if none is present.
pub fn use_direction() -> Direction {
    try_use_context::<Direction>().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn direction_label(direction: Direction) -> &'static str {
        if direction.is_rtl() { "rtl" } else { "ltr" }
    }

    #[component]
    fn Observer() -> Element {
        rsx! {
            span { "{direction_label(use_direction())}" }
        }
    }

    #[component]
    fn WithoutProvider() -> Element {
        rsx! {
            Observer {}
        }
    }

    #[component]
    fn WithRtlProvider() -> Element {
        rsx! {
            DirectionProvider {
                direction: Direction::Rtl,
                Observer {}
            }
        }
    }

    #[component]
    fn NestedProviderOverridesInnerSubtree() -> Element {
        rsx! {
            DirectionProvider {
                direction: Direction::Rtl,
                div { class: "outer", Observer {} }
                DirectionProvider {
                    direction: Direction::Ltr,
                    div { class: "inner", Observer {} }
                }
            }
        }
    }

    fn render(root: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(root);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn defaults_to_ltr_without_a_provider() {
        assert!(render(WithoutProvider).contains("ltr"));
    }

    #[test]
    fn provider_overrides_the_default() {
        assert!(render(WithRtlProvider).contains("rtl"));
    }

    #[test]
    fn nested_provider_overrides_only_its_own_subtree() {
        let html = render(NestedProviderOverridesInnerSubtree);
        let outer = html.split("outer").nth(1).expect("outer subtree present");
        let inner = html.split("inner").nth(1).expect("inner subtree present");

        assert!(outer.contains("rtl"));
        assert!(inner.contains("ltr"));
    }
}
