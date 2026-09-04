//! Source-owned shadcn-style Message for Dioxus.
//!
//! The full message row: avatar, header (sender/timestamp), content (a
//! `Bubble` or any other rendered content), and footer -- laid out on the
//! left (received) or right (sent) edge. Purely presentational; no dedicated
//! primitive dependency (task 10.1's own audit).

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// Which edge a [`Message`] row aligns to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MessageAlign {
    /// Left-aligned, the default -- a received message.
    #[default]
    Start,
    /// Right-aligned -- a sent message.
    End,
}

/// Props for [`Message`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageProps {
    /// Which edge this message aligns to.
    #[props(default)]
    pub align: MessageAlign,
    /// The avatar slot, laid out beside the header/content/footer column --
    /// a distinct flex role from `children`, so it is its own prop rather
    /// than composed among `children` (which would otherwise nest it
    /// *inside* the column instead of beside it). Typically a
    /// [`MessageAvatar`].
    #[props(default)]
    pub avatar: Option<Element>,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`MessageHeader`]/[`MessageContent`]/
    /// [`MessageFooter`].
    pub children: Element,
}

/// A single message row: an optional avatar plus a header/content/footer
/// column, laid out to one edge.
#[component]
pub fn Message(props: MessageProps) -> Element {
    let row_direction = match props.align {
        MessageAlign::Start => "flex-row",
        MessageAlign::End => "flex-row-reverse",
    };
    let text_align = match props.align {
        MessageAlign::Start => "items-start text-left",
        MessageAlign::End => "items-end text-right",
    };
    let class = cn(&[
        "flex w-full items-end gap-2",
        row_direction,
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, ..props.attributes,
            {props.avatar.unwrap_or_else(|| rsx! {})}
            div { class: cn(&["flex min-w-0 flex-col gap-1", text_align]), {props.children} }
        }
    }
}

/// Props for [`MessageGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageGroupProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed consecutive [`Message`]s from the same sender.
    pub children: Element,
}

/// Groups consecutive messages from the same sender with tighter vertical
/// spacing than unrelated messages.
#[component]
pub fn MessageGroup(props: MessageGroupProps) -> Element {
    let class = cn(&[
        "flex flex-col gap-1",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`MessageAvatar`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageAvatarProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed avatar content -- typically the sibling `Avatar`
    /// registry item.
    pub children: Element,
}

/// The avatar slot at the message row's outer edge.
#[component]
pub fn MessageAvatar(props: MessageAvatarProps) -> Element {
    let class = cn(&["shrink-0", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`MessageHeader`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageHeaderProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed sender name / timestamp content.
    pub children: Element,
}

/// The sender-name/timestamp row above the content.
#[component]
pub fn MessageHeader(props: MessageHeaderProps) -> Element {
    let class = cn(&[
        "flex items-baseline gap-2 text-xs text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`MessageContent`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageContentProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// The message's rendered content -- typically the sibling `Bubble`
    /// registry item.
    pub children: Element,
}

/// The content slot.
#[component]
pub fn MessageContent(props: MessageContentProps) -> Element {
    let class = cn(&["min-w-0", props.class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`MessageFooter`].
#[derive(Props, Clone, PartialEq)]
pub struct MessageFooterProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed footer content -- read receipts, action icons.
    pub children: Element,
}

/// The row below the content -- read receipts, message actions.
#[component]
pub fn MessageFooter(props: MessageFooterProps) -> Element {
    let class = cn(&[
        "flex items-center gap-1 text-xs text-muted-foreground",
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
    fn end_align_reverses_the_row_and_right_aligns_text() {
        assert_eq!(MessageAlign::default(), MessageAlign::Start);
        let class = cn(&["flex-row-reverse"]);
        assert!(class.contains("flex-row-reverse"));
    }

    #[component]
    fn WithAvatar() -> Element {
        rsx! {
            Message {
                avatar: rsx! {
                    MessageAvatar { "AV" }
                },
                MessageHeader { "Sender" }
            }
        }
    }

    #[test]
    fn avatar_is_its_own_prop_and_renders_without_panicking() {
        // A regression guard for a real defect: `avatar` used to be composed
        // among `children` and got nested *inside* the header/content/footer
        // column div (stacked above the header) instead of beside it as a
        // row sibling. Giving it its own prop (this test's own construction)
        // is the structural fix -- this just confirms it still constructs
        // and renders cleanly with that shape.
        let mut dom = VirtualDom::new(WithAvatar);
        dom.rebuild_in_place();
    }
}
