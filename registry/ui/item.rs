//! Source-owned shadcn-style Item composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// A vertical list of [`Item`] rows.
#[component]
pub fn ItemGroup(children: Element, class: Option<String>) -> Element {
    let class = cn(&["flex flex-col", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, role: "list", {children} }
    }
}

/// A single row combining optional media, content, and actions.
#[component]
pub fn Item(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex items-center gap-4 rounded-md border p-4 text-sm outline-none transition-colors focus-visible:ring-1 focus-visible:ring-ring",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, role: "listitem", {children} }
    }
}

/// A leading icon, avatar, or image slot for an [`Item`].
#[component]
pub fn ItemMedia(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex shrink-0 items-center justify-center",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// The flexible text region of an [`Item`], holding title/description.
#[component]
pub fn ItemContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-1 flex-col gap-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// The primary label of an [`Item`].
#[component]
pub fn ItemTitle(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "text-sm font-medium leading-none",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// Supporting text placed under an [`ItemTitle`].
#[component]
pub fn ItemDescription(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "text-sm text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A trailing slot for buttons or other controls on an [`Item`].
#[component]
pub fn ItemActions(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex items-center gap-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A full-width row above an [`Item`]'s main content, e.g. for grouped headers.
#[component]
pub fn ItemHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex basis-full items-center justify-between gap-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A full-width row below an [`Item`]'s main content.
#[component]
pub fn ItemFooter(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex basis-full items-center justify-between gap-2",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A thin horizontal rule between items in an [`ItemGroup`].
#[component]
pub fn ItemSeparator(class: Option<String>) -> Element {
    let class = cn(&["my-0 border-t", class.as_deref().unwrap_or_default()]);
    rsx! {
        hr { class, role: "separator" }
    }
}
