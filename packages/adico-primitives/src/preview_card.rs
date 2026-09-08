// Preview Card is a thin facade over `hover_card.rs` (`openspec/changes/deduplicate-primitives`,
// D2), not a second implementation: both are hover/focus-triggered, `Positioner`-placed
// disclosures over rich, potentially-interactive content, and Base UI's own `PreviewCard.Trigger`
// differs from this crate's `HoverCard` only in its defaults -- a `delay`/`closeDelay` (Base UI:
// 600ms/300ms, vs `HoverCard`'s own 0/0) so moving the pointer from the trigger toward the popup
// doesn't close it before the popup's own `on_mouse_enter` cancels the close, a `side: Bottom`
// default (vs `HoverCard`'s `Top`), and no ARIA `role="tooltip"` on its content (`HoverCard`'s
// default; discouraged for Preview Card's richer, potentially-interactive content by the ARIA
// tooltip role's own spec). `hover_card.rs` now accepts all of these as props (`delay_ms`,
// `close_delay_ms`, content `role: Option<&'static str>`), so this module supplies its own
// defaults for them rather than re-implementing the delay/generation/`Positioner` machinery a
// second time -- the one genuine behavioral difference this file used to own independently
// (its own hand-rolled `PreviewCardCtx::request_open`'s generation-counter debounce; this
// module's earlier doc called the technique `use_open_close_delay`, but no function of that
// name ever existed here) is now `crate::hover_intent`, shared with `menu.rs` and
// `navigation_menu.rs` (D1).
//
// `hover_card.rs`'s own module doc comment documents a real, pre-existing defect this facade
// inherits unchanged (task 7.3, not fixed here -- a behavior change belongs in its own change):
// `force_mount` never actually keeps closed content mounted on any target, since
// `use_animated_open` never honors it. `PreviewCardContent::force_mount` (default `true`, same
// as `HoverCardContent`'s) carries the identical gap.

//! Defines the [`PreviewCard`] component and its subcomponents.

use dioxus::prelude::*;

use crate::{
    ContentAlign, ContentSide,
    hover_card::{HoverCard, HoverCardContent, HoverCardTrigger},
};

/// The props for the [`PreviewCard`] component
#[derive(Props, Clone, PartialEq)]
pub struct PreviewCardProps {
    /// Whether the preview card is open
    pub open: ReadSignal<Option<bool>>,

    /// Default open state
    #[props(default)]
    pub default_open: bool,

    /// Callback when open state changes
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether the preview card is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Milliseconds to wait after the pointer enters the trigger before
    /// opening. Defaults to 600, matching Base UI's own default (this
    /// facade's own default, distinct from [`crate::hover_card::HoverCard`]'s
    /// `0`).
    #[props(default = ReadSignal::new(Signal::new(600)))]
    pub delay_ms: ReadSignal<u64>,

    /// Milliseconds to wait after the pointer leaves the trigger (or
    /// content) before closing. Defaults to 300, matching Base UI's own
    /// default (this facade's own default, distinct from
    /// [`crate::hover_card::HoverCard`]'s `0`).
    #[props(default = ReadSignal::new(Signal::new(300)))]
    pub close_delay_ms: ReadSignal<u64>,

    /// Additional attributes for the preview card
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the preview card
    pub children: Element,
}

/// # PreviewCard
///
/// The `PreviewCard` component wraps a [`PreviewCardTrigger`] and a
/// [`PreviewCardContent`]. It shows a rich preview after hovering (or
/// focusing) the trigger for [`PreviewCardProps::delay_ms`], distinct from
/// [`crate::hover_card::HoverCard`]'s instant-by-default open/close (a facade
/// over the same implementation with different defaults -- see this module's
/// own doc comment).
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::{
///     ContentAlign, ContentSide,
///     preview_card::{PreviewCard, PreviewCardContent, PreviewCardTrigger},
/// };
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         PreviewCard {
///             PreviewCardTrigger {
///                 a { href: "#", "@dioxuslabs" }
///             }
///             PreviewCardContent {
///                 side: ContentSide::Bottom,
///                 div { "Dioxus is the Rust framework for building fullstack web, desktop, and mobile apps." }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`PreviewCard`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the current state of the preview card. Values are `open` or `closed`.
/// - `data-disabled`: Indicates whether the item is disabled. Values are `true` or `false`.
#[component]
pub fn PreviewCard(props: PreviewCardProps) -> Element {
    rsx! {
        HoverCard {
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            disabled: props.disabled,
            delay_ms: props.delay_ms,
            close_delay_ms: props.close_delay_ms,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`PreviewCardTrigger`] component
#[derive(Props, Clone, PartialEq)]
pub struct PreviewCardTriggerProps {
    /// Optional ID for the trigger element
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes for the preview card trigger
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the preview card trigger
    pub children: Element,
}

/// # PreviewCardTrigger
///
/// The [`PreviewCardTrigger`] component triggers the [`PreviewCardContent`]
/// to appear, after [`PreviewCardProps::delay_ms`], when hovered or
/// focused.
///
/// This component must be used inside a [`PreviewCard`] component.
#[component]
pub fn PreviewCardTrigger(props: PreviewCardTriggerProps) -> Element {
    rsx! {
        HoverCardTrigger { id: props.id, attributes: props.attributes, {props.children} }
    }
}

/// The props for the [`PreviewCardContent`] component
#[derive(Props, Clone, PartialEq)]
pub struct PreviewCardContentProps {
    /// Optional ID for the preview card content
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Side of the trigger to place the preview card
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,

    /// Alignment of the preview card relative to the trigger
    #[props(default = ContentAlign::Center)]
    pub align: ContentAlign,

    /// Whether to keep the preview card mounted even when closed
    #[props(default = true)]
    pub force_mount: bool,

    /// Additional attributes for the preview card content
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the preview card content
    pub children: Element,
}

/// # PreviewCardContent
///
/// The [`PreviewCardContent`] component defines the content of the parent
/// [`PreviewCard`]. Unlike [`crate::hover_card::HoverCardContent`] (which
/// uses `role="tooltip"`, matching a short text label), this renders with
/// no tooltip role -- its content is expected to be rich and potentially
/// interactive, which the ARIA tooltip role's own spec discourages.
///
/// This component must be used inside a [`PreviewCard`] component.
///
/// ## Styling
///
/// The [`PreviewCardContent`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the state of the preview card. Values are `open` or `closed`.
/// - `data-side`: Indicates the side of the trigger where the preview card is placed. Values are `top`, `right`, `bottom`, or `left`.
/// - `data-align`: Indicates the alignment of the preview card relative to the trigger. Values are `start`, `center`, or `end`.
#[component]
pub fn PreviewCardContent(props: PreviewCardContentProps) -> Element {
    rsx! {
        HoverCardContent {
            id: props.id,
            side: props.side,
            align: props.align,
            force_mount: props.force_mount,
            role: None,
            attributes: props.attributes,
            {props.children}
        }
    }
}
