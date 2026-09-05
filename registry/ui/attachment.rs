//! Source-owned shadcn-style Attachment for Dioxus.
//!
//! A file-attachment card: thumbnail/icon, title/description, a
//! `state`-driven status indicator, and optional actions (remove, retry).
//! Purely presentational; no dedicated primitive dependency (task 10.1's own
//! audit) -- `state` is a plain caller-supplied enum, not an owned state
//! machine.
//!
//! Upstream's `trigger` part declares `asChild`; per this ecosystem's
//! established caller-composition answer (see `marker.rs`'s header
//! comment), [`AttachmentTrigger`] is instead its own real `<button>` (the
//! same "IS a button, does not wrap one" shape as `DialogTrigger`/
//! `SheetTrigger` -- see `dropdown_menu.rs`'s `DropdownMenuTrigger` doc
//! comment for why that distinction matters).

use dioxus::prelude::*;

use adico_primitives::icons::{CircleAlert, CircleCheck, Paperclip};

use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::spinner::Spinner;

/// The upload/processing state of an [`Attachment`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AttachmentState {
    /// No transfer in progress.
    #[default]
    Idle,
    /// Actively uploading.
    Uploading,
    /// Uploaded, server-side processing in progress.
    Processing,
    /// Upload or processing failed.
    Error,
    /// Complete.
    Done,
}

impl AttachmentState {
    fn data_state(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Uploading => "uploading",
            Self::Processing => "processing",
            Self::Error => "error",
            Self::Done => "done",
        }
    }

    fn border_class(self) -> &'static str {
        match self {
            Self::Error => "border-destructive/50",
            _ => "border-border",
        }
    }

    fn indicator(self) -> Element {
        match self {
            Self::Uploading | Self::Processing => rsx! {
                Spinner { class: "size-4 text-muted-foreground" }
            },
            Self::Error => rsx! {
                CircleAlert { class: "size-4 text-destructive", "aria-label": "Failed" }
            },
            Self::Done => rsx! {
                CircleCheck { class: "size-4 text-emerald-600 dark:text-emerald-500", "aria-label": "Complete" }
            },
            Self::Idle => rsx! {},
        }
    }
}

/// Props for [`Attachment`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentProps {
    /// The transfer/processing state, surfaced as `data-state` and a
    /// trailing status indicator.
    #[props(default)]
    pub state: AttachmentState,
    /// Corner radius of the card surface.
    #[props(default)]
    pub radius: Radius,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native div/global attributes.
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    pub attributes: Vec<Attribute>,
    /// Caller-composed [`AttachmentMedia`]/[`AttachmentContent`]/
    /// [`AttachmentActions`].
    pub children: Element,
}

/// A file-attachment card.
#[component]
pub fn Attachment(props: AttachmentProps) -> Element {
    let class = cn(&[
        "flex items-center gap-3 border bg-card p-3",
        props.state.border_class(),
        props.radius.class(),
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, "data-state": props.state.data_state(), ..props.attributes,
            {props.children}
            {props.state.indicator()}
        }
    }
}

/// Props for [`AttachmentGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentGroupProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed [`Attachment`] cards.
    pub children: Element,
}

/// A wrapping row of multiple attachment cards, e.g. in a message composer.
#[component]
pub fn AttachmentGroup(props: AttachmentGroupProps) -> Element {
    let class = cn(&[
        "flex flex-wrap gap-2",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`AttachmentTrigger`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentTriggerProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native click handler, e.g. to open a preview.
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// Caller-composed [`AttachmentMedia`]/[`AttachmentContent`].
    pub children: Element,
}

/// A clickable region (e.g. to open a preview) that IS its own real
/// `<button>`, not a wrapper around one -- see this module's own header
/// comment.
#[component]
pub fn AttachmentTrigger(props: AttachmentTriggerProps) -> Element {
    let class = cn(&[
        "flex min-w-0 flex-1 items-center gap-3 rounded-md text-left outline-none focus-visible:ring-2 focus-visible:ring-ring",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        button {
            r#type: "button",
            class,
            onclick: move |event| props.onclick.call(event),
            {props.children}
        }
    }
}

/// Props for [`AttachmentMedia`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentMediaProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed thumbnail/icon. Defaults to a paperclip icon when
    /// left empty.
    #[props(default)]
    pub children: Option<Element>,
}

/// The thumbnail/icon slot.
#[component]
pub fn AttachmentMedia(props: AttachmentMediaProps) -> Element {
    let class = cn(&[
        "flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted text-muted-foreground [&>img]:size-full [&>img]:object-cover",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class,
            {props.children.unwrap_or_else(|| rsx! { Paperclip { class: "size-4" } })}
        }
    }
}

/// Props for [`AttachmentContent`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentContentProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed [`AttachmentTitle`]/[`AttachmentDescription`].
    pub children: Element,
}

/// The title/description column.
#[component]
pub fn AttachmentContent(props: AttachmentContentProps) -> Element {
    let class = cn(&[
        "flex min-w-0 flex-1 flex-col",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`AttachmentTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentTitleProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed filename.
    pub children: Element,
}

/// The filename.
#[component]
pub fn AttachmentTitle(props: AttachmentTitleProps) -> Element {
    let class = cn(&[
        "truncate text-sm font-medium",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        p { class, {props.children} }
    }
}

/// Props for [`AttachmentDescription`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentDescriptionProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed secondary text -- file size, status text.
    pub children: Element,
}

/// The secondary text line.
#[component]
pub fn AttachmentDescription(props: AttachmentDescriptionProps) -> Element {
    let class = cn(&[
        "truncate text-xs text-muted-foreground",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        p { class, {props.children} }
    }
}

/// Props for [`AttachmentActions`].
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentActionsProps {
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Caller-composed [`AttachmentAction`]s.
    pub children: Element,
}

/// The trailing action-button row.
#[component]
pub fn AttachmentActions(props: AttachmentActionsProps) -> Element {
    let class = cn(&[
        "flex shrink-0 items-center gap-1",
        props.class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {props.children} }
    }
}

/// Props for [`AttachmentAction`]. Composes the sibling `Button` registry
/// item (upstream's `action` part composes `button.button.root`).
#[derive(Props, Clone, PartialEq)]
pub struct AttachmentActionProps {
    /// Accessible label, since this action is icon-only.
    pub aria_label: String,
    /// Extra classes appended to the semantic default.
    #[props(default)]
    pub class: Option<String>,
    /// Native click handler.
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// Caller-composed icon.
    pub children: Element,
}

/// A single icon-only action (remove, retry, download).
#[component]
pub fn AttachmentAction(props: AttachmentActionProps) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            class: props.class,
            "aria-label": props.aria_label,
            onclick: move |event| props.onclick.call(event),
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_state_uses_destructive_border() {
        assert_eq!(
            AttachmentState::Error.border_class(),
            "border-destructive/50"
        );
        assert_eq!(AttachmentState::Idle.border_class(), "border-border");
    }

    #[test]
    fn data_state_matches_every_variant() {
        assert_eq!(AttachmentState::Idle.data_state(), "idle");
        assert_eq!(AttachmentState::Uploading.data_state(), "uploading");
        assert_eq!(AttachmentState::Processing.data_state(), "processing");
        assert_eq!(AttachmentState::Error.data_state(), "error");
        assert_eq!(AttachmentState::Done.data_state(), "done");
    }
}
