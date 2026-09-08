use dioxus::prelude::*;

use crate::components::ui;
use crate::components::ui::button::{ButtonSize, ButtonVariant};

/// Where a preview pan began: the pointer's position and the offset the
/// component already had, so the pan continues from wherever it was.
#[derive(Clone, Copy, PartialEq)]
struct PanStart {
    pointer: (f64, f64),
    offset: (f64, f64),
}

/// A fixed preview workspace: its title stays at the top, the live component
/// occupies the upper three quarters, and its prop controls occupy the lower
/// quarter. The controls pane alone scrolls when it contains more content.
///
/// The preview canvas is pannable: dragging its grid background moves the
/// rendered component (a `position: relative; left; top;` offset on the
/// centered wrapper — deliberately NOT `transform`/`translate`, which would
/// establish a new containing block for every `position: fixed` descendant
/// per the CSS Transforms spec; `adico_primitives::positioner::Positioner`
/// renders every anchored popup as `position: fixed` with viewport-space
/// coordinates, so a `transform` ancestor here would silently break the
/// placement of every Tooltip/Popover/HoverCard/DropdownMenu/Select/
/// Combobox/etc. demoed inside it — found live via a screenshot showing
/// ModeToggle's dropdown anchored to the bottom-right corner instead of its
/// trigger), and the Center button in the title row — the installed
/// `Button` — resets it. A drag beginning on the demoed component (or anywhere inside its wrapper)
/// never starts a pan: the wrapper stops `pointerdown` propagation, so
/// component interactions like slider thumbs or the carousel track keep
/// full pointer behavior. One named limitation: content a component portals
/// *outside* its wrapper (e.g. a popover surface) doesn't get that guard,
/// so a drag starting there pans. Panning replaces the canvas's old
/// `overflow-auto` scrolling; the canvas itself stays `overflow-visible` (a
/// non-transformed ancestor's `overflow: hidden`/`auto` still clips a
/// `position: fixed` popover in some browsers even though the popover's own
/// placement is viewport-relative — found live via a screenshot of
/// DatePicker's calendar getting clipped at the canvas edge) so an anchored
/// popup taller or wider than the small preview box can still render in
/// full; a component panned far off-center can visually spill past the
/// canvas border as a result.
///
/// `wide` opts a demoed component out of the default intrinsic-width
/// centering: with `justify-content: center`, a block-layout component
/// (e.g. Accordion) shrinks to its content width before any internal
/// `justify-between` styling has room to act, silently defeating it. Pages
/// for such components should pass `wide: true` so the component instead
/// stretches to fill the preview's width budget.
#[component]
pub fn Demo(
    name: &'static str,
    #[props(default)] wide: bool,
    controls: Option<Element>,
    children: Element,
) -> Element {
    let mut offset = use_signal(|| (0.0_f64, 0.0_f64));
    let mut pan: Signal<Option<PanStart>> = use_signal(|| None);

    let (x, y) = offset();
    let centered = x == 0.0 && y == 0.0;
    let wrapper_layout = if wide {
        "width: min(100%, 36rem); height: 100%; align-self: center; justify-self: center;"
    } else {
        "display: flex; width: min(100%, 36rem); height: 100%; align-items: center; justify-content: center; align-self: center; justify-self: center;"
    };
    let wrapper_style = format!("{wrapper_layout} position: relative; left: {x}px; top: {y}px;");

    rsx! {
        section { class: "flex h-full min-h-0 w-full flex-col text-foreground",
            div { class: "flex shrink-0 items-center justify-between gap-2",
                h1 { class: "text-2xl font-bold", "{name}" }
                // Anchored in the title row, not inside (or right below) the
                // canvas: the canvas is `overflow-visible` so an anchored
                // popup taller than the preview can render in full (see the
                // module doc), which means demoed content taller than the
                // canvas box also spills past its bottom edge -- ANY sibling
                // positioned by the canvas's own flow (even a plain in-flow
                // row right after it) still lands inside that spilled-over
                // content's occupied space once the overflow is tall enough,
                // burying the button behind it (found live via screenshots of
                // ThemeBuilder's tall control stack, first with the button
                // absolutely pinned to the canvas edge, then again after
                // moving it to an in-flow row directly below the canvas). The
                // title row's height is fixed by its own content, entirely
                // independent of the canvas's, so it can never collide.
                ui::Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    disabled: centered,
                    onclick: move |_| offset.set((0.0, 0.0)),
                    "Center"
                }
            }
            ui::ResizablePanelGroup {
                direction: ui::ResizableDirection::Vertical,
                class: "my-3 min-h-0 flex-1 gap-3",
                ui::ResizablePanel {
                    index: 0usize,
                    default_size: 70.0,
                    min_size: 60.0,
                    max_size: 80.0,
                    class: "flex min-h-0 flex-col",
                    div {
                        class: "relative z-20 grid min-h-0 flex-1 cursor-grab place-items-center overflow-visible rounded-lg border border-border bg-muted/20 p-6",
                        style: "background-image: linear-gradient(hsl(var(--border) / 0.08) 1px, transparent 1px), linear-gradient(90deg, hsl(var(--border) / 0.08) 1px, transparent 1px); background-size: 2rem 2rem;",
                        onpointerdown: move |event: Event<PointerData>| {
                            let point = event.client_coordinates();
                            pan.set(
                                Some(PanStart {
                                    pointer: (point.x, point.y),
                                    offset: *offset.peek(),
                                }),
                            );
                        },
                        div {
                            class: "text-card-foreground",
                            style: wrapper_style,
                            // Pan starts only from the canvas background: a drag
                            // beginning on the demoed component must reach the
                            // component, not move it around.
                            onpointerdown: move |event: Event<PointerData>| {
                                event.stop_propagation();
                            },
                            {children}
                        }
                        if let Some(start) = *pan.read() {
                            div {
                                class: "fixed inset-0 z-[100] cursor-grabbing select-none",
                                onpointermove: move |event: Event<PointerData>| {
                                    let point = event.client_coordinates();
                                    offset
                                        .set((
                                            start.offset.0 + (point.x - start.pointer.0),
                                            start.offset.1 + (point.y - start.pointer.1),
                                        ));
                                },
                                onpointerup: move |_| pan.set(None),
                                onpointercancel: move |_| pan.set(None),
                            }
                        }
                    }
                }
                ui::ResizableHandle { handle_index: 0usize, with_handle: true }
                ui::ResizablePanel {
                    index: 1usize,
                    default_size: 30.0,
                    min_size: 20.0,
                    max_size: 40.0,
                    class: "flex min-h-0 flex-col",
                    ui::Card { class: "z-10 flex min-h-0 flex-1 flex-col border-border p-0 shadow-none",
                        ui::CardHeader { class: "shrink-0 gap-0 border-b border-border px-4 py-2",
                            ui::CardTitle { class: "text-sm", "Component controls" }
                        }
                        ui::CardContent { class: "min-h-0 flex-1 overflow-y-auto p-4",
                            if let Some(controls) = controls {
                                // `CardContent`'s own base class is `p-6 pt-0`
                                // (`ui/card.rs`) -- `cn()` is a plain space-joiner,
                                // not a tailwind-merge-style conflict resolver, so
                                // this `CardContent`'s `p-4` override above does
                                // NOT reliably win the cascade against the base
                                // class on any axis (which utility wins between
                                // two same-specificity classes depends on
                                // Tailwind's internal generation order in
                                // tailwind.css, not source/call order); `pt-0`
                                // reliably wins, leaving zero visible gap above
                                // the first control group. `mt-4` sidesteps the
                                // conflict entirely by using a property
                                // (margin) the base class never touches.
                                div { class: "mt-4 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3", {controls} }
                            } else {
                                p { class: "mt-4 text-sm text-muted-foreground", "This component has no live props in the playground yet." }
                            }
                        }
                    }
                }
            }
        }
    }
}
