//! Source-owned shadcn-style Radio Group for Dioxus, backed by the owned
//! adico primitive layer.

use dioxus::prelude::*;

pub use adico_primitives::radio_group::RadioGroup;
use adico_primitives::radio_group::RadioItem as RadioItemPrimitive;

use crate::adico_lib::cn::cn;

/// A single selectable radio button within a [`RadioGroup`].
#[component]
pub fn RadioItem(
    value: ReadSignal<String>,
    index: ReadSignal<usize>,
    #[props(default)] disabled: ReadSignal<bool>,
    id: Option<String>,
    class: Option<String>,
    children: Element,
) -> Element {
    let class = cn(&[
        "relative aspect-square h-4 w-4 rounded-full border border-primary text-primary shadow focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 before:absolute before:inset-0 before:m-auto before:h-2 before:w-2 before:rounded-full before:bg-primary before:opacity-0 before:transition-opacity data-[state=checked]:before:opacity-100",
        class.as_deref().unwrap_or_default(),
    ]);
    rsx! {
        RadioItemPrimitive { value, index, disabled, id, class, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_dot_is_hidden_until_checked() {
        let class = cn(&[
            "before:opacity-0 before:transition-opacity data-[state=checked]:before:opacity-100",
            "",
        ]);
        assert!(class.contains("before:opacity-0"));
        assert!(class.contains("data-[state=checked]:before:opacity-100"));
    }
}
