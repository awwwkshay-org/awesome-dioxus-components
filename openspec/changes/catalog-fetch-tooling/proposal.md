## Why

`upstreams/dioxus-components/catalog.json` and `upstreams/shadcn/catalog.json`
are hand-maintained, manually-refreshed snapshots with two divergent shapes,
and they only cover two of the four upstream projects adico actually compares
itself against (shadcn and dioxus-components; Base UI and dioxus-primitives
are tracked via hand-written Rust tables instead). Neither catalog records
props or primitive composition, so `primitive-compat`/`component-compat` can
only report presence/absence, not API-level parity. A single `cargo xtask
catalog fetch <axis>` command that snapshots all four axes into one schema
closes that gap while keeping the compat checks fully offline.

## What Changes

- Add a new `cargo xtask catalog fetch <axis>` command (initial axes:
  `shadcn`, `base-ui`, `dioxus-components`, `dioxus-primitives`) that
  live-fetches each upstream's component/primitive inventory and writes a
  revision-pinned snapshot to `statics/catalogs/<axis>.json`.
- Axes are registered, not hardcoded: adding a fifth upstream later (e.g.
  Radix as its own primitive axis, or another component library) is a new
  fetcher registration, not a change to `primitive-compat`/`component-compat`
  or the CLI dispatch plumbing. Each axis declares whether it is a
  **primitive** axis (feeds `primitive-compat`, like Base UI/dioxus-primitives)
  or a **component** axis (feeds `component-compat`, like shadcn/
  dioxus-components), and compat tooling iterates all registered axes of the
  relevant kind rather than naming them individually.
- **BREAKING** (tooling-internal): replace `upstreams/dioxus-components/catalog.json`
  and `upstreams/shadcn/catalog.json` with `statics/catalogs/dioxus-components.json`
  and `statics/catalogs/shadcn.json`; remove the `upstreams/` directory and the
  existing `cargo xtask upstream dioxus-components ...` command (folded into
  `catalog fetch dioxus-components`).
- Add `statics/catalogs/base-ui.json` and `statics/catalogs/dioxus-primitives.json`,
  generated snapshots for the two axes that previously had no catalog file at
  all (only hand-written Rust tables in `primitive_compat.rs`).
- Define one shared catalog schema across all four axes (replacing today's two
  divergent ad hoc shapes), including per component/primitive:
  - which primitives/parts it is composed from, both adico's own composition
    (locally derived from `registry/ui/*.rs` + `registry.json`, no fetch) and
    upstream's own composition (e.g. "shadcn Dialog wraps Base UI
    `Dialog.Root`/`Dialog.Popup`", derived from source parsing);
  - props, with an explicit `props_source` discriminator per prop set —
    `explicit` (real prop table, e.g. Base UI, or Rust `#[derive(Props)]`
    fields), `inherits_from: <axis>.<component>.<part>` (shadcn passthrough via
    `React.ComponentProps<typeof X>`), or `unavailable` (nothing to record).
- Move the existing best-effort live Base UI drift GET
  (`primitive_compat.rs`'s `sync`/`diff` path hitting `base-ui.com`) into
  `catalog fetch base-ui`; `primitive-compat`/`component-compat` `sync`,
  `check`, and `diff` become fully offline, reading only committed
  `statics/catalogs/*.json`.
- Preserve hand-maintained judgment data untouched by fetch/sync: the `status`
  field (`built`/`partial`/`not_started`/`not_applicable`) in
  `BASEUI_COMPONENTS`, and `SHADCN_EXCEPTIONS`/`DIOXUS_COMPONENT_EXCEPTIONS`,
  stay hand-authored Rust tables that `primitive-compat`/`component-compat`
  join against the fetched inventory.
- Update `UPSTREAMS.md`, `docs/adico/m1-primitive-ownership.md`,
  `docs/adico/m3-*.md`, `docs/adico/m4-parity-audit.md`, and the `upstream`
  workflow references in `UPSTREAMS.md` to describe `catalog fetch` and
  `statics/catalogs/` instead of `upstreams/`.

## Capabilities

### New Capabilities
- `upstream-catalog-tooling`: `cargo xtask catalog fetch <axis>` command, an
  extensible axis registry (each axis tagged `primitive` or `component`, new
  axes addable without touching compat-tooling internals), shared
  cross-axis catalog schema (inventory, primitive composition, props with
  `props_source`), and the offline/online boundary between `catalog fetch`
  (only network-touching command) and `primitive-compat`/`component-compat`
  `sync`/`check`/`diff` (fully offline, read committed
  `statics/catalogs/*.json`).

### Modified Capabilities
- none (`primitive-compat`/`component-compat` have no existing OpenSpec
  capability spec today — their behavior is documented in `docs/adico/*.md`
  evidence records, not `openspec/specs/`; this change's new capability spec
  covers their revised data-source contract directly).

## Impact

- `packages/adico-xtask/src/main.rs`: replace the `upstream dioxus-components
  ...` CLI arm with `catalog fetch <axis>`; update usage string.
- `packages/adico-xtask/src/`: new `catalog.rs` (or per-axis modules) housing
  the four fetchers, the shared schema types, and (de)serialization; replaces
  the ad hoc upstream-snapshot code currently inline in `main.rs`.
- `packages/adico-xtask/src/primitive_compat.rs`: read
  `statics/catalogs/dioxus-primitives.json` and `statics/catalogs/base-ui.json`
  instead of the hand-written `BASEUI_COMPONENTS` inventory (status stays
  hand-written and is joined against fetched data); remove the live
  `base-ui.com` GET from `sync`/`diff`.
- `packages/adico-xtask/src/component_compat.rs`: read
  `statics/catalogs/shadcn.json` and `statics/catalogs/dioxus-components.json`
  instead of `upstreams/*/catalog.json`.
- `packages/adico-xtask/src/rust_introspect.rs`: extended/reused to parse
  fetched dioxus-components/dioxus-primitives tarballs for prop structs.
- New Cargo dependency surface in `adico-xtask` for tarball fetch + extract
  (e.g. `ureq`/`reqwest` + `flate2`/`tar`) and HTML table parsing for Base UI
  (e.g. `scraper`) — dev-tooling only, not shipped to consumers.
- `upstreams/` directory removed; `statics/catalogs/*.json` committed in its
  place.
- `UPSTREAMS.md`, `docs/adico/m1-primitive-ownership.md`,
  `docs/adico/m3-migration-queue.md`, `docs/adico/m3-wave5-migration.md`,
  `docs/adico/m4-parity-audit.md`, `openspec/changes/build-adico-component-ecosystem/{design,tasks}.md`:
  update `upstreams/` references.
- No impact on `adico-cli`, `adico-registry-core`, `adico-primitives`, or any
  consumer-facing install flow — this is xtask developer tooling only.
