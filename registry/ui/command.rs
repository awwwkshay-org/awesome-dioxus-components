//! Source-owned shadcn-style Command palette composition for Dioxus, backed
//! by the owned adico primitive layer.

use dioxus::prelude::*;

use adico_primitives::command::{
    CommandEmpty as CommandPrimitiveEmpty, CommandGroup as CommandPrimitiveGroup,
    CommandInput as CommandPrimitiveInput, CommandItem as CommandPrimitiveItem,
    CommandList as CommandPrimitiveList, CommandRoot as CommandPrimitiveRoot,
    CommandSeparator as CommandPrimitiveSeparator,
};

use super::dialog::{Dialog, DialogContent, DialogDescription, DialogTitle};
use crate::adico_lib::cn::cn;
use crate::adico_lib::variants::Radius;

/// The root of a command palette: an always-visible, query-filtered,
/// roving-focus list of actions.
#[component]
pub fn Command(
    children: Element,
    #[props(default = Radius::Md)] radius: Radius,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "flex h-full w-full flex-col overflow-hidden bg-popover text-popover-foreground",
        radius.class(),
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        CommandPrimitiveRoot { class, {children} }
    }
}

/// The search input that filters the palette's [`CommandList`].
#[component]
pub fn CommandInput(placeholder: Option<String>, class: Option<String>) -> Element {
    let class = cn(&[
        "flex h-9 w-full rounded-md bg-transparent px-3 py-1 text-sm outline-none placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        div { class: "flex items-center border-b px-3",
            CommandPrimitiveInput { class, placeholder: placeholder.unwrap_or_default() }
        }
    }
}

/// The always-visible, scrollable container for a [`Command`]'s items.
#[component]
pub fn CommandList(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "max-h-[300px] overflow-y-auto overflow-x-hidden p-1",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        CommandPrimitiveList { class, {children} }
    }
}

/// Shown only while no [`CommandItem`] matches the current query.
#[component]
pub fn CommandEmpty(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "py-6 text-center text-sm text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        CommandPrimitiveEmpty { class, {children} }
    }
}

/// A visual/ARIA grouping of related [`CommandItem`]s, with an optional heading.
#[component]
pub fn CommandGroup(children: Element, heading: Option<String>, class: Option<String>) -> Element {
    let class = cn(&[
        "overflow-hidden p-1 text-foreground [&_[data-heading]]:px-2 [&_[data-heading]]:py-1.5 [&_[data-heading]]:text-xs [&_[data-heading]]:font-medium [&_[data-heading]]:text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        CommandPrimitiveGroup { class,
            if let Some(heading) = heading {
                div { "data-heading": "true", {heading} }
            }
            {children}
        }
    }
}

/// A visual divider between [`CommandGroup`]s.
#[component]
pub fn CommandSeparator(class: Option<String>) -> Element {
    let class = cn(&["-mx-1 h-px bg-border", class.as_deref().unwrap_or_default()]);
    rsx! {
        CommandPrimitiveSeparator { class }
    }
}

/// A single filterable action inside a [`CommandList`].
#[component]
pub fn CommandItem<T: Clone + PartialEq + 'static>(
    index: ReadSignal<usize>,
    value: ReadSignal<T>,
    #[props(default)] text_value: ReadSignal<Option<String>>,
    #[props(default)] disabled: ReadSignal<bool>,
    #[props(default)] on_select: Callback<T>,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&[
        "relative flex cursor-default select-none items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-none data-[disabled=true]:pointer-events-none data-[highlighted=true]:bg-accent data-[highlighted=true]:text-accent-foreground data-[disabled=true]:opacity-50",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        CommandPrimitiveItem { index, value, text_value, disabled, on_select, class, {children} }
    }
}

/// A trailing keyboard-shortcut hint (e.g. `⌘K`) for a [`CommandItem`]. Purely
/// presentational -- no primitive backs this, matching upstream shadcn's own
/// `CommandShortcut`, which is just a styled `<span>`.
#[component]
pub fn CommandShortcut(children: Element, class: Option<String>) -> Element {
    let class = cn(&[
        "ml-auto text-xs tracking-widest text-muted-foreground",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        span { class, {children} }
    }
}

/// A [`Command`] palette presented inside a [`Dialog`] -- the common pattern
/// for a global "⌘K" launcher. Composes the `dialog` registry item directly
/// rather than reimplementing focus-trapping/Escape/outside-dismissal, which
/// `Dialog` already provides.
#[component]
pub fn CommandDialog(
    open: ReadSignal<Option<bool>>,
    #[props(default)] on_open_change: Callback<bool>,
    /// Visually-hidden accessible title, announced by screen readers even
    /// though the palette itself has no visible dialog chrome. Defaults to
    /// "Command Palette", matching shadcn's own default.
    #[props(default = "Command Palette".to_string())]
    title: String,
    /// Visually-hidden accessible description, announced alongside `title`.
    #[props(default = "Search for a command to run...".to_string())]
    description: String,
    #[props(default = false)] show_close_button: bool,
    children: Element,
    class: Option<String>,
) -> Element {
    let class = cn(&["overflow-hidden p-0", class.as_deref().unwrap_or_default()]);
    rsx! {
        Dialog { open, on_open_change,
            DialogContent { class, show_close_button,
                span { class: "sr-only",
                    DialogTitle { "{title}" }
                    DialogDescription { "{description}" }
                }
                Command { {children} }
            }
        }
    }
}
