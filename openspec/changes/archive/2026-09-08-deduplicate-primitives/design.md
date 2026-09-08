## Context

See `proposal.md` - Why for the two spec requirements this change brings the crate
into compliance with. `packages/adico-primitives/src` is 66 files, 28,023 lines,
65 public-or-private modules.

**The worked example this change is trying to reach everywhere else:**
`theme_mode.rs` used to own its own persistence read/write logic; `persisted_state.rs`
was later extracted from it as a standalone, reusable primitive, and
`theme_mode::use_persisted_theme_mode` is now a one-line call into that shared
primitive (`use_persisted_global(&MODE, STORAGE_KEY, mode_token, mode_from_token)`).
`theme_mode::use_theme_mode` itself is a one-line pass-through to the crate's
`use_controlled`. That is the end state every consolidation below is asked to reach:
one implementation, one file that owns it, every other consumer composing it.

## Goals / Non-Goals

**Goals:**
- One implementation per behavior inside `packages/adico-primitives/src`, with every
  survivor implementation a strict superset (by prop, default, ARIA attribute, event,
  and test coverage) of every implementation it replaces.
- Promote the two crate-private modules (`segment`, `time`) that already have
  multiple in-crate consumers, per `adico-primitives/spec.md`'s existing promotion-gap
  language.
- Fix the two Escape-key sites that skip the shared dismissal-layer check as a
  byproduct of consolidating them onto the shared hook — this is a real bug, not only
  duplication, and the fix is the same edit as the consolidation.
- Leave every verified, on-the-books exception (`context_menu`'s click-point anchor,
  `context_menu`'s own scroll-lock technique) untouched, and correct — not silently
  close — the one exception (`menubar`) whose stated justification only partly holds.

**Non-Goals:**
- Registry-side duplication (a different, already-existing requirement:
  `adico-primitives/spec.md:23-27`). One specific finding —
  `registry/ui/time_picker.rs:585-608` inlining `time_picker::angle_to_value`'s body
  instead of calling it, with a `primitive_usage` record misclassified as `delegated`
  instead of `exception` — is named here as a pointer for a future change, not
  addressed in this one.
- The four parallel sorted-item registries (`collection`, `selection`, `tag_group`,
  and the `text_values: HashMap` duplicated in `menu.rs` and `command.rs`), where
  `select`/`combobox` currently double-register into two of them at once. Highest
  regression risk of everything found in the audit; deferred to its own change.
- `DateElement`/`TimeElement` duplication, `use_select_root`/`use_combobox_root`
  duplication, `progress`/`meter` overlap.
- Resolving whether `menubar`/`context_menu`/`navigation_menu` can reuse `menu`'s
  content/item rendering — the question `lib.rs:16-18` leaves explicitly open. This
  change touches only `menubar`'s *placement* (via `Positioner`), never its
  content/item rendering, and does not touch `context_menu` or `navigation_menu`'s
  rendering either.
- Fixing `force_mount`/`use_animated_open` (a pre-existing, separately documented
  defect both `hover_card` and `preview_card` share identically; carried forward
  unchanged by the D2 merge below, not fixed by it).

## Full primitive inventory (all 65 modules)

`D` = duplicate, consolidated by this change. `P` = promotion gap (shared but
private), fixed by this change. `X` = exception, verified correct, left alone.
`F` = facade re-export, already the desired end-state pattern. `doc` = documented,
not merged (see D6). `—` = unchanged. `~` = overlap noted, deferred (see Non-Goals).

| Module | LoC | Verdict | Note |
|---|---:|:--:|---|
| `accordion` | 540 | **D** | hand-rolls `use_controlled` twice (`:190`, `:281`) |
| `alert_dialog` | 423 | — | carries un-retracted defect claim in doc (`:24`) |
| `aspect_ratio` | 61 | — | |
| `autocomplete` | 144 | **F** | 11-item `pub use` of `combobox` — model to copy |
| `avatar` | 443 | — | |
| `calendar` | 2622 | — | largest module; no duplication found |
| `checkbox` | 324 | — | |
| `checkbox_group` | 269 | — | |
| `clipboard` | 148 | — | |
| `collapsible` | 258 | — | |
| `collection` | 766 | ~ | canonical roving-focus registry; registry overlap deferred |
| `color_picker` | 734 | — | |
| `combobox` | 769 | ~ | `use_combobox_root` ≈ `use_select_root`; deferred |
| `command` | 580 | **D** | `CommandSeparator` duplicates `MenuSeparator` |
| `context_menu` | 560 | **X** | click-point anchor; own scroll-lock technique |
| `date_picker` | 1246 | ~ | `DateElement` ≈ `TimeElement`; deferred |
| `dialog` | 395 | — | |
| `direction` | 145 | — | |
| `drag_and_drop_list` | 801 | — | |
| `dropdown_menu` | 33 | **F** | pure `pub use` of `menu` — model to copy |
| `field` | 287 | — | |
| `fieldset` | 121 | — | |
| `form` | 103 | — | |
| `gesture` | 201 | **D** | `moved_past_threshold` built to consolidate, one caller never migrated |
| `hover_card` | 294 | **D** | **survivor** of the hover-disclosure merge |
| `label` | 58 | — | |
| `layer` | 158 | — | |
| `listbox` | 136 | — | |
| `menu` | 953 | **D** | hover delay ×1, Escape ×2, `use_controlled` ×1, separator ×1 |
| `menubar` | 469 | **D**\* | \*gated on live browser verification — see Risks |
| `message_scroller` | 328 | — | |
| `meter` | 259 | ~ | mirrors `progress`; deferred |
| `move_interaction` | 196 | — | carries un-retracted defect claim (`:12-16`) |
| `navigation_menu` | 554 | **D** | hover delay ×1 |
| `number_field` | 423 | — | |
| `otp_field` | 413 | — | |
| `persisted_state` | 282 | — | extracted from `theme_mode` — the pattern that worked |
| `pointer` | 140 | — | carries un-retracted defect claim (`:12`) |
| `popover` | 304 | — | correct `use_escape_key` consumer |
| `preview_card` | 350 | **D** | **absorbed** into `hover_card` |
| `progress` | 126 | ~ | see `meter` |
| `radio_group` | 338 | — | |
| `scroll_area` | 138 | — | |
| `scroll_lock` | 150 | **X** | `:16-19` explicitly forbids migrating `context_menu` |
| `segment` *(private)* | 426 | **P** | 2 consumers; spec already requires it public |
| `select` | 939 | ~ | see `combobox` |
| `selectable` | 333 | **D** | `use_controlled` ×1, `moved_past_threshold` ×1, `selected_texts` ×1 |
| `selection` | 137 | **D** | `selected_text` duplicates `selectable::selected_texts` |
| `separator` | 74 | **D** | **survivor** of the separator merge |
| `slider` | 835 | — | |
| `switch` | 178 | — | |
| `tabs` | 452 | — | |
| `tag_group` | 1062 | ~ | 3rd item registry + own `next_focus_after_removal`; deferred |
| `theme_mode` | 332 | — | **the worked example** — `persisted_state` was extracted from it |
| `time` *(private)* | 16 | **P** | 6+ consumers, still `mod time;` |
| `time_picker` | 711 | ~ | see `date_picker` |
| `toast` | 759 | doc | separate id counter (`:200`) — documented, not merged (D6) |
| `toggle` | 118 | — | |
| `toggle_group` | 283 | — | |
| `toolbar` | 295 | **D** | `ToolbarSeparator` markup byte-identical to `Separator` |
| `tooltip` | 321 | **D** | Escape with no layer check (`:197`) |
| `typeahead` | 565 | — | |
| `virtual_list` | 630 | — | stale reference to old `use_global_escape_listener` name (`:15`) |
| `portal` | 103 | doc | `GlobalSignal` counter (`:48`) — different mechanism, not merged (D6) |
| `positioner` | 805 | **D** | module doc (`:16-29`) stale since `use_reposition_bridge` landed |

Totals: **14 D** (real merges, one gated) + **2 doc-only**, **2 P**, **2 X**, **2 F**,
**7 ~ (deferred, see Non-Goals)**, rest unchanged.

## Decisions

**The governing rule for every consolidation below: the survivor is the union.** No
prop, default, ARIA attribute, event, or test may be dropped. Where two callers
disagree on a *default*, the survivor takes a prop and each caller keeps its own
default — a bare `pub use` cannot re-default, so a thin facade is used instead of a
re-export in that case.

### D1 — Hover-intent delay → one shared primitive (highest value)

Identical generation-counter + `time::sleep` body exists in three places:
`menu.rs:120-141` (`request_hover_open`), `preview_card.rs:43-63` (`request_open`),
`navigation_menu.rs:69-92` (`request_open`). Each file's own comment already admits
this — `menu.rs:33` calls it "the same technique `preview_card.rs`/
`navigation_menu.rs` use … not shared code."

New `packages/adico-primitives/src/hover_intent.rs`, public. **Union of all three:**

| Capability | From | Must survive |
|---|---|---|
| open delay / close delay, reactive | all three | yes |
| supersede-by-generation cancel | all three | yes |
| `delay == 0` ⇒ resolves on the next microtask with no timer wait | all three | yes — `menu`'s 0/0 default depends on it |
| payload is `Option<usize>`, not just `bool` | `navigation_menu` | yes — API must be generic over the value |
| "switching between already-open items ⇒ delay 0" | `navigation_menu:75` | yes — caller computes 0 for this case before calling |

Alternative considered: keep the three implementations and only extract the shared
subset (generation counter + sleep) as a lower-level helper each site still wraps.
Rejected — that's the current state in spirit (all three already share the technique
by copying it), and the audit found it doesn't converge in practice; a single
generic primitive is the only version that structurally prevents a fourth copy.

**Correction (implementation time, 2026-09-07):** two things in this section's
original text didn't survive contact with the actual three bodies, found while
reading `menu.rs:119-140`, `preview_card.rs:43-63`, and
`navigation_menu.rs:69-93` fresh before writing `hover_intent.rs`:

1. **"`delay == 0` ⇒ apply synchronously, no `spawn`" was never true of any of
   the three.** All three unconditionally call `spawn(async move { .. })`; only
   the *sleep inside it* is conditional on `delay > 0`. A `delay == 0` request
   still defers to the next microtask via the spawned task, it just never
   awaits a timer. `hover_intent.rs` preserves this exact shape (spawn always,
   sleep conditionally) rather than introducing a true synchronous fast path —
   that would be a new behavior beyond what any of the three ever did, not a
   preserved one. The union-table row above is corrected to say what the code
   actually guarantees.
2. **The `skip_delay_when` predicate parameter is dropped from the shared
   primitive's own signature.** All three sites already compute their own
   per-request delay from ambient state before invoking the shared
   generation-counter/spawn mechanics (`menu`: which of `hover_open_delay_ms`/
   `hover_close_delay_ms`; `navigation_menu`: 0 when
   `index.is_some() && open_index().is_some()`, otherwise `delay_ms`/
   `close_delay_ms`). Adding a predicate parameter to `hover_intent` itself
   would just relocate logic that already lives correctly at each call site
   and has to close over that site's own signals (`navigation_menu`'s
   predicate reads its own `open_index`, not the requested value) — the
   primitive cannot express that generically without becoming caller-specific
   again. Instead, `hover_intent::HoverIntent<T>::request(&self, value: T,
   delay_ms: u64)` takes the already-resolved delay; each site keeps its own
   delay-selection `if`/`match`, unchanged in substance, just relocated out of
   three near-identical `spawn` bodies into one shared one.

Chosen shape: `packages/adico-primitives/src/hover_intent.rs` exposes
`use_hover_intent<T: Copy + PartialEq + 'static>(setter: Callback<T>) ->
HoverIntent<T>`, where `HoverIntent<T>` is `#[derive(Clone, Copy)]` (a
`Signal<u64>` generation counter plus the `Callback<T>` setter) with a
`request(&self, value: T, delay_ms: u64)` method carrying the generation-bump +
conditional-sleep + supersede-check body every site had duplicated. `Copy` is
load-bearing, not incidental: `MenuContext`, `PreviewCardCtx`, and
`NavigationMenuCtx` are each themselves `#[derive(Clone, Copy)]` (required so
they flow through `use_context_provider`/`use_context` into event handlers
outside render), so whatever replaces their hand-rolled `request_*` method must
be storable as a plain `Copy` field on those structs and callable from
`onmouseenter`/`onmouseleave` closures — ruling out a hook that returns a
non-`Copy` `impl FnMut(..)` closure (the shape `use_escape_key` uses, which
works there only because its callers store the closure locally, not inside a
`Copy` context struct). `bool` callers (`menu`, `hover_card`'s hover-open path)
instantiate `T = bool`; `navigation_menu` instantiates `T = Option<usize>`.

### D2 — `preview_card` → `hover_card`

`preview_card.rs:2` states its parts are "*exactly* `hover_card.rs`'s own shape".
Bodies are byte-identical at the `force_mount` guard, the `merged_attributes` block,
and the `Positioner` call.

| Prop / behavior | `HoverCard` today | `PreviewCard` today | Merged |
|---|---|---|---|
| `open` / `default_open` / `on_open_change` / `disabled` | ✔ | ✔ | ✔ |
| `delay_ms` | — | 600 | **added to `HoverCard`**, default `0` |
| `close_delay_ms` | — | 300 | **added to `HoverCard`**, default `0` |
| content `side` default | `Top` | `Bottom` | prop; facade keeps each default |
| content `align` default | `Center` | `Center` | `Center` |
| `force_mount` | `true` | `true` | `true` |
| Positioner `role` | `"tooltip"` | *(none)* | **new `role` prop**; facade keeps each |
| `offset` | `4.0` | `4.0` | `4.0` |
| trigger `role="button"`, `tabindex="0"`, `aria-describedby` | ✔ | ✔ | ✔ |
| content stays open on `on_mouse_enter` | ✔ | ✔ | ✔ |

Because `side` default, `role`, and the delay defaults all differ, `preview_card.rs`
becomes a **thin facade of components supplying different defaults** into the
`hover_card` implementation — *not* a bare `pub use` like `dropdown_menu` — per the
`adico-primitives-authorship` delta above. Its two existing tests move with it and
must still pass verbatim.

**Correction (implementation time, 2026-09-07):** two things surfaced while
implementing task 7.1, neither anticipated by the table above:

1. **`HoverCardContentProps::role` ended up typed `Option<&'static str>`, not a
   bare `&'static str`**, defaulting to `Some("tooltip")`. `HoverCard`'s own
   `role: "tooltip"` used to reach `Positioner` as a plain named field the
   `#[props(extends = GlobalAttributes)]` sugar captured automatically; making
   it conditional (so `preview_card`'s facade can omit the attribute entirely,
   not merely set it to an empty string) needed it folded into
   `merged_attributes` by hand instead, the same way `"data-state"` already was
   — so the `role` field moved off the `Positioner {}` call and into that
   vector, guarded by `if let Some(role) = props.role`.
2. **`HoverCard` gains a real, if practically unobservable, behavior change: a
   spawn-deferred apply where it previously had none.** `HoverCardCtx` never had
   its own generation/spawn logic at all before this task — `HoverCardTrigger`/
   `HoverCardContent` called `ctx.set_open.call(..)` directly, synchronously,
   every time. Migrating onto `hover_intent::HoverIntent<bool>` (needed so
   `delay_ms`/`close_delay_ms` at non-zero values work at all) means even the
   *default* `delay_ms: 0` case now resolves through a spawned task, per D1's
   own "spawn always, sleep conditionally" design. Raised with the advisor
   before implementing: kept as designed, not special-cased, because (a) a
   zero-delay spawned task resolves within the same `render_immediate_to_vec()`
   pass (confirmed empirically for D1 already), so there is no observable frame
   of lag in a real browser either — Dioxus polls ready tasks in the same work
   cycle, before the next paint; and (b) the alternative (a true synchronous
   fast path at `delay_ms == 0`) would make zero-delay requests structurally
   uncancelable, contradicting the spec's own "the primitive still runs its own
   generation-counter cancellation ... for any other delay value" requirement,
   and would have deleted `hover_intent`'s only deterministic cancellation test
   (`tests/test_hover_intent.rs`'s `a_superseded_request_never_reaches_the_
   setter_only_the_latest_does`, which depends on two same-tick zero-delay
   requests actually racing on generation, not on real elapsed time). Live-
   verified via `dx serve` + Chrome automation against `apps/playground`'s
   `/hover-card` route that hover-open/close still read as instant and
   `role="tooltip"` is still present (task 7.4).
3. **`HoverCardCtx` needed to become `Copy`** (it was `Clone`-only, with every
   field already `Copy`-able and simply never revisited). Its several
   `move`-closures over `ctx` (`HoverCardTrigger`'s `open_event`/`close_event`,
   `HoverCardContent`'s `handle_mouse_enter`/`handle_mouse_leave`) each need
   their own copy of `ctx` to call `ctx.request_open(..)`; a `Clone`-only struct
   moves into the first closure and leaves the rest failing to compile
   (`E0382`, confirmed by trying the `Clone`-only version first). This makes
   `HoverCardCtx` consistent with `MenuContext`/`PreviewCardCtx`/
   `NavigationMenuCtx`, all already `Copy` for the identical reason.

Carried forward, not fixed here: `hover_card.rs:9-20` documents that `force_mount`
is non-functional on every target because `use_animated_open` never honors it. Both
modules share the defect identically today; the merge preserves it as-is and
re-records the note on the survivor (`preview_card.rs`'s own module doc now
cross-references it, rather than duplicating it, since the facade no longer owns a
separate implementation). Fixing it is a behavior change and belongs in its own
change (see Non-Goals).

### D3 — Separators → `separator::Separator`

`toolbar.rs:276` `ToolbarSeparator` markup is byte-identical to `separator.rs:59`
(`role`/`aria_orientation`/`data-orientation`, same `decorative` branch).
`ToolbarSeparator` keeps its `horizontal: Option<bool>` defaulting to
`!(ctx.horizontal)()` and its "no `children`" signature; it delegates the markup to
`Separator` internally. `MenuSeparator` (`menu.rs:767`) and `CommandSeparator`
(`command.rs:337`) are the same three hardcoded-`horizontal` lines twice — both
delegate the same way. `date_picker`'s and `otp_field`'s separators are a different
element (`span`) with different semantics — explicitly not merged.

### D4/D5 — `mod segment` and `mod time` → `pub mod`

`lib.rs:97,100`. `segment` has 2 consumers (`date_picker:20`, `time_picker:27`),
`time` has 6+ (`preview_card`, `typeahead`, `virtual_list`, `toast`, `gesture`,
`menu`). Promotion is `pub(crate)` → `pub` on the module and its items, plus doc
comments; no logic changes. The `adico-primitives` spec delta above already requires
`segment` specifically to be public.

### D6 — Three id generators: document, do not merge

`lib.rs:107` (`AtomicUsize`, `Relaxed`, formats `adico-{n}`) vs `toast.rs:200`
(function-local `AtomicUsize`, `SeqCst`, bare `usize` toast keys) vs `portal.rs:48`
(`GlobalSignal<usize>` — a different mechanism entirely, not a second copy of the
same one).

Reduced to a doc task. These are three counters, but not three implementations of
one behavior: only `lib.rs`'s produces ARIA ids. Merging would buy nothing
behavioral while changing atomic ordering semantics (`SeqCst` → `Relaxed`) on a
change whose whole point is *not* altering behavior. Add a comment at each site
explaining why it is separate; no `SeqCst`→`Relaxed` change.

### D7 — `use_controlled` ×4 (corrected during implementation)

Originally proposed as: `selectable.rs:201` (`use_single_selectable_value`),
`accordion.rs:190` and `:281`, `menu.rs:625` (`MenuRadioGroup`) each re-derive
`lib.rs:154`, "semantically equivalent (`match` vs `unwrap_or_else`)."

**That equivalence claim does not survive inspection and was corrected before
implementation.** Three of the four (`Accordion`, `use_single_selectable_value`,
`MenuRadioGroup`) take `value: Option<ReadSignal<Option<T>>>` — optionality at
the *outer* level, not only inside the signal — specifically because
`use_controlled`'s single-level `Option` cannot distinguish "controlled, but
currently nothing selected" from "not controlled at all": a controlled `prop`
reading `None` in `use_controlled` always falls back to `default`. Routing
these three through `use_controlled` as originally planned would silently turn
a controlled-and-cleared `Select`/`Accordion`/`MenuRadioGroup` into one that
falls back to a default value — a real behavior regression, not a refactor.
This was not a fresh discovery: `AccordionProps::value`'s doc comment already
said it matches `MenuRadioGroup`'s "identical convention," and vice versa —
the crate's own authors had already recognized this as one shared pattern,
just never extracted it.

Only `AccordionMulti` (`values: ReadSignal<Option<Vec<String>>>`) is a genuine,
safe `use_controlled` duplicate — `Vec` already has a native "nothing
selected" representation (`vec![]`), so there is no ambiguous third state to
preserve, and its memo/setter shape is structurally identical to
`use_controlled`'s.

**Revised union:** a new `use_optionally_controlled<T>` primitive, alongside
`use_controlled` in `lib.rs`, implementing the shared tri-state pattern with a
`Callback<Option<T>>` setter (not `Callback<T>` — `Accordion`'s own toggle
logic needs to set `None` directly, to collapse a `collapsible` item, which
`MenuRadioGroup` and `use_single_selectable_value` never need but can trivially
adapt around). `AccordionMulti` migrates onto the existing `use_controlled`
directly; `Accordion`, `use_single_selectable_value`, and `MenuRadioGroup`
migrate onto the new primitive, each keeping its own outer wrapping layer
(toggle-vs-collapse logic, `RcPartialEqValue` type erasure, `Callback<T>`
adaptation) unchanged. A new test proves the controlled-and-cleared case still
ignores `default_value` — the exact property `use_controlled` would have
broken.

### D8 — Escape ×3 (also fixes two real bugs)

- `menu.rs:845` + `:880` re-derive `use_escape_key`'s exact condition — delegate.
- `menu.rs:243` (root `Menu`) has **no layer check** — a root menu nested under a
  dialog reacts to the same Escape meant for the dialog.
- `tooltip.rs:197` has **no layer check**, and `tooltip.rs` never calls `use_layer`
  at all.

Route all three through `use_escape_key`. The hook also calls `prevent_default` and
`stop_propagation`, which the hand-rolled versions do not — verified this doesn't
regress `menu`'s submenu key handling, with a test added per site.

### D9 — `gesture::moved_past_threshold`

Built to consolidate two drift checks; its own tests (`gesture.rs:172-184`) assert
both tolerances. Only `context_menu` migrated. `selectable.rs:309-329` still inlines
`dx*dx + dy*dy > 25.0`. Same math, tolerance passed as an argument; the
`pointer_type() == "touch"` guard is preserved unchanged.

### D10 — `selected_text` / `selected_texts`

`selection.rs:87` and `selectable.rs:89` have an identical `filter_map` body; the
former returns `parts.join(", ")`, the latter `Vec<String>`. Both public signatures
are kept — `selected_text` becomes `selected_texts(..).join(", ")`.

### D11 — `menubar` → `positioner` (exception closed 2026-09-08)

`adico-primitives/spec.md`'s 2026-09-01 Correction exempts `menubar` because
`Positioner` "computes its position once … because the observer-bridge capability …
is unimplemented." That capability's `MutationObserver` path landed and was
live-verified 2026-09-03 (`use_reposition_bridge`, `positioner.rs:309`, wired at
`:526`; a `+75px marginTop` mutation produced an exact `+75px` reposition,
reproduced twice).

But the *scroll* path — the one menubar's exception actually depends on — was never
fire-tested end-to-end. The same archived task recorded that
`IntersectionObserver`/`ResizeObserver`/`scroll` dispatch were confirmed only to
*register* (spied via `addEventListener`), not to fire: the verification
environment's `document.hidden` is permanently `true`, and Chrome throttles those
callbacks under that condition. Menubar's exception was earned by a live *scroll*
measurement (trigger and content both moved by an identical −100px through a
scrolled containing region) — exactly the unverified path.

Decision at the time: do not treat "the observer bridge landed" as sufficient to
migrate. `menubar.rs` was migrated on its own isolated git worktree/branch
(`d11-menubar-positioner-migration`, based on this branch's own HEAD, not
`origin/main`, so the migration developed against the same `Positioner`/`menu.rs`
history the rest of this change already builds on), gated on a real rerun of that
same scroll measurement before merging.

**Update (2026-09-08): that rerun happened and passed, closing the exception.**
`MenubarMenuContext` gained a `trigger_id: Signal<String>` (via `use_unique_id()`
in `MenubarMenu`, applied to `MenubarTrigger`'s own `button`); `MenubarContentProps`
gained `side`/`align` props (`ContentSide::Bottom`/`ContentAlign::Start` defaults,
matching `MenuContentProps`'s own for the same top-level-trigger case);
`MenubarContent`'s body now wraps its children in `positioner::Positioner` instead
of a bare `div` with no placement logic of its own (confirmed
`registry/ui/menubar.rs`'s styled facade never set `absolute`/`fixed` utility
classes either — placement was left entirely to whatever the consumer's own layout
happened to do, which is exactly the bug a screenshot from the user's own,
separately-running `dx serve` session caught live: the unmigrated `File` dropdown
rendered detached from its trigger, overlapping the `Edit`/`View` buttons).

Read `Positioner`'s own `rsx!` body before relying on this: it renders inline (a
`position: fixed`-styled `div` still in its normal place in the component tree, not
a DOM portal), so `MenubarMenu`'s pre-existing `onkeydown` (wrapping both trigger
and content) needed zero changes — keydown events from inside the now-`Positioner`
-wrapped content still bubble up through the normal DOM tree to that same handler.
This was a placement-only migration, matching the task's own stated scope.

The scroll measurement itself: with a menu open, scrolling the page 100px moved
both the trigger and the positioned content from `top: 335.05px`/`367.05px` to
`235.05px`/`267.05px` — an exact 100px delta for both, the same 32.0px offset
preserved before and after. Run via Chrome automation (`javascript_tool` +
`computer` actions against a real, user-selected connected browser, not a
synthetic harness), with `document.hidden` confirmed `true` on that tab
immediately before and after the measurement.

That last point corrects an assumption the prior session's spec Correction made,
not just adds a result: `document.hidden` being permanently `true` in this
environment does **not** block this specific measurement, because
`use_reposition_bridge`'s scroll-tracking path (`positioner.rs`) is a plain
`document.addEventListener('scroll', ..., true)` listener — unlike
`IntersectionObserver`/`ResizeObserver`, it is not gated on page-visibility
throttling. Confirmed by reading `use_reposition_bridge`'s own injected JS before
trusting the empirical result, not by the result alone. The user reviewed the exact
before/after numbers and confirmed they satisfy the exception's dependency, rather
than requiring a literal foregrounded-tab repetition of the same measurement.

`context_menu`'s exception is **not** affected by this — its reason (no anchor
element exists for a click-point) does not depend on the observer-bridge
capability at all.

### D12 — Stale doc / spec text

- `positioner.rs:16-29` still says continuous repositioning "was deferred", citing
  `openspec/changes/build-adico-component-ecosystem/tasks.md` — a path that no
  longer resolves (that change is archived under a dated directory). Rewritten to
  describe the current, partial state (mirrors the spec Correction above).
- **Evidence defect, corrected without changing the conclusion:** `lib.rs:270-289`
  and `positioner.rs:19-27` retract an old defect claim on the grounds that the
  provenance record it cited "does not exist anywhere in this repository's git
  history." It does: added in `1caf731`, modified five times, deleted in `6851afc`.
  The *behavioral* conclusion (live Chrome verification found the listener pattern
  works) is independently supported by the live-verification evidence already in
  those same comments and is kept; only the false "never existed" claim is removed.
  `pointer.rs:12`, `gesture.rs:25`, and `alert_dialog.rs:24` still carry the
  un-retracted original claim and are reconciled to match. `virtual_list.rs:15`
  names a `use_global_escape_listener` that no longer exists under that name;
  corrected to reference the current name.
- **New finding, discovered while writing D8's regression tests:**
  `layer.rs:3-8`'s module doc claims the shared stack is "a single ordered
  registry of every currently-*open* overlay" crate-wide. This is empirically
  false for **sibling** components with no stack-providing common ancestor.
  `use_layer`'s registration (`layer.rs:107-120`) does
  `try_consume_context::<LayerStack>().unwrap_or_else(|| provide_context(..))`
  — Dioxus's `consume_context` walks strictly up the `parent_id` chain
  (`dioxus-core`'s `scope_context.rs::consume_context`), never sideways. Two
  independent overlay roots that are siblings (not one nested inside the
  other's subtree) each find nothing on that walk and each `provide_context`
  their *own* private `LayerStack` on their own scope — confirmed with a
  minimal repro: two sibling components each calling `use_layer` and
  comparing `Rc::as_ptr(&layer.stack.0)` between them yields two distinct
  addresses, not one shared one. Only a genuine ancestor/descendant
  relationship (one overlay's root rendered as a descendant of another's,
  e.g. a `Menu` opened from inside an already-open `Dialog`'s content) shares
  a stack correctly today. This means D8's Escape fix is real and correct for
  that nested case, but does **not** yet help the more common real-app case
  of two unrelated, sibling-level overlays (e.g. a page-level `Tooltip` and a
  page-level `Dialog` triggered from separate, non-nested UI) — those remain
  each trivially "topmost" on their own private stack, unchanged from before
  this task. Fixing `layer.rs` to genuinely share a stack app-wide (e.g. via
  a `GlobalSignal`-backed stack instead of context-provide/consume, or a
  required app-level provider component) is a separate, larger architectural
  change and explicitly out of scope here — flagged, not silently worked
  around. `openspec/changes/deduplicate-primitives`'s own D8 regression tests
  (`test_lib.rs`) were restructured around this finding to use a genuine
  ancestor/descendant fixture instead of siblings, so they exercise the real,
  currently-working path rather than one that doesn't.

Audit claim checked and rejected during planning: `use_global_keydown_listener`
(`lib.rs:236`) is not dead code — `toast.rs:252` uses it for F6 region cycling. Left
unchanged.

## Risks / Trade-offs

- **[Risk] D11's verification cannot run in an automated/backgrounded browser tab** →
  **Mitigation:** treated as a hard gate, not a best-effort check. `menubar.rs` stays
  on its own branch and does not merge into this change unless a human runs
  `dx serve` in a real foreground window and reproduces the exact scroll-follow
  measurement menubar's original exception was earned by. If it can't be run, the
  task ships as proposed-not-implemented and is reported as such, not silently
  dropped or claimed done.
- **[Risk] D1's generic `hover_intent` primitive could subtly change
  `navigation_menu`'s "switching between open items" feel if the predicate is wired
  wrong** → **Mitigation:** port `navigation_menu`'s existing test coverage for that
  exact behavior before switching its call site, per the authorship spec's
  test-before-rewrite requirement.
- **[Risk] D8 changes real behavior** (Escape now respects the layer stack in two
  places it didn't before) **, which could be perceived as scope creep beyond
  "just" deduplication** → **Mitigation:** named explicitly in `proposal.md`'s What
  Changes as a deliberate, in-scope bug fix, not hidden inside the refactor; a
  regression test is added for each of the two sites (tooltip-under-dialog,
  root-menu-under-dialog).
- **[Risk] `cargo xtask primitive-compat sync` output changes** (new public module,
  new hook usages) and could be blindly accepted, masking an unintended API drift →
  **Mitigation:** tasks.md requires reviewing the diff, not just re-running sync.
- **[Trade-off] D6 leaves three id-generation mechanisms in the crate** rather than
  one → accepted: they are not implementations of the same behavior (two produce
  different value shapes for different purposes), and forcing them into one would
  touch atomic-ordering semantics for zero behavioral gain on a change whose premise
  is "no behavior changes except the two named bug fixes."

## Migration Plan

Implementation proceeds low-risk-mechanical first, so a regression is easy to
bisect. See `tasks.md` for the concrete, ordered task list. In brief:

1. Doc/spec-only corrections (D12), then visibility-only promotions (D4/D5).
2. Mechanical, behavior-preserving merges with no cross-cutting risk (D3, D6-doc,
   D9, D10).
3. `use_controlled` consolidation (D7) and the Escape consolidation-plus-bug-fix
   (D8).
4. The new `hover_intent` primitive and its three call sites (D1).
5. The `preview_card`/`hover_card` facade merge (D2).
6. `menubar` → `Positioner` (D11), on its own branch, merged only if its gated live
   verification actually runs and passes.

Each step adds or extends tests before the corresponding edit, per
`adico-primitives-authorship/spec.md`'s existing test-before-rewrite requirement.
Rollback is per-step: each numbered step above is a separable commit: `hover_intent`,
the facade merge, and the menubar migration in particular are structured so any one
can be reverted independently of the others.

## Open Questions

None — every ambiguity found during the audit (the three id generators, the
`menubar` exception's actual current status, whether `use_global_keydown_listener`
is dead code) was resolved during planning rather than deferred; see Decisions above
for each.
