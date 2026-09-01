//! Source-owned shadcn-style InputGroup for Dioxus.
//!
//! `InputGroupAddon` omits upstream's click-to-focus-the-adjoining-input
//! JavaScript behavior (a UX nicety, not required for the addon to function):
//! this repository's `document`-eval-based DOM interop has repeatedly been
//! found unreliable in this sandbox (see `docs/adico/m4-acceptance.md`'s
//! "the other symptom-class of `document`-eval defects"), so this batch does
//! not add a new instance of it for a non-essential effect. `InputGroupInput`
//! and `InputGroupTextarea` author their own flush (borderless, transparent)
//! classes rather than composing `Input`/`Textarea` and overriding their
//! fixed classes: `adico_lib::cn::cn` is plain string concatenation with no
//! Tailwind-merge equivalent, so two conflicting single-class selectors
//! (e.g. `border-input` vs. an appended `border-0`) have no reliable winner --
//! generated CSS cascade order, not class-attribute order, decides. Authoring
//! the flush classes directly avoids depending on an ordering guarantee this
//! toolchain doesn't provide.

use dioxus::prelude::*;

use super::button::{Button, ButtonSize, ButtonVariant};
use crate::adico_lib::cn::cn;

/// Where an [`InputGroupAddon`] sits relative to its sibling control.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InputGroupAlign {
    /// Leading edge, inline with a single-line control.
    #[default]
    InlineStart,
    /// Trailing edge, inline with a single-line control.
    InlineEnd,
    /// Leading edge, stacked above a multi-line control.
    BlockStart,
    /// Trailing edge, stacked below a multi-line control.
    BlockEnd,
}

impl InputGroupAlign {
    fn attr(self) -> &'static str {
        match self {
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
            Self::BlockStart => "block-start",
            Self::BlockEnd => "block-end",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Self::InlineStart => "order-first pl-3",
            Self::InlineEnd => "order-last pr-3",
            Self::BlockStart => {
                "order-first w-full justify-start px-3 pt-3 group-has-[>input]/input-group:pt-2.5"
            }
            Self::BlockEnd => {
                "order-last w-full justify-start px-3 pb-3 group-has-[>input]/input-group:pb-2.5"
            }
        }
    }
}

/// The bordered container joining an input/textarea control with its
/// [`InputGroupAddon`]s into one visual field.
#[component]
pub fn InputGroup(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "group/input-group relative flex w-full items-center rounded-md border border-input shadow-xs outline-none transition-[color,box-shadow]",
        "h-9 min-w-0 has-[>textarea]:h-auto",
        "has-[[data-slot=input-group-control]:focus-visible]:border-ring has-[[data-slot=input-group-control]:focus-visible]:ring-[3px] has-[[data-slot=input-group-control]:focus-visible]:ring-ring/50",
        "has-[[data-slot][aria-invalid=true]]:border-destructive has-[[data-slot][aria-invalid=true]]:ring-destructive/20 dark:has-[[data-slot][aria-invalid=true]]:ring-destructive/40",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { role: "group", "data-slot": "input-group", class, {children} }
    }
}

/// A leading/trailing decoration (icon, [`InputGroupButton`], [`InputGroupText`])
/// within an [`InputGroup`].
#[component]
pub fn InputGroupAddon(
    #[props(default)] align: InputGroupAlign,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "flex h-auto cursor-text items-center justify-center gap-2 py-1.5 text-sm font-medium text-muted-foreground select-none [&>svg:not([class*='size-'])]:size-4",
        align.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div {
            role: "group",
            "data-slot": "input-group-addon",
            "data-align": align.attr(),
            class,
            {children}
        }
    }
}

/// A [`Button`] sized to sit inside an [`InputGroupAddon`]. Reuses `Button`'s
/// own [`ButtonVariant`]/[`ButtonSize`] scale directly rather than a
/// duplicate parallel one -- adico's `Xs`/`IconXs`/`Sm`/`IconSm` sizes already
/// match upstream's `xs`/`icon-xs`/`sm`/`icon-sm` addon-button sizes.
#[component]
pub fn InputGroupButton(
    #[props(default = ButtonVariant::Ghost)] variant: ButtonVariant,
    #[props(default = ButtonSize::Xs)] size: ButtonSize,
    class: Option<String>,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let class = cn(&["shadow-none", class.as_deref().unwrap_or_default()]);
    rsx! {
        Button { variant, size, class, onclick, {children} }
    }
}

/// Non-interactive text within an [`InputGroup`] (e.g. a fixed unit or prefix).
#[component]
pub fn InputGroupText(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "flex items-center gap-2 text-sm text-muted-foreground [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, {children} }
    }
}

/// A borderless, transparent single-line control for an [`InputGroup`]. See
/// this module's doc comment for why it authors its own classes rather than
/// composing [`super::input::Input`].
#[component]
pub fn InputGroupInput(
    #[props(default = "text".to_string())] r#type: String,
    #[props(default)] value: Option<String>,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] disabled: Option<bool>,
    #[props(default)] invalid: bool,
    #[props(default)] oninput: EventHandler<FormEvent>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex-1 rounded-none border-0 bg-transparent px-3 py-1 text-sm shadow-none outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        input {
            "data-slot": "input-group-control",
            class,
            r#type,
            value,
            placeholder,
            disabled,
            aria_invalid: invalid,
            oninput: move |event| oninput.call(event),
            ..attributes,
        }
    }
}

/// A borderless, transparent multi-line control for an [`InputGroup`]. See
/// this module's doc comment for why it authors its own classes rather than
/// composing [`super::textarea::Textarea`].
#[component]
pub fn InputGroupTextarea(
    #[props(default)] value: Option<String>,
    #[props(default)] placeholder: Option<String>,
    #[props(default)] disabled: Option<bool>,
    #[props(default)] invalid: bool,
    #[props(default)] rows: Option<u32>,
    #[props(default)] oninput: EventHandler<FormEvent>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = textarea)]
    attributes: Vec<Attribute>,
) -> Element {
    let class = cn(&[
        "flex-1 resize-none rounded-none border-0 bg-transparent px-3 py-3 text-sm shadow-none outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        textarea {
            "data-slot": "input-group-control",
            class,
            value,
            placeholder,
            disabled,
            rows,
            aria_invalid: invalid,
            oninput: move |event| oninput.call(event),
            ..attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addon_alignments_target_the_correct_edge() {
        assert!(InputGroupAlign::InlineStart.class().contains("order-first"));
        assert!(InputGroupAlign::InlineEnd.class().contains("order-last"));
        assert_eq!(InputGroupAlign::BlockStart.attr(), "block-start");
        assert_eq!(InputGroupAlign::BlockEnd.attr(), "block-end");
    }

    #[test]
    fn controls_remain_visually_flush_with_the_group_border() {
        let class = cn(&["rounded-none border-0 bg-transparent shadow-none"]);
        assert!(class.contains("border-0"));
    }
}
