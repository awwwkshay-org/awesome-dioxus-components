## Context

See `proposal.md` for motivation. Current state, precisely:

- `upstreams/dioxus-components/catalog.json` is written by `cargo xtask
  upstream dioxus-components --source <local-clone> --refreshed-at YYYY-MM-DD
  --write` (`packages/adico-xtask/src/main.rs:300-419`). It does **not**
  fetch anything itself — it requires a pre-existing local git clone, reads
  `git -C <source> rev-parse HEAD` for the revision, and walks
  `preview/src/components` and `primitives/src` for inventory. No props.
- `upstreams/shadcn/catalog.json` has no refresh command at all today; it was
  hand-written.
- `packages/adico-xtask/src/primitive_compat.rs` hand-maintains
  `BASEUI_COMPONENTS` (component name, description, `status`) as a static
  Rust table, and does a best-effort live GET of
  `https://base-ui.com/react/components` inside `sync`/`diff` purely to flag
  drift against that hand table (line ~702-711).
- `packages/adico-xtask/src/rust_introspect.rs` already parses Rust source
  with `syn` for component functions, `#[derive(Props)]` field names/types,
  and hook usage — this is the exact mechanism needed for
  dioxus-primitives/dioxus-components prop extraction, just not yet pointed
  at fetched (rather than locally-checked-out) source.
- Spike findings (see conversation): shadcn has no first-party prop tables
  and is mostly `React.ComponentProps<typeof X>` passthrough to whatever
  primitive layer that release wraps (Radix or Base UI — shadcn's registry is
  actively mid-migration between the two, confirmed by the docs page for
  Dialog deferring to Base UI while the fetched `new-york-v4` registry source
  still imports `@radix-ui/react-dialog`); Base UI has genuine structured
  per-part API tables but only as rendered HTML; the two Dioxus axes only
  expose props as Rust `#[derive(Props)]` structs, fetchable via
  `codeload.github.com/DioxusLabs/<repo>/legacy.tar.gz/<sha>` (confirmed
  reachable).

## Goals / Non-Goals

**Goals:**
- One shared Rust schema and one `catalog fetch <axis>` command family for
  all four axes, replacing the two divergent ad hoc snapshot shapes.
- Keep `primitive-compat`/`component-compat` `sync`/`check`/`diff` provably
  offline (no `reqwest`/`ureq` call reachable from those code paths).
- Make prop-set asymmetry across axes a first-class, typed distinction
  (`props_source`) rather than something callers have to infer from empty
  arrays.

**Non-Goals:**
- Extracting adico's *own* per-component prop tables or composition — that's
  already directly readable from `registry/ui/*.rs` via the existing
  `rust_introspect` machinery at check time and does not belong in the
  fetched catalogs (see spec's composition requirement).
  `primitive-compat`/`component-compat` already do this for adico's side;
  this change only replaces the upstream-side input.
- A general-purpose TSX or HTML parser. The shadcn parse only needs to
  detect one pattern family (`React.ComponentProps<typeof X>` plus an
  optional `& { ... }` augmentation); the Base UI parse only needs one page
  shape (an API reference table per part). Neither needs a full AST.
- Authentication/rate-limit handling beyond an optional `GITHUB_TOKEN`
  env var passthrough for the two GitHub-sourced axes.
- Continuous/CI-triggered fetching. `catalog fetch` stays a manually-invoked,
  commit-the-result workflow, same as today's `upstream dioxus-components
  --write`.

## Decisions

**One `catalog` xtask module, one schema, per-axis fetcher functions,
registered in one static table.** Add `packages/adico-xtask/src/catalog/`
with `schema.rs` (shared serde types: `CatalogSnapshot`, `CatalogEntry`,
`PartEntry`, `PropsSource`, `CompositionRef`) and one file per axis
(`shadcn.rs`, `base_ui.rs`, `dioxus_components.rs`,
`dioxus_primitives.rs`), each implementing `fn fetch(revision: Option<&str>)
-> Result<CatalogSnapshot, String>`. `catalog/mod.rs` defines:

```rust
pub enum AxisKind { Primitive, Component }

pub struct AxisDef {
    pub id: &'static str,               // e.g. "base-ui"
    pub kind: AxisKind,
    pub fetch: fn(Option<&str>) -> Result<CatalogSnapshot, String>,
}

pub const AXES: &[AxisDef] = &[
    AxisDef { id: "base-ui", kind: AxisKind::Primitive, fetch: base_ui::fetch },
    AxisDef { id: "dioxus-primitives", kind: AxisKind::Primitive, fetch: dioxus_primitives::fetch },
    AxisDef { id: "shadcn", kind: AxisKind::Component, fetch: shadcn::fetch },
    AxisDef { id: "dioxus-components", kind: AxisKind::Component, fetch: dioxus_components::fetch },
];
```

`main.rs`'s `catalog fetch <axis>` arm looks up the id in `AXES` (or
iterates all of them for `fetch all`) and writes
`statics/catalogs/<axis>.json` via the existing `write_if_changed` helper;
with no axis argument it lists every entry in `AXES` with its `kind`.
`primitive_compat.rs`/`component_compat.rs` never match on axis names —
they filter `AXES` by `AxisKind::Primitive`/`AxisKind::Component`, load
`statics/catalogs/<id>.json` for each match, and merge. Adding a fifth axis
(e.g. Radix, or a second component library — see design's Open Questions in
the prior revision) means: write a new `catalog/<axis>.rs` fetcher, add one
`AxisDef` entry, done — no edits inside `primitive_compat.rs` or
`component_compat.rs`. This mirrors the existing
`primitive_compat`/`component_compat`/`rust_introspect` module split — one
concern per file — rather than growing `main.rs` or the compat modules with
per-axis special cases.

**Dioxus axes fetch a pinned tarball, not a local clone.** Replace
`--source <local-clone>` with: resolve HEAD sha via GitHub's REST API
(`GET /repos/DioxusLabs/<repo>/commits/<branch>`, default branch unless
`--revision <sha>` is passed to pin explicitly), download
`codeload.github.com/DioxusLabs/<repo>/legacy.tar.gz/<sha>`, extract to a
`tempfile` dir, then run the existing `rust_introspect::introspect_file`
over every `.rs` file under `preview/src/components` (dioxus-components
styled) and `primitives/src` (both axes' primitives) to build inventory +
props. This removes the "you must have a local clone checked out" precondition
the current command has, which is the actual gap in "get it from the site."
*Alternative considered*: keep requiring `--source`. Rejected — it's the
exact friction this change is meant to remove, and a temp-dir tarball fetch
is a well-understood pattern with no added runtime dependency beyond
tar+flate2, which the Dioxus axes need regardless for parsing.

**`rust_introspect::PropField` gains a `default` field.** Currently only
`name`/`type`. `syn::Fields` already gives access to the struct's default
attribute (`#[props(default = ...)]`) or `Option<T>`-implied default;
extend `PropField` to carry it (`None` when not determinable) rather than
adding a second parallel introspection path.

**Base UI: fetch the index for inventory, fetch each component page for
props, using `scraper`.** `GET https://base-ui.com/react/components` for
the component list (replacing the current best-effort inventory GET,
relocated here per the spec's fetch-is-sole-network-command requirement);
then `GET https://base-ui.com/react/components/<slug>` per component,
`scraper::Selector` against the API reference table structure (per-part
heading → props table with Name/Type/Default/Description columns, per the
spike's confirmed structure) to build `props_source: explicit` entries.
*Risk*: a from-scratch CSS-selector reverse-engineering of the site's
current markup, no public API. See Risks below.

**shadcn: fetch raw registry source, detect passthrough vs. augmentation
by regex over the type expression, not a TSX parser.** For each shadcn
component, `GET
raw.githubusercontent.com/shadcn-ui/ui/<pinned-sha>/apps/v4/registry/new-york-v4/ui/<name>.tsx`
(sha resolved the same way as the Dioxus axes, via GitHub's commits API).
Scan exported const/function type annotations for
`React\.ComponentProps<typeof (\w+)(?:\.(\w+))?>` — this identifies the
wrapped primitive (module alias + optional dotted part) — and for a
trailing `& \{ ... \}` object type literal, whose fields become explicit
augmentation props. The wrapped module alias (e.g. `DialogPrimitive`) is
resolved to an axis+component+part via that file's own `import` statement
(`@radix-ui/react-dialog` → note as `radix.dialog.*` since adico's catalog
doesn't track Radix as its own axis — see Open Questions — vs
`@base-ui-components/react/dialog` → `base-ui.dialog.*`). Components with
no `React.ComponentProps<typeof ...>` pattern found get
`props_source: unavailable`.

**Hand-maintained judgment stays hand-maintained; fetched inventory is
joined against it by id.** `BASEUI_COMPONENTS`'s `status` field,
`SHADCN_EXCEPTIONS`, and `DIOXUS_COMPONENT_EXCEPTIONS` stay as Rust
`const`/`static` tables in `primitive_compat.rs`/`component_compat.rs`,
keyed by the same identifier the fetched catalog uses. `sync` loads the
fetched `statics/catalogs/*.json`, looks up each entry's hand-maintained
judgment by key (defaulting to `not_started`/no-exception when absent from
the hand table, so a brand-new upstream component doesn't silently vanish),
and never writes back into the hand table.

## Risks / Trade-offs

- **Base UI's page markup has no stability guarantee (no public API).** →
  `catalog fetch base-ui` fails loudly (non-zero exit, clear error message)
  if expected selectors don't match, rather than silently writing an empty
  props table. Because `sync`/`check` read the *committed* snapshot, a
  breaking site change only blocks the next `fetch`, never blocks
  `check`/CI.
- **shadcn is actively mid-migration from Radix to Base UI**, so
  `inherits_from` targets will be inconsistent across components within the
  same fetch (some point at `radix.*`, some at `base-ui.*`) until shadcn
  finishes migrating. This is recorded as-observed, not normalized — treating
  it as a bug to paper over would misrepresent what's actually shipping in
  shadcn today.
- **GitHub API unauthenticated rate limits (60 req/hr)** could make fetching
  many components' commit-pinned raw source slow/flaky in one sitting. →
  Resolve the commit sha once per fetch invocation (one API call), then use
  it for all raw-content/tarball URLs (which don't count against the API rate
  limit, only the API-proper endpoints do); support `GITHUB_TOKEN` env var to
  raise the limit when set.
- **Tarball layout assumption** (`preview/src/components`, `primitives/src`)
  is unchanged from what the current `upstream dioxus-components` command
  already assumes — no new risk introduced, just relocated.
- **New xtask-only dependencies** (`ureq` or `reqwest`, `flate2`, `tar`,
  `scraper`) are dev/build tooling, not shipped to any adico consumer or
  registry install — matches the existing architecture boundary
  (`adico-xtask` is not on the `Dioxus → adico-primitives → registry →
  CLI/registry-core → consumer` dependency path).

## Migration Plan

1. Add `packages/adico-xtask/src/catalog/` (schema + four fetchers) and the
   `catalog fetch <axis>` CLI arm in `main.rs`; keep the old `upstream
   dioxus-components` arm working in parallel during development.
2. Run `catalog fetch all` once, review the four generated
   `statics/catalogs/*.json` files by hand, commit them.
3. Point `primitive_compat.rs`/`component_compat.rs` reads at
   `statics/catalogs/*.json` instead of `upstreams/*/catalog.json` and the
   in-source `BASEUI_COMPONENTS` inventory (status field only stays
   in-source); move the Base UI live GET out of `sync`/`diff`.
4. Remove the `upstream dioxus-components` CLI arm and its supporting
   functions in `main.rs`; delete `upstreams/`.
5. Update `UPSTREAMS.md` and the `docs/adico/*.md` references.
6. Run `cargo fmt --all --check`, `cargo check --locked --workspace`,
   `cargo clippy ... -D warnings`, `cargo test`, `cargo run -p adico-xtask --
   primitive-compat check`, `cargo run -p adico-xtask -- component-compat
   check`, `openspec validate build-adico-component-ecosystem --strict`.

Rollback: the change is additive-then-subtractive within one PR; reverting
the commit restores `upstreams/` and the old command with no data loss, since
`catalog fetch`'s output is fully regenerable from upstream.

## Open Questions

- Should Radix be tracked as a fifth catalog axis (since shadcn's
  in-migration components still point at it), or should `inherits_from:
  radix.*` be accepted as a terminal, unexpanded reference? Doesn't change
  this change's schema, approach, or tasks — the `props_source` discriminator
  already accommodates either answer — so it can be decided during
  implementation once the actual current mix of Radix-vs-Base-UI shadcn
  components is seen from a real fetch.
