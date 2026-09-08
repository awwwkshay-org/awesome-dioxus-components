//! Black-box tests for `adico_primitives::scroll_area`, per this repo's test-placement
//! convention (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`):
//! every test lives under `packages/adico-primitives/tests/`, never inline in
//! `src/scroll_area.rs`.
//!
//! `ScrollArea` renders a `dx-scroll-area-root` wrapper containing the real scrolling
//! element (`ScrollAreaViewport`) plus overlay scrollbar siblings, so assertions here
//! check substrings of the whole render rather than counting attributes on a single
//! element -- checking that a merged class/style string appears as one *contiguous*
//! substring is a stronger proof of correct merging than counting `class=`/`style=`
//! occurrences across the whole tree (which now legitimately has more than one element).
//! `ScrollAreaScrollbar`/`ScrollAreaThumb`/`ScrollAreaCorner` render nothing during SSR
//! (no DOM to measure), so their absence from every SSR string below is expected, not a
//! gap in coverage -- their behavior is exercised live in `dx serve`, not via SSR string
//! assertions.

use adico_primitives::scroll_area::{
    CrossAxisOverflow, ScrollArea, ScrollAreaViewport, ScrollDirection, ScrollType,
    scroll_area_visibility_class, use_scroll_area_scroll_to,
};
use dioxus::prelude::*;

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[component]
fn AutoBoth() -> Element {
    rsx! { ScrollArea { "content" } }
}

#[test]
fn default_scroll_area_auto_overflows_both_axes_and_hides_the_native_scrollbar() {
    let html = render(AutoBoth);
    assert!(html.contains("overflow-x: auto"), "{html}");
    assert!(html.contains("overflow-y: auto"), "{html}");
    // `ScrollArea` (the wrapper) always hides the native scrollbar unconditionally --
    // the overlay thumb replaces it -- regardless of `scroll_type`. This is a
    // deliberate change from the standalone `ScrollAreaViewport`'s own
    // `scroll_type`-gated behavior (see `hide_native_scrollbar_defaults_to_false_on_a_standalone_viewport`
    // below), not an oversight.
    assert!(html.contains("scrollbar-width: none"), "{html}");
    assert!(html.contains(r#"data-scroll-direction="both""#), "{html}");
    assert!(html.contains("dx-scroll-area-root"), "{html}");
}

#[component]
fn AlwaysVertical() -> Element {
    rsx! {
        ScrollArea {
            direction: ScrollDirection::Vertical,
            scroll_type: ScrollType::Always,
            "content"
        }
    }
}

#[test]
fn always_vertical_scroll_area_hides_the_cross_axis_and_forces_the_scroll_axis() {
    let html = render(AlwaysVertical);
    assert!(html.contains("overflow-x: hidden"), "{html}");
    assert!(html.contains("overflow-y: scroll"), "{html}");
    assert!(
        html.contains(r#"data-scroll-direction="vertical""#),
        "{html}"
    );
}

#[component]
fn HiddenHorizontal() -> Element {
    rsx! {
        ScrollArea {
            direction: ScrollDirection::Horizontal,
            scroll_type: ScrollType::Hidden,
            "content"
        }
    }
}

#[test]
fn hidden_horizontal_scroll_area_keeps_scrolling_but_hides_the_scrollbar() {
    let html = render(HiddenHorizontal);
    assert!(html.contains("overflow-x: scroll"), "{html}");
    assert!(html.contains("overflow-y: hidden"), "{html}");
    // `scrollbar-width` must land inside the `style` declaration (a real CSS rule), not
    // as a standalone, inert HTML attribute of the same name -- that was the original
    // defect: it rendered as `scrollbar-width="none"` and had no visual effect.
    assert!(html.contains("scrollbar-width: none"), "{html}");
    assert!(!html.contains(r#"scrollbar-width="none""#), "{html}");
    assert!(
        html.contains(r#"data-scroll-direction="horizontal""#),
        "{html}"
    );
}

#[component]
fn HorizontalVisibleCrossAxis() -> Element {
    rsx! {
        ScrollArea {
            direction: ScrollDirection::Horizontal,
            cross_axis_overflow: CrossAxisOverflow::Visible,
            "content"
        }
    }
}

#[test]
fn visible_cross_axis_overflow_leaves_the_cross_axis_unclipped() {
    let html = render(HorizontalVisibleCrossAxis);
    assert!(html.contains("overflow-x: auto"), "{html}");
    assert!(html.contains("overflow-y: visible"), "{html}");
}

#[component]
fn VerticalDefaultCrossAxis() -> Element {
    rsx! {
        ScrollArea {
            direction: ScrollDirection::Vertical,
            "content"
        }
    }
}

#[test]
fn default_cross_axis_overflow_still_clips_matching_original_behavior() {
    let html = render(VerticalDefaultCrossAxis);
    assert!(html.contains("overflow-x: hidden"), "{html}");
    assert!(html.contains("overflow-y: auto"), "{html}");
}

#[component]
fn AlwaysShowScrollbars() -> Element {
    rsx! {
        ScrollArea {
            always_show_scrollbars: true,
            "content"
        }
    }
}

#[test]
fn always_show_scrollbars_uses_the_always_show_visibility_class() {
    let html = render(AlwaysShowScrollbars);
    assert!(html.contains("dx-scroll-area-always-show"), "{html}");
}

#[test]
fn auto_hide_is_the_default_visibility_class() {
    let html = render(AutoBoth);
    assert!(html.contains("dx-scroll-area-auto-hide"), "{html}");
}

#[component]
fn WithCallerClass() -> Element {
    rsx! {
        ScrollArea {
            class: "caller-class",
            "content"
        }
    }
}

#[test]
fn caller_supplied_class_is_merged_with_the_root_class_not_replacing_it() {
    let html = render(WithCallerClass);
    // `class`/`style` apply to the outer wrapper (Radix/shadcn's "Root"), not the
    // inner viewport -- sizing/layout classes are what a caller almost always means to
    // set on a `ScrollArea` (confirmed live: the original 3 call sites all pass
    // height/width/border). Both classes appear together, contiguously, in one
    // `class="..."` attribute on the wrapper -- the regression check for the SSR/CSR
    // class-collision defect: passing a class through the generic attribute spread
    // used to silently drop one class or the other depending on rendering mode.
    assert!(
        html.contains(r#"class="dx-scroll-area-root caller-class""#),
        "{html}"
    );
    // The viewport's own class is untouched by the caller's class -- it always carries
    // just the plain visibility class.
    assert!(
        html.contains(r#"class="dx-scroll-area-auto-hide""#),
        "{html}"
    );
}

#[component]
fn WithCallerStyle() -> Element {
    rsx! {
        ScrollArea {
            style: "width: 14em; height: 8em;",
            "content"
        }
    }
}

#[test]
fn caller_supplied_style_is_merged_with_the_root_style_not_replacing_it() {
    let html = render(WithCallerStyle);
    // The wrapper's own `style=` attribute must contain both its base positioning
    // rules and the caller's extra declarations together, not as two separate
    // `style=` attributes on the same element (the SSR collision this component's
    // `style` prop exists to prevent).
    assert!(
        html.contains("style=\"position: relative; overflow: hidden; width: 14em; height: 8em;\""),
        "{html}"
    );
    // The viewport fills whatever size the wrapper (sized by the caller above) ends
    // up being -- it never receives the caller's own height/width directly.
    assert!(html.contains("overflow-x: auto"), "{html}");
    assert!(
        html.contains(
            "style=\"overflow-x: auto; overflow-y: auto; scrollbar-width: none; height: 100%; width: 100%;\""
        ),
        "{html}"
    );
}

#[test]
fn scroll_area_visibility_class_helper_matches_the_component_output() {
    assert_eq!(
        scroll_area_visibility_class(false),
        "dx-scroll-area-auto-hide"
    );
    assert_eq!(
        scroll_area_visibility_class(true),
        "dx-scroll-area-always-show"
    );
}

// -- `ScrollAreaViewport` standalone (in-place adoption shape) --

#[component]
fn StandaloneViewportDefaultHidesNative() -> Element {
    adico_primitives::scroll_area::provide_scroll_area_context();
    rsx! {
        ScrollAreaViewport { "content" }
    }
}

#[test]
fn standalone_viewport_also_hides_native_scrollbar_by_default() {
    let html = render(StandaloneViewportDefaultHidesNative);
    assert!(html.contains("scrollbar-width: none"), "{html}");
}

#[component]
fn StandaloneViewportOptOut() -> Element {
    adico_primitives::scroll_area::provide_scroll_area_context();
    rsx! {
        ScrollAreaViewport { hide_native_scrollbar: false, "content" }
    }
}

#[test]
fn standalone_viewport_can_opt_back_into_scroll_type_only_native_scrollbar_semantics() {
    let html = render(StandaloneViewportOptOut);
    // With `hide_native_scrollbar: false`, the original `ScrollType`-gated behavior
    // applies: the default `ScrollType::Auto` never hides the native scrollbar.
    assert!(!html.contains("scrollbar-width"), "{html}");
}

#[component]
fn StandaloneViewportOptOutHidden() -> Element {
    adico_primitives::scroll_area::provide_scroll_area_context();
    rsx! {
        ScrollAreaViewport {
            hide_native_scrollbar: false,
            scroll_type: ScrollType::Hidden,
            "content"
        }
    }
}

#[test]
fn standalone_viewport_opt_out_still_respects_scroll_type_hidden() {
    let html = render(StandaloneViewportOptOutHidden);
    assert!(html.contains("scrollbar-width: none"), "{html}");
}

// -- Overlay parts render nothing during SSR (no DOM to measure yet) --

#[component]
fn ScrollAreaWithScrollbarChild() -> Element {
    rsx! { ScrollArea { "content" } }
}

#[test]
fn overlay_scrollbar_and_corner_render_nothing_during_ssr() {
    let html = render(ScrollAreaWithScrollbarChild);
    // No guessed-size thumb ever flashes on hydration: before the first client-side
    // measurement (`measured` starts `false` and only a live `onmounted` sets it),
    // `ScrollAreaScrollbar`/`ScrollAreaThumb`/`ScrollAreaCorner` render nothing at all.
    assert!(!html.contains("dx-scroll-area-scrollbar"), "{html}");
    assert!(!html.contains("dx-scroll-area-thumb"), "{html}");
    assert!(!html.contains("dx-scroll-area-corner"), "{html}");
}

// -- Programmatic scroll escape hatch (task 2.4) --

#[component]
fn ScrollAreaScrollToButton() -> Element {
    // `use_scroll_area_scroll_to` is a hook -- called unconditionally during render,
    // never inside the `onclick` closure below.
    let scroll_to = use_scroll_area_scroll_to();
    rsx! {
        button {
            onclick: move |_| scroll_to.call((0.0, 0.0)),
            "Scroll to top"
        }
    }
}

#[component]
fn ScrollAreaWithProgrammaticScrollButton() -> Element {
    rsx! {
        ScrollArea {
            ScrollAreaScrollToButton {}
            "content"
        }
    }
}

#[test]
fn use_scroll_area_scroll_to_is_reachable_from_a_scroll_area_descendant_with_no_panic() {
    // `use_scroll_area_scroll_to` must be callable from any descendant of `ScrollArea`
    // (which calls `provide_scroll_area_context` internally) without panicking -- the
    // panic-on-missing-context contract only fires when no ancestor provided one. A
    // real scroll (async, requires a live viewport handle) is exercised via `dx serve`,
    // not SSR; this proves the hook wiring itself is sound.
    let html = render(ScrollAreaWithProgrammaticScrollButton);
    assert!(html.contains("Scroll to top"), "{html}");
}
