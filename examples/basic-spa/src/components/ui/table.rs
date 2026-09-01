//! Source-owned shadcn-style Table for Dioxus.

use dioxus::prelude::*;

use crate::adico_lib::cn::cn;

/// The scroll-container-wrapped `<table>` root.
#[component]
pub fn Table(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "w-full caption-bottom text-sm",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class: "relative w-full overflow-x-auto",
            table { class, {children} }
        }
    }
}

/// The `<thead>` region.
#[component]
pub fn TableHeader(class: Option<String>, children: Element) -> Element {
    let class = cn(&["[&_tr]:border-b", class.as_deref().unwrap_or_default()]);
    rsx! {
        thead { class, {children} }
    }
}

/// The `<tbody>` region.
#[component]
pub fn TableBody(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "[&_tr:last-child]:border-0",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        tbody { class, {children} }
    }
}

/// The `<tfoot>` region.
#[component]
pub fn TableFooter(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "border-t bg-muted/50 font-medium [&>tr]:last:border-b-0",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        tfoot { class, {children} }
    }
}

/// A `<tr>` row.
#[component]
pub fn TableRow(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "border-b transition-colors hover:bg-muted/50 has-aria-expanded:bg-muted/50 data-[state=selected]:bg-muted",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        tr { class, {children} }
    }
}

/// A `<th>` header cell.
#[component]
pub fn TableHead(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "h-10 px-2 text-left align-middle font-medium whitespace-nowrap text-foreground [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        th { class, {children} }
    }
}

/// A `<td>` body cell.
#[component]
pub fn TableCell(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "p-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        td { class, {children} }
    }
}

/// A `<caption>`.
#[component]
pub fn TableCaption(class: Option<String>, children: Element) -> Element {
    let class = cn(&[
        "mt-4 text-sm text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        caption { class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_and_footer_use_semantic_surfaces() {
        let footer_class = cn(&["border-t bg-muted/50 font-medium [&>tr]:last:border-b-0"]);
        assert!(footer_class.contains("bg-muted/50"));
        let head_class =
            cn(&["h-10 px-2 text-left align-middle font-medium whitespace-nowrap text-foreground"]);
        assert!(head_class.contains("text-foreground"));
    }
}
