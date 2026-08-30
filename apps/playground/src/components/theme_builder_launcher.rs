//! Playground-only wiring that opens the installed `ThemeBuilder` inside the
//! installed `Dialog`. Contains no theme logic of its own -- pure
//! composition of two already-installed registry components, per
//! `openspec/changes/2026-08-30-playground-uses-registry-theme-and-sidebar/design.md`
//! decision 3.

use dioxus::prelude::*;

use crate::components::ui;

#[component]
pub fn ThemeBuilderLauncher() -> Element {
    rsx! {
        ui::Dialog {
            ui::DialogTrigger { class: "w-full justify-start", "Customize theme" }
            ui::DialogOverlay {}
            ui::DialogContent { class: "max-h-[calc(100svh-2rem)] max-w-md overflow-y-auto p-5 sm:p-6",
                ui::DialogHeader {
                    ui::DialogTitle { "Theme builder" }
                    ui::DialogDescription { "Edit every semantic theme token live, generate a random theme, or copy the CSS export." }
                }
                div { class: "mt-4", ui::ThemeBuilder {} }
            }
        }
    }
}
