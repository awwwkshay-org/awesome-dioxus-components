//! `/docs/spacing`.

use dioxus::prelude::*;

use super::{GuidePage, GuideSection};
use crate::adico_lib::variants::Radius;
use crate::components::code_block::CodeBlock;
use crate::components::prose::Prose;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle, AlertVariant};

const STEPS: &[(&str, &str)] = &[
    ("1", "0.25rem"),
    ("2", "0.5rem"),
    ("3", "0.75rem"),
    ("4", "1rem"),
    ("6", "1.5rem"),
    ("8", "2rem"),
    ("12", "3rem"),
    ("16", "4rem"),
];

/// Each arm's Rust name beside the Tailwind class it actually emits. The
/// pairing is the point of the table — they deliberately do not match.
const RADIUS_STEPS: &[(&str, Radius, &str)] = &[
    ("Radius::None", Radius::None, "rounded-none"),
    ("Radius::Sm", Radius::Sm, "rounded-sm"),
    ("Radius::Md", Radius::Md, "rounded-md"),
    ("Radius::Default", Radius::Default, "rounded-lg"),
    ("Radius::Lg", Radius::Lg, "rounded-xl"),
    ("Radius::Xl", Radius::Xl, "rounded-2xl"),
    ("Radius::Full", Radius::Full, "rounded-full"),
];

#[component]
pub fn SpacingGuide() -> Element {
    rsx! {
        GuidePage {
            title: "Spacing & radius".to_string(),
            lead: "Spacing uses Tailwind's own 0.25rem scale unchanged. Radius does not — it is a shared Rust enum, because a component's corners have to be settable from a prop, not just a class.".to_string(),

            GuideSection { title: "Spacing scale".to_string(),
                Prose { text: "Stock Tailwind: each step is `0.25rem × n`. Components lean on the low end — `gap-2` and `gap-3` inside controls, `p-6` for card padding.".to_string() }
                div { class: "flex flex-col gap-2",
                    for (step , size) in STEPS {
                        div { key: "{step}", class: "flex items-center gap-4",
                            code { class: "w-12 shrink-0 font-mono text-xs text-muted-foreground", "{step}" }
                            div { class: "h-4 rounded-sm bg-primary", style: "width: {size}" }
                            span { class: "font-mono text-xs text-muted-foreground", "{size}" }
                        }
                    }
                }
            }

            GuideSection { title: "Radius".to_string(),
                Prose { text: "Components take `radius: Radius` rather than a class, so a caller can restyle corners without fighting class ordering. Every arm except `None` and `Full` derives from the single `--radius` custom property, so changing that one value rescales them all.".to_string() }
                Alert { variant: AlertVariant::Info,
                    AlertTitle { "The enum names the visual size, not the Tailwind suffix" }
                    AlertDescription {
                        Prose { text: "`Radius::Lg` emits `rounded-xl`, not `rounded-lg`. The names describe how large the corner *looks* in this system; they are deliberately offset by one from Tailwind's own suffixes. Read the table rather than assuming the mapping.".to_string() }
                    }
                }
                div { class: "overflow-x-auto rounded-lg border border-border",
                    table { class: "w-full min-w-[26rem] border-collapse text-sm",
                        thead {
                            tr { class: "border-b border-border bg-muted/40 text-left",
                                th { class: "p-3 font-medium", "Rust" }
                                th { class: "p-3 font-medium", "Class" }
                                th { class: "p-3 font-medium", "Looks like" }
                            }
                        }
                        tbody {
                            for (name , radius , class) in RADIUS_STEPS {
                                tr { key: "{name}", class: "border-b border-border/60 last:border-0",
                                    td { class: "p-3 font-mono text-xs", "{name}" }
                                    td { class: "p-3 font-mono text-xs text-muted-foreground", "{class}" }
                                    td { class: "p-3",
                                        div { class: "size-10 border border-border bg-muted {radius.class()}" }
                                    }
                                }
                            }
                        }
                    }
                }
                CodeBlock { code: "Card { radius: Radius::Xl, /* ... */ }\nButton { radius: Radius::Full, \"Pill\" }".to_string() }
            }

            GuideSection { title: "Breakpoints".to_string(),
                Prose { text: "Registry components are audited to use only `sm:` (640px) and, where genuinely needed, `md:` (768px). Fewer breakpoints keeps component diffs reviewable and behaviour predictable — application pages are free to use more.".to_string() }
            }
        }
    }
}
