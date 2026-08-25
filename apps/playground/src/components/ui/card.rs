//! Source-owned shadcn-style Card composition for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// The outer container for a Card composition.
#[component]
pub fn Card(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "rounded-xl border bg-card text-card-foreground shadow",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// A semantic header region for a Card's title and description.
#[component]
pub fn CardHeader(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex flex-col space-y-1.5 p-6",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}

/// The primary heading of a Card.
#[component]
pub fn CardTitle(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "font-semibold leading-none tracking-tight",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        h3 { class, {children} }
    }
}

/// Supporting text placed under a [`CardTitle`].
#[component]
pub fn CardDescription(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "text-sm text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        p { class, {children} }
    }
}

/// The main body region of a Card.
#[component]
pub fn CardContent(children: Element, class: Option<String>) -> Element {
    let class = cn(&["p-6 pt-0", class.as_deref().unwrap_or_default()]);
    rsx! {
        div { class, {children} }
    }
}

/// A footer region typically used for Card actions.
#[component]
pub fn CardFooter(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "flex items-center p-6 pt-0",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class, {children} }
    }
}
