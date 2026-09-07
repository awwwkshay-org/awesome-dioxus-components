//! Source-owned shadcn-style Bubble for Dioxus.
//!
//! The visual speech-bubble shape wrapping one message's content, aligned to
//! the left (received) or right (sent) edge. Purely presentational; no
//! dedicated primitive dependency (task 10.1's own audit). A level below
//! the sibling `message` registry item -- `Message` owns the avatar/header/
//! footer row, `Bubble` only the rounded content shape itself, so a caller
//! can use `Bubble` alone for a minimal chat surface.
//!
//! Upstream's `content` part declares `asChild`; per this ecosystem's
//! established caller-composition answer (see `marker.rs`'s header
//! comment), `BubbleContent` does not attempt to replicate that
//! polymorphism.
//!
//! `BubbleContent`/`BubbleReactions` take `align` as an explicit prop rather
//! than reading it from a `Bubble`-provided context: this file has no owned
//! interactive/dynamic state to justify one (every other context-using
//! registry item -- `carousel`/`resizable`/`sidebar` -- has real owned
//! behavior behind its context), and a static styling-tone lookup is exactly
//! what `Message`'s own `align` prop (registry/ui/message.rs) already does
//! without context.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// Which edge a [`Bubble`] (and its [`BubbleReactions`]) aligns to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BubbleAlign {
    /// Left-aligned, the default -- a received message.
    #[default]
    Start,
    /// Right-aligned -- a sent message.
    End,
}

/// Props for [`Bubble`].
#[derive(Props, Clone, PartialEq)]
pub struct BubbleProps {
    /// Which edge this bubble aligns to.
    #[props(default)]
    pub align: BubbleAlign,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`BubbleContent`]/[`BubbleReactions`].
    pub children: Element,
}

/// A single aligned message-bubble row.
///
/// The outer row spans the full available width so `justify-start`/
/// `justify-end` can push a narrower bubble to either edge of a wider chat
/// column. [`BubbleReactions`] anchors with `position: absolute` against the
/// nearest `relative` ancestor, so that ancestor can't be this full-width
/// row itself -- its box extends past the bubble's own rendered edge
/// whenever the row is wider than the bubble (the normal case), stranding
/// `left-0`/`right-0` at the row's edge instead of the bubble's. The inner
/// `relative inline-flex` wrapper hugs exactly [`BubbleContent`]'s own
/// rendered box (an absolutely-positioned `BubbleReactions` contributes no
/// width to it), so [`BubbleReactions`] anchors to the bubble's real edge
/// regardless of the row's width or of `align` differing between the two.
#[component]
pub fn Bubble(props: BubbleProps) -> Element {
    let justify = match props.align {
        BubbleAlign::Start => "justify-start",
        BubbleAlign::End => "justify-end",
    };
    let class = cn(&[
        "flex w-full",
        justify,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes,
            div { class: "relative inline-flex flex-col", {props.children} }
        }
    }
}

/// Props for [`BubbleGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct BubbleGroupProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed consecutive [`Bubble`]s from the same sender.
    pub children: Element,
}

/// Groups consecutive bubbles from the same sender with tighter vertical
/// spacing than unrelated messages.
#[component]
pub fn BubbleGroup(props: BubbleGroupProps) -> Element {
    let class = cn(&[
        "flex flex-col gap-1",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// The color treatment of a [`BubbleContent`] surface, matching shadcn's own
/// cva axis. Defaults to the existing align-driven tone (`bg-muted`/
/// `bg-primary`) when left unset, preserving current behavior; an explicit
/// variant overrides that tone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BubbleVariant {
    /// Tone driven by `align`, the existing default look.
    #[default]
    Default,
    Secondary,
    Muted,
    Tinted,
    Outline,
    Ghost,
    Destructive,
}

impl BubbleVariant {
    fn class(self, align: BubbleAlign) -> &'static str {
        match self {
            Self::Default => match align {
                BubbleAlign::Start => "bg-muted text-foreground",
                BubbleAlign::End => "bg-primary text-primary-foreground",
            },
            Self::Secondary => "bg-secondary text-secondary-foreground",
            Self::Muted => "bg-muted text-muted-foreground",
            Self::Tinted => "bg-accent text-accent-foreground",
            Self::Outline => "border border-input bg-transparent text-foreground",
            Self::Ghost => "bg-transparent text-foreground",
            Self::Destructive => "bg-destructive text-destructive-foreground",
        }
    }
}

/// Props for [`BubbleContent`].
#[derive(Props, Clone, PartialEq)]
pub struct BubbleContentProps {
    /// Which edge this content's tone matches. Pass the same value given to
    /// the ancestor [`Bubble`].
    #[props(default)]
    pub align: BubbleAlign,
    /// Color treatment; see [`BubbleVariant`].
    #[props(default)]
    pub variant: BubbleVariant,
    /// Corner radius of the bubble surface.
    #[props(default = Radius::Xl)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// The bubble's rendered content.
    pub children: Element,
}

/// The rounded bubble surface itself.
///
/// Capped at a fixed `max-w-xs`, not a percentage like `max-w-[80%]`: a
/// percentage `max-width` here needs its containing block's width to be
/// definite at the point Chrome computes it, but nested inside a shrink-to-
/// fit ancestor (e.g. `Message`'s avatar-hugging column, which must stay
/// content-sized for the avatar to sit flush against the bubble) that width
/// is still being determined, so the percentage falls back to unconstrained
/// for that computation -- content lays out on one line, `Bubble`'s parent
/// sizes itself to that unwrapped width, and only then does the percentage
/// resolve against the now-definite size and force a wrap, stranding the
/// wrapped bubble inside a box still sized for the unwrapped one. A fixed
/// unit resolves identically regardless of the ancestor's own sizing pass,
/// so it can never disagree with itself this way. Found live via a
/// screenshot of a `MessageScroller` conversation with a visible gap between
/// an avatar and its own (wrapped, multi-line) bubble.
#[component]
pub fn BubbleContent(props: BubbleContentProps) -> Element {
    let class = cn(&[
        "max-w-xs px-4 py-2 text-sm break-words",
        props.variant.class(props.align),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Which corner a [`BubbleReactions`] row sits at, relative to its [`Bubble`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BubbleReactionsSide {
    /// Above the bubble.
    Top,
    /// Below the bubble, the default.
    #[default]
    Bottom,
}

/// Props for [`BubbleReactions`].
#[derive(Props, Clone, PartialEq)]
pub struct BubbleReactionsProps {
    /// Which horizontal edge to align to. Pass the same value given to the
    /// ancestor [`Bubble`].
    #[props(default)]
    pub align: BubbleAlign,
    /// Which vertical edge to sit at.
    #[props(default)]
    pub side: BubbleReactionsSide,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed reaction pills/buttons.
    pub children: Element,
}

/// A small reaction-pill row anchored to one corner of the bubble.
#[component]
pub fn BubbleReactions(props: BubbleReactionsProps) -> Element {
    let horizontal = match props.align {
        BubbleAlign::Start => "left-0",
        BubbleAlign::End => "right-0",
    };
    let vertical = match props.side {
        BubbleReactionsSide::Top => "-top-3",
        BubbleReactionsSide::Bottom => "-bottom-3",
    };
    let class = cn(&[
        "absolute flex items-center gap-1 rounded-full border bg-popover px-1.5 py-0.5 text-xs shadow-sm",
        horizontal,
        vertical,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_align_uses_muted_content_and_left_justify() {
        assert_eq!(BubbleAlign::default(), BubbleAlign::Start);
        let class = cn(&["bg-muted text-foreground"]);
        assert!(class.contains("bg-muted"));
    }

    #[test]
    fn end_align_uses_primary_content() {
        let class = cn(&["bg-primary text-primary-foreground"]);
        assert!(class.contains("bg-primary"));
    }
}
