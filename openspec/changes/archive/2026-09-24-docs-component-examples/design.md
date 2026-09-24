## Context

See `proposal.md` — Why. Design-relevant current state:

- `apps/web/src/pages/docs/data.rs` is deliberately "a read-only renderer over
  `registry/registry.json`". A JSON manifest cannot carry a live render, so
  examples need a home in Rust source.
- `apps/web/src/pages/docs/component.rs` is one 109-line template rendering all
  69 components.
- `Tabs`, `CopyButton`, and `Card` are already installed and unused in docs.
- `cn()` is a plain join with no conflict resolution
  (`apps/web/src/adico_lib/cn.rs`), so restyling installed components by passing
  competing utilities through `class` is unreliable.
- `main.rs` is `h-dvh … overflow-hidden` with `main` as the scroll container;
  nothing here may introduce document-level scroll (see
  `redesign-web-visual-foundation`'s design.md, constraint 4).

## Goals / Non-Goals

**Goals**
- Displayed code and rendered code cannot diverge — by construction, not by
  convention or by a CI check.
- Adding examples for a component is a single-file, additive operation.
- Each example owns its own hooks, so examples with state are safe.

**Non-Goals**
- A markdown or MDX authoring pipeline. Examples are Rust, because they must
  compile against the real installed components.
- Editable/live-code examples. That is the playground's job, and this change
  deliberately links to it rather than duplicating it.

## Decisions

### D1 — Code comes from the module's own source via `include_str!`, not a generator

Each example module starts with `const SRC: &str = include_str!("<self>.rs");`.
Example bodies are wrapped in `// doc-example:start <id>` / `// doc-example:end`
comment markers, and a small extractor slices the region out at runtime and
dedents it.

The displayed snippet is therefore *literally the bytes the compiler compiled*.

*Alternative rejected — an `xtask docs-examples sync|check` emitting
`statics/docs_examples/<item>.json`, gated in CI.* This was the approach agreed
when planning, matching `component_props.json` and `styling_usage`. It was
abandoned on discovering the simpler mechanism, because the simpler one is
strictly stronger: a sync-and-check generator still has a window where the
committed JSON is stale (between editing an example and re-running `sync`), and
relies on CI to notice. `include_str!` has no window at all — there is nothing
to regenerate, nothing to commit, and nothing for CI to check. It also removes
an xtask, a `statics/` artifact, and a CI gate from the repo.

*Alternative rejected — `stringify!` on the rsx tokens.* Same sync guarantee,
but the macro normalizes whitespace, so the rendered snippet loses all
formatting and reads nothing like the source a user would write.

*Verified before adopting:* a standalone probe confirmed `include_str!` on a
module's own path plus marker slicing and dedent returns the exact region.

*Cost:* each example module's full source text ships in the wasm bundle. Close
to free — the snippet text would have shipped either way, as a string literal or
a JSON blob.

### D2 — Each example is a real `#[component]`, dispatched by id

```rust
pub const METAS: &[DocExampleMeta] = &[ DocExampleMeta { id: "variants", … } ];
pub fn render(id: &str) -> Element {
    match id { "variants" => rsx! { Variants {} }, _ => rsx! {} }
}
#[component] fn Variants() -> Element { … }
```

Examples must be components, not inline `rsx!` fragments spliced into
`DocsComponent`. An example with state (an open `Dialog`, a controlled `Tabs`)
calls hooks; splicing those into the page component's own hook list would make
the hook count vary with the route parameter, and navigating between two
components with different example counts would violate hook ordering. A
component boundary gives each example its own hook list.

*Alternative rejected — `render: fn() -> Element` fields in a `const` array.*
Requires a named wrapper fn per example purely to satisfy const evaluation; the
`match` is less code and reads better.

### D3 — Syntax highlighting is a small local lexer, not a dependency

A ~100-line tokenizer over the snippet (strings, comments, keywords, types,
numbers, punctuation) emitting `<span>`s with token-colored classes.

*Alternative rejected — `syntect` / `tree-sitter`.* Both are large, and this
target is wasm; shipping a general-purpose highlighting engine to colorize short
RSX snippets is a poor trade. *Alternative rejected — build-time highlighting in
an xtask*: reintroduces exactly the generator D1 removed.

The lexer is deliberately approximate. It colors RSX and Rust well enough to
read and is not a parser; a mis-colored token is a cosmetic bug, not a
correctness one.

### D4 — Presentation composes installed registry components

`DocExample` wraps each example in a `Card`, with `Tabs` switching Preview /
Code and a `CopyButton` on the code pane — all three already installed, per
`adico-web-structure`'s no-reimplementation requirements. Layout classes go on
the app's own wrapper elements, never as competing utilities passed into a
registry component's `class` prop (constraint: `cn()` is a join).

### D5 — Missing examples fall through, they do not error

`examples::for_item(name)` returns `Option`. `None` renders today's page
unchanged. This keeps the tranche incremental: 69 components do not have to be
authored before any of this ships, and no page is ever worse than it is now.

## Risks / Trade-offs

- **Marker comments could drift from the rsx they wrap** (someone edits inside
  the component but outside the markers) → the markers sit immediately inside
  the component body wrapping the whole `rsx!`, so the natural edit is inside
  them. A missing marker degrades to "no code shown", not to wrong code.
- **Variant matrices render many instances** → for popup-family components,
  examples stay closed-by-default, given the pre-existing stuck
  `visibility: hidden` bug on anchored content.
- **Examples are compiled code and can break the build** → that is the point;
  a doc example that no longer compiles is a doc that would otherwise have
  silently lied.
- **Page length grows** → examples come first, prose and props after, so the
  thing a reader came for is above the fold.

## Migration Plan

Additive. No route changes, no data changes, no consumer contract. Revertable as
one commit; reverting restores the prose-and-props page.

Archiving inherits `redesign-web-visual-foundation`'s ordering dependency: both
carry a MODIFIED delta against `adico-web-structure`, which reaches
`openspec/specs/` only when `2026-09-24-merge-apps-into-web` is unblocked and
archived. This change's delta supersedes the previous change's version of the
same requirement, so they must be archived in order.
