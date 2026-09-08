// Navigation Menu's shape (root > list > item > trigger/link + portal > positioner > popup >
// viewport, per its catalog entry's parts) is close to `menubar.rs`'s own roving-focus,
// single-open-at-a-time composite container -- both are a row of top-level items where only one
// dropdown is open at a time, moved between with ArrowLeft/ArrowRight -- so this module reuses
// `menubar.rs`'s `crate::collection` roving-focus wiring the same way. It diverges from
// `menubar.rs` in the two places Navigation Menu is genuinely a different pattern, not a
// relabeled Menubar:
//
// 1. Menubar's menus are `role="menu"` listboxes built on the same roving-focus collection
//    `menubar.rs` itself owns; Navigation Menu's content is arbitrary rich content (cards, link
//    grids), not a listbox -- so `NavigationMenuContent` composes `positioner::Positioner`
//    (matching `hover_card.rs`/`preview_card.rs`'s own composition of it) instead of building a
//    second nested `role="menu"` collection the way `MenubarMenu`/`MenubarContent` do.
// 2. Menubar opens/closes only on click (hover only *switches* an already-open menu, per its own
//    `MenubarTrigger::onmouseenter` guard); a navigation menu's primary activation is hover, with
//    an open delay so moving the pointer across a plain link row doesn't flash every dropdown.
//    `NavigationMenuCtx::request_open` applies `delay_ms` only when *nothing* is currently open;
//    once one item is open, hovering a sibling trigger switches immediately with no delay,
//    matching real navigation-menu feel and Base UI's own behavior. The debounced-by-
//    generation-counter delay mechanics themselves are the shared `crate::hover_intent` primitive
//    (`openspec/changes/deduplicate-primitives`, D1) -- this used to be "the same technique
//    `preview_card.rs` uses ... not shared code", copied independently in three files; it no
//    longer is. `request_open` keeps only the delay-*selection* logic above, which stays local
//    since it reads this context's own ambient `open_index` state, not something the shared
//    primitive can express generically.
//
// Deliberately NOT built (documented, not silently dropped): `NavigationMenuViewport`'s actual
// Base UI behavior -- a single shared popup region that animates its width/height/position while
// morphing between different items' content -- is real choreography-level animation work, a
// separate feature from this primitive's own value/focus/open-state management; each item's
// `NavigationMenuContent` positions independently instead (a real, working, but simpler
// per-trigger popup rather than one shared morphing viewport). `NavigationMenuLink` does not
// register into the roving-focus collection when used as a plain top-level item alongside
// `NavigationMenuTrigger`s (unlike `MenubarItem`, which is always inside an already-open menu's
// own separate collection) -- it stays a normal Tab stop instead of an arrow-key stop, a real
// simplification traded for scope, not an oversight. `Arrow`/`Backdrop`/`Portal` parts are not
// wrapped by dedicated components; a consumer composes `crate::positioner::Arrow` and
// `crate::portal` directly inside `NavigationMenuContent`, the same way no other primitive in
// this crate wraps them either.

//! Defines the [`NavigationMenuRoot`] component and its sub-components, per the WAI-ARIA APG
//! [Disclosure Navigation](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/) shape applied
//! to a roving-focus top-level item row.

use dioxus::prelude::*;

use crate::{
    ContentAlign, ContentSide,
    collection::{CollectionState, collection_item, use_collection_provider, use_item},
    hover_intent::{HoverIntent, use_hover_intent},
    positioner::Positioner,
    use_animated_open,
};

#[derive(Clone, Copy)]
struct NavigationMenuCtx {
    open_index: Signal<Option<usize>>,
    set_open_index: Callback<Option<usize>>,
    disabled: ReadSignal<bool>,
    delay_ms: ReadSignal<u64>,
    close_delay_ms: ReadSignal<u64>,
    /// The shared hover-intent primitive (`openspec/changes/deduplicate-primitives`,
    /// D1/task 6.4) backing [`Self::request_open`].
    hover: HoverIntent<Option<usize>>,
    focus: CollectionState,
}

impl NavigationMenuCtx {
    /// Requests opening `index` (or closing, when `None`) after this
    /// context's configured delay -- except switching directly between two
    /// already-open-adjacent triggers, which applies immediately (no
    /// delay), matching real navigation-menu hover feel.
    fn request_open(&self, index: Option<usize>) {
        let delay = resolve_open_request_delay(
            index,
            (self.open_index)(),
            (self.delay_ms)(),
            (self.close_delay_ms)(),
        );
        self.hover.request(index, delay);
    }
}

/// The pure delay-selection decision `request_open` makes, extracted so it is
/// directly unit-testable without needing a live Dioxus runtime or a real
/// timer (`hover_intent`'s own async supersede mechanics are tested
/// separately, in `tests/test_hover_intent.rs`): switching directly from one
/// already-open item to a sibling applies zero delay regardless of the
/// configured `delay_ms`/`close_delay_ms`, matching real navigation-menu
/// hover feel and Base UI's own behavior.
fn resolve_open_request_delay(
    requested_index: Option<usize>,
    currently_open_index: Option<usize>,
    delay_ms: u64,
    close_delay_ms: u64,
) -> u64 {
    let switching_between_open_items = requested_index.is_some() && currently_open_index.is_some();
    if switching_between_open_items {
        0
    } else if requested_index.is_some() {
        delay_ms
    } else {
        close_delay_ms
    }
}

#[derive(Clone, Copy)]
struct NavigationMenuItemCtx {
    index: ReadSignal<usize>,
    is_open: Memo<bool>,
    disabled: ReadSignal<bool>,
    trigger_id: Signal<String>,
    content_id: Signal<String>,
}

/// The props for the [`NavigationMenuRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuRootProps {
    /// Whether the whole navigation menu ignores user interaction.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end of the row.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Milliseconds to wait after the pointer enters a trigger before
    /// opening its content, when nothing else is currently open.
    #[props(default = ReadSignal::new(Signal::new(200)))]
    pub delay_ms: ReadSignal<u64>,

    /// Milliseconds to wait after the pointer leaves a trigger (or its
    /// content) before closing it.
    #[props(default = ReadSignal::new(Signal::new(150)))]
    pub close_delay_ms: ReadSignal<u64>,

    /// Additional attributes to apply to the root `nav` element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the navigation menu, typically a [`NavigationMenuList`].
    pub children: Element,
}

/// # NavigationMenuRoot
///
/// The `NavigationMenuRoot` component provides shared open-item and
/// roving-focus state to a row of [`NavigationMenuItem`]s.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use adico_primitives::navigation_menu::{
///     NavigationMenuRoot, NavigationMenuList, NavigationMenuItem, NavigationMenuTrigger,
///     NavigationMenuContent, NavigationMenuLink,
/// };
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         NavigationMenuRoot {
///             NavigationMenuList {
///                 NavigationMenuItem { index: 0usize,
///                     NavigationMenuTrigger { "Products" }
///                     NavigationMenuContent {
///                         NavigationMenuLink { href: "/widgets", "Widgets" }
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`NavigationMenuRoot`] component defines the following data
/// attribute you can use to control styling:
/// - `data-disabled`: Indicates whether the menu ignores user interaction.
#[component]
pub fn NavigationMenuRoot(props: NavigationMenuRootProps) -> Element {
    let mut open_index = use_signal(|| None);
    let set_open_index = use_callback(move |index| open_index.set(index));
    let focus = use_collection_provider(props.roving_loop);
    let hover =
        use_hover_intent::<Option<usize>>(Callback::new(move |index| set_open_index.call(index)));

    use_context_provider(|| NavigationMenuCtx {
        open_index,
        set_open_index,
        disabled: props.disabled,
        delay_ms: props.delay_ms,
        close_delay_ms: props.close_delay_ms,
        hover,
        focus,
    });

    rsx! {
        nav { "data-disabled": (props.disabled)(), ..props.attributes, {props.children} }
    }
}

/// The props for the [`NavigationMenuList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuListProps {
    /// Additional attributes to apply to the list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the list, typically [`NavigationMenuItem`]s.
    pub children: Element,
}

/// # NavigationMenuList
///
/// The row of top-level items in a [`NavigationMenuRoot`]. When the row
/// receives focus, it redirects to the most recently focused item (roving
/// tabindex), matching this crate's other composite widgets. Must be used
/// inside a `NavigationMenuRoot`.
#[component]
pub fn NavigationMenuList(props: NavigationMenuListProps) -> Element {
    let mut ctx: NavigationMenuCtx = use_context();

    rsx! {
        ul {
            role: "list",
            tabindex: (!ctx.focus.any_focused()).then_some("0"),
            onfocus: move |_| ctx.focus.set_focus(Some(ctx.focus.recent_focus_or_default())),
            onkeydown: move |event: Event<KeyboardData>| {
                match event.key() {
                    Key::ArrowLeft => ctx.focus.focus_prev(),
                    Key::ArrowRight => ctx.focus.focus_next(),
                    Key::Home => ctx.focus.focus_first(),
                    Key::End => ctx.focus.focus_last(),
                    _ => return,
                }
                event.prevent_default();
            },
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavigationMenuItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuItemProps {
    /// This item's position among its siblings. Used for the roving-focus
    /// order and to identify which item's content, if any, is open.
    pub index: ReadSignal<usize>,

    /// Whether this item is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Additional attributes to apply to the item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the item, typically a [`NavigationMenuTrigger`] +
    /// [`NavigationMenuContent`] pair, or a standalone [`NavigationMenuLink`].
    pub children: Element,
}

/// # NavigationMenuItem
///
/// A single top-level entry in a [`NavigationMenuList`]. Must be used
/// inside a `NavigationMenuList`.
///
/// ## Styling
///
/// The [`NavigationMenuItem`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates whether this item's content is open. Values
///   are `open` or `closed`.
/// - `data-disabled`: Indicates whether this item ignores user interaction.
#[component]
pub fn NavigationMenuItem(props: NavigationMenuItemProps) -> Element {
    let ctx: NavigationMenuCtx = use_context();
    let is_open = use_memo(move || (ctx.open_index)() == Some(props.index.cloned()));
    let trigger_id = crate::use_unique_id();
    let content_id = crate::use_unique_id();

    use_context_provider(|| NavigationMenuItemCtx {
        index: props.index,
        is_open,
        disabled: props.disabled,
        trigger_id,
        content_id,
    });

    rsx! {
        li {
            "data-state": if is_open() { "open" } else { "closed" },
            "data-disabled": (ctx.disabled)() || (props.disabled)(),
            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavigationMenuTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuTriggerProps {
    /// Additional attributes to apply to the trigger element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the trigger.
    pub children: Element,
}

/// # NavigationMenuTrigger
///
/// Opens the ancestor [`NavigationMenuItem`]'s [`NavigationMenuContent`] on
/// hover (after the root's configured delay, or immediately if another
/// item is already open), focus-then-Enter, or click. Must be used inside a
/// `NavigationMenuItem`.
///
/// ## Styling
///
/// The [`NavigationMenuTrigger`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates whether this trigger's content is open.
///   Values are `open` or `closed`.
/// - `data-disabled`: Indicates whether this trigger ignores user
///   interaction.
#[component]
pub fn NavigationMenuTrigger(props: NavigationMenuTriggerProps) -> Element {
    let mut ctx: NavigationMenuCtx = use_context();
    let item_ctx: NavigationMenuItemCtx = use_context();
    let disabled = move || (ctx.disabled)() || (item_ctx.disabled)();
    let index = item_ctx.index;

    let item = use_item(collection_item(ctx.focus, index).disabled(disabled));
    let onmounted = item.onmounted();
    let is_open = item_ctx.is_open;
    let is_focused = move || item.focused();

    rsx! {
        button {
            id: item_ctx.trigger_id,
            type: "button",
            onmounted,

            "aria-expanded": is_open(),
            aria_controls: item_ctx.content_id.cloned(),
            "data-state": if is_open() { "open" } else { "closed" },
            "data-disabled": disabled(),
            tabindex: if is_focused() { "0" } else { "-1" },

            // Always opens (never toggles closed) -- a real mouse click on a
            // trigger is preceded by a real `mouseenter` on the same
            // element, which (via `request_open`'s zero-delay "switching
            // between open items" path, resolved synchronously by `spawn`'s
            // own first poll since there's no `.await` when delay is 0)
            // already updates `is_open()` *before* this handler runs. A
            // toggle read here would see that just-applied hover state, not
            // the state from before this interaction started, and could
            // close what the same click just asked to open (found live: a
            // click on a not-yet-open sibling trigger closed both it and
            // the previously-open one instead of switching). Closing stays
            // reachable via hover-away, a different trigger, or Escape.
            onclick: move |_| {
                if !disabled() {
                    ctx.set_open_index.call(Some(index.cloned()));
                    ctx.focus.set_focus(Some(index.cloned()));
                }
            },
            onmouseenter: move |_| {
                if !disabled() {
                    ctx.request_open(Some(index.cloned()));
                }
            },
            onmouseleave: move |_| {
                if !disabled() {
                    ctx.request_open(None);
                }
            },
            onfocus: move |_| ctx.focus.set_focus(Some(index.cloned())),

            ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`NavigationMenuContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuContentProps {
    /// Side of the trigger to place the content.
    #[props(default = ContentSide::Bottom)]
    pub side: ContentSide,

    /// Alignment of the content relative to the trigger.
    #[props(default = ContentAlign::Start)]
    pub align: ContentAlign,

    /// Whether to keep the content mounted even when closed.
    #[props(default = true)]
    pub force_mount: bool,

    /// Additional attributes to apply to the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the content.
    pub children: Element,
}

/// # NavigationMenuContent
///
/// The floating content region opened by a [`NavigationMenuTrigger`],
/// positioned relative to it via [`crate::positioner::Positioner`]. Stays
/// open while the pointer is over the content itself, not just the
/// trigger. Must be used inside a `NavigationMenuItem`.
///
/// ## Styling
///
/// The [`NavigationMenuContent`] component defines the following data
/// attributes you can use to control styling:
/// - `data-state`: Indicates whether the content is open. Values are
///   `open` or `closed`.
/// - `data-side` / `data-align`: The content's resolved placement relative
///   to its trigger.
#[component]
pub fn NavigationMenuContent(props: NavigationMenuContentProps) -> Element {
    let ctx: NavigationMenuCtx = use_context();
    let item_ctx: NavigationMenuItemCtx = use_context();
    let is_open = (item_ctx.is_open)();
    if !is_open && !props.force_mount {
        return rsx!({});
    }

    let render = use_animated_open(item_ctx.content_id, item_ctx.is_open);
    let index = item_ctx.index;

    let handle_mouse_enter = move |_: Event<MouseData>| ctx.request_open(Some(index.cloned()));
    let handle_mouse_leave = move |_: Event<MouseData>| ctx.request_open(None);

    let mut merged_attributes = vec![dioxus_core::Attribute::new(
        "data-state",
        if is_open { "open" } else { "closed" },
        None,
        false,
    )];
    merged_attributes.extend(props.attributes);

    rsx! {
        if render() {
            Positioner {
                id: Some(item_ctx.content_id.cloned()),
                anchor_id: item_ctx.trigger_id,
                side: props.side,
                align: props.align,
                offset: 4.0,
                role: "region",
                attributes: merged_attributes,

                on_mouse_enter: handle_mouse_enter,
                on_mouse_leave: handle_mouse_leave,

                {props.children}
            }
        }
    }
}

/// The props for the [`NavigationMenuLink`] component.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuLinkProps {
    /// Whether this link represents the current page.
    #[props(default)]
    pub active: ReadSignal<bool>,

    /// Whether selecting this link closes any open content. Defaults to
    /// `true` -- set to `false` for a link that should stay open (e.g. one
    /// that only navigates within the same open panel).
    #[props(default = true)]
    pub close_on_click: bool,

    /// Additional attributes to apply to the anchor element (e.g. `href`).
    #[props(extends = GlobalAttributes, extends = a)]
    pub attributes: Vec<Attribute>,

    /// The children of the link.
    pub children: Element,
}

/// # NavigationMenuLink
///
/// A plain navigable item, usable either as a top-level
/// [`NavigationMenuItem`]'s activator (in place of a
/// [`NavigationMenuTrigger`]/[`NavigationMenuContent`] pair, when that item
/// has no dropdown of its own) or nested inside a [`NavigationMenuContent`]
/// as a sub-navigation link. Selecting it closes any open content by
/// default; set `close_on_click: false` to keep it open.
///
/// ## Styling
///
/// The [`NavigationMenuLink`] component defines the following data
/// attribute you can use to control styling:
/// - `data-active`: Indicates whether this link represents the current
///   page.
#[component]
pub fn NavigationMenuLink(props: NavigationMenuLinkProps) -> Element {
    let ctx: NavigationMenuCtx = use_context();

    rsx! {
        a {
            "aria-current": (props.active)().then_some("page"),
            "data-active": props.active,
            onclick: move |_| {
                if props.close_on_click {
                    ctx.set_open_index.call(None);
                }
            },
            ..props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_open_items_content_renders_without_force_mount() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                NavigationMenuRoot {
                    NavigationMenuList {
                        NavigationMenuItem { index: 0usize,
                            NavigationMenuTrigger { "First" }
                            NavigationMenuContent { force_mount: false, "first content" }
                        }
                        NavigationMenuItem { index: 1usize,
                            NavigationMenuTrigger { "Second" }
                            NavigationMenuContent { force_mount: false, "second content" }
                        }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("first content"), "{html}");
        assert!(!html.contains("second content"), "{html}");
    }

    #[test]
    fn active_link_gets_aria_current_page() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                NavigationMenuRoot {
                    NavigationMenuList {
                        NavigationMenuItem { index: 0usize,
                            NavigationMenuLink { active: true, href: "/", "Home" }
                        }
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains(r#"aria-current="page""#), "{html}");
    }

    /// Regression coverage for task 6.4 (`openspec/changes/deduplicate-primitives`):
    /// before this change, this decision lived inline inside `request_open`'s
    /// generation/spawn body, untestable without driving a real (or paused)
    /// async timer -- a harness this crate's own `test_toast.rs` documents a
    /// real, failed attempt at building. Extracted as a pure function, it is
    /// directly testable: switching between two already-open items must
    /// resolve to zero delay regardless of how large `delay_ms`/`close_delay_ms`
    /// are configured.
    #[test]
    fn switching_between_open_items_ignores_the_configured_delay() {
        assert_eq!(
            resolve_open_request_delay(Some(1), Some(0), 99_999, 99_999),
            0
        );
    }

    #[test]
    fn opening_when_nothing_else_is_open_uses_the_configured_open_delay() {
        assert_eq!(resolve_open_request_delay(Some(0), None, 200, 150), 200);
    }

    #[test]
    fn closing_uses_the_configured_close_delay() {
        assert_eq!(resolve_open_request_delay(None, Some(0), 200, 150), 150);
    }
}
