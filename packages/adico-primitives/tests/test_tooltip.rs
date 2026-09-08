//! Black-box tests for `adico_primitives::tooltip`, per this repo's test-placement convention
//! (see `openspec/changes/reauthor-primitives-from-independent-spec/design.md`): every test
//! lives under `packages/adico-primitives/tests/`, never inline in `src/tooltip.rs`.

use adico_primitives::dialog::{DialogContent, DialogRoot};
use adico_primitives::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;
use dioxus_core::{Event, Mutation};
use dioxus_html::{
    Code, EventData, Key, Location, Modifiers, SerializedHtmlEventConverter,
    SerializedKeyboardData, set_event_converter,
};

fn render(root: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(root);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Dispatches a real Escape keydown at the `nth` (0-indexed) `onkeydown`-listening
/// element in `dom`, in mount/edit order, and returns the freshly re-rendered HTML.
fn press_escape_on_nth_keydown_listener(dom: &mut VirtualDom, nth: usize) -> String {
    let edits = dom.rebuild_to_vec();
    let id = edits
        .edits
        .iter()
        .filter_map(|edit| match edit {
            Mutation::NewEventListener { name, id } if name == "keydown" => Some(*id),
            _ => None,
        })
        .nth(nth)
        .unwrap_or_else(|| panic!("no keydown listener at index {nth}"));
    set_event_converter(Box::new(SerializedHtmlEventConverter));
    let event = Event::new(
        EventData::Keyboard(SerializedKeyboardData::new(
            Key::Escape,
            Code::Escape,
            Location::Standard,
            false,
            Modifiers::empty(),
            false,
        ))
        .into_any(),
        true,
    );
    dom.runtime().handle_event("keydown", event, id);
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
}

#[component]
fn OpenTooltip() -> Element {
    rsx! {
        Tooltip { default_open: true,
            TooltipTrigger { "Hover me" }
            TooltipContent { "Tooltip text" }
        }
    }
}

#[test]
fn an_open_tooltip_reports_open_state_and_a_describedby_trigger() {
    let html = render(OpenTooltip);
    assert!(html.contains(r#"data-state="open""#), "{html}");
    assert!(html.contains("Hover me"), "{html}");
    // `TooltipContent` publishes its real id into the trigger's `aria-describedby` from a
    // `use_effect`, which a bare `rebuild_in_place()` does not drive to completion -- the same
    // effect-driven-state limitation this change has documented elsewhere (e.g.
    // test_select.rs, test_accordion.rs) -- so only presence, not the exact id match, is
    // asserted here.
    assert!(html.contains("aria-describedby"), "{html}");
}

#[component]
fn TwoTooltipsInsideAnOpenDialog() -> Element {
    // `adico_primitives::layer`'s shared stack is provided by whichever
    // `use_layer` caller registers *first among components that share an
    // ancestor path* -- confirmed empirically while writing this test
    // (two sibling components with no stack-providing common ancestor each
    // independently `provide_context` their own private `LayerStack`; see
    // `openspec/changes/deduplicate-primitives` design.md's D12 for the full
    // repro and why fixing that is out of this change's scope). So a real,
    // shared-stack regression test needs a genuine ancestor: `DialogRoot`
    // registers first (providing the stack), then `TooltipA` and `TooltipB`
    // -- true descendants of `DialogRoot`, not siblings of it -- join that
    // *same* stack in render order, with `TooltipB` registering last and
    // therefore topmost.
    rsx! {
        DialogRoot { default_open: true,
            DialogContent {
                Tooltip { default_open: true,
                    TooltipTrigger { "A" }
                    TooltipContent { "A content" }
                }
                Tooltip { default_open: true,
                    TooltipTrigger { "B" }
                    TooltipContent { "B content" }
                }
            }
        }
    }
}

/// Finds `marker` (a tooltip's own content text, unique in this fixture) and
/// returns the value of the nearest preceding `data-state="..."` attribute --
/// belonging either to that tooltip's own root `div` or its `TooltipContent`
/// (both reflect the same `open` state, so either is a correct signal).
fn nearest_preceding_data_state<'a>(html: &'a str, marker: &str) -> &'a str {
    let marker_pos = html.find(marker).expect("marker text renders");
    let prefix = &html[..marker_pos];
    let attr_start = prefix
        .rfind(r#"data-state=""#)
        .expect("a data-state before marker")
        + r#"data-state=""#.len();
    let value_end = attr_start + prefix[attr_start..].find('"').unwrap();
    &prefix[attr_start..value_end]
}

/// Regression test for task 5.1 (`openspec/changes/deduplicate-primitives`):
/// before this change, `TooltipTrigger`'s hand-rolled Escape handler had no
/// dismissal-layer check at all, so it closed the tooltip on *any* Escape
/// press -- including one dispatched at a tooltip that is not the topmost
/// overlay on its own shared layer stack. Now routed through the shared
/// `use_escape_key`, tooltip A's own handler must recognize it is not
/// topmost (tooltip B, mounted after it, is) and leave A open.
///
/// Listener index 1 is tooltip A's trigger: index 0 is `DialogRoot`'s own
/// `onkeydown` (registered first, since it is the outermost element); index
/// 2 is not tooltip B's trigger but tooltip A's own `TooltipContent` (its
/// `Positioner` always attaches an `onkeydown` that forwards to
/// `on_keydown`, a no-op here since `TooltipContent` doesn't set it);
/// tooltip B's trigger is index 3.
#[test]
fn an_escape_at_a_non_topmost_tooltip_does_not_close_it() {
    let mut dom = VirtualDom::new(TwoTooltipsInsideAnOpenDialog);
    let html = press_escape_on_nth_keydown_listener(&mut dom, 1);
    // Asserted before the state check so a vacuous pass (content never
    // mounted at all) can't hide as a false positive.
    assert!(html.contains("A content"), "{html}");
    assert_eq!(
        nearest_preceding_data_state(&html, "A content"),
        "open",
        "tooltip A (not topmost) incorrectly closed on Escape: {html}"
    );
}

/// The inverse of the test above, guarding against the fix breaking normal
/// dismissal: an Escape at the *topmost* tooltip (B, mounted last) still
/// closes it.
///
/// The listener index for B's trigger is feature-dependent: on the
/// SSR-fallback path (no `web`/`native`), `use_animated_open` mounts A's
/// content synchronously at the initial `rebuild_to_vec()` (the same call
/// `press_escape_on_nth_keydown_listener` uses to find the listener, before
/// any effect flush), so A's `Positioner`-forwarded no-op keydown occupies
/// index 2 and B's trigger is index 3. On the real `web`/`native` path,
/// content-mounting is effect-driven and hasn't happened by that same
/// pre-dispatch `rebuild_to_vec()`, so A's content listener doesn't exist yet
/// and B's trigger is index 2 instead.
#[cfg(not(any(feature = "web", feature = "native")))]
const TOPMOST_TOOLTIP_B_TRIGGER_INDEX: usize = 3;
#[cfg(any(feature = "web", feature = "native"))]
const TOPMOST_TOOLTIP_B_TRIGGER_INDEX: usize = 2;

#[test]
fn an_escape_at_the_topmost_tooltip_still_closes_it() {
    let mut dom = VirtualDom::new(TwoTooltipsInsideAnOpenDialog);
    let html = press_escape_on_nth_keydown_listener(&mut dom, TOPMOST_TOOLTIP_B_TRIGGER_INDEX);
    // Anchored on the trigger text (`>A<`), not `"A content"`: `Tooltip`'s own
    // root `data-state` is driven directly by its `open` signal with no
    // `use_animated_open` effect in the way (see `tooltip.rs:112`), so it is
    // accurate immediately, unlike content-mounting -- which, on the real
    // `web`/`native` path, only flushes for the scope that actually received
    // the dispatched event (here, B's), not sibling tooltip A's.
    assert_eq!(
        nearest_preceding_data_state(&html, ">A<"),
        "open",
        "tooltip A (unrelated to this Escape) should stay open: {html}"
    );
    // Unlike tooltip A above, B's own content unmounts entirely once closed
    // (`TooltipContent` has no `force_mount` escape hatch, unlike
    // `HoverCard`/`PreviewCard`'s), so this anchors on B's trigger text
    // (`>B<`, still always rendered) rather than B's content.
    assert!(!html.contains("B content"), "{html}");
    assert_eq!(
        nearest_preceding_data_state(&html, ">B<"),
        "closed",
        "tooltip B (topmost) should still close on its own Escape: {html}"
    );
}

#[component]
fn ClosedDisabledTooltip() -> Element {
    rsx! {
        Tooltip { disabled: true,
            TooltipTrigger { "Hover me" }
            TooltipContent { "Tooltip text" }
        }
    }
}

#[test]
fn a_closed_disabled_tooltip_reports_closed_and_disabled_state() {
    let html = render(ClosedDisabledTooltip);
    assert!(html.contains(r#"data-state="closed""#), "{html}");
    assert!(html.contains("data-disabled=true"), "{html}");
}

// `TooltipContent` gates its markup on `use_animated_open`, whose real (`web`/`native`)
// implementation only flips its content-mounted signal from inside a `use_effect` -- see
// `test_accordion.rs` for the same, already-established gap. This runs only on the
// SSR-fallback path, where `use_animated_open` returns `open` directly with no effect
// involved.
#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn an_open_tooltip_s_content_renders_the_tooltip_role_and_its_children() {
    let html = render(OpenTooltip);
    assert!(html.contains(r#"role="tooltip""#), "{html}");
    assert!(html.contains("Tooltip text"), "{html}");
}

#[cfg(not(any(feature = "web", feature = "native")))]
#[test]
fn a_closed_tooltip_does_not_render_its_content() {
    let html = render(ClosedDisabledTooltip);
    assert!(!html.contains("Tooltip text"), "{html}");
}
