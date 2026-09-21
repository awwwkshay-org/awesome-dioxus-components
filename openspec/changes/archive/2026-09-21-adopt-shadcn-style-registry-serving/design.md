## Context

See `proposal.md` for motivation. Current-state facts, verified directly
against the repo:

- `RegistryFile` (`packages/adico-registry-core/src/lib.rs:239-250`) has
  `#[serde(deny_unknown_fields)]` and exactly four fields: `source`,
  `target_root`, `target`, `checksum`. `RegistryManifest::validate` (lib.rs
  ~130-138) requires `format_version == REGISTRY_FORMAT_VERSION` (currently
  `1`, lib.rs:21) — an exact match, not a range.
- `RegistrySourceLoader::load` (lib.rs ~1192-1270) always calls
  `self.validate(&loaded)` before returning. `validate` (lib.rs ~1273-1334)
  loops over every item and every file and does `sha256_hex(self
  .read_source(registry, &source)?)` against `file.checksum` — for an HTTPS
  registry this is one GET per file in the *entire* catalog, not per
  requested item. This is the actual N+1 cost; it is independent of whether
  file content is inlined or fetched separately.
- The official `@adico` registry never goes through this path today: CLI
  `main.rs`'s `configured_catalog` (~477-516) special-cases
  `RegistrySource::Embedded` and calls
  `LoadedRegistry::from_embedded_manifest` directly (lib.rs ~655-670),
  bypassing `loader.load`/`validate` entirely. File bytes for the embedded
  registry come from `ConfiguredRegistryReader::read`'s 72-arm
  `include_bytes!("../../../registry/...")` match (`main.rs` ~525-737) —
  which reaches outside the crate directory and is why `adico-cli` cannot be
  `cargo package`d.
- `packages/adico-xtask/src/main.rs`'s `build_registry` (~263-312) already
  reads `registry/registry.json` and writes `registry/generated/index.json`
  (a bespoke `GeneratedRegistryIndex { format_version, namespace, name,
  description, compatibility, items: BTreeMap<String, String> }`, ~lines
  61-68) plus one `registry/generated/items/<name>.json` per item — a
  verbatim serialized `RegistryItem`, still pointer-based (no content).
  Nothing in the codebase reads either file today.
- `validate_registry` (xtask ~314-334) never touches `registry/generated/`
  at all — it re-runs `load_registry_manifest` against the local
  `registry/` tree, so it re-validates the *authored* source exhaustively
  (cheap: local disk reads) but says nothing about whether the generated
  served tree is in sync. This is the exact gap that let 22/43 generated
  payloads go missing in a prior change.
- `add.rs`'s `plan_source_install` (~174-231) already checksum-verifies
  every file it reads (`reader.read(resolved, &file.source)` then compares
  `checksum(&bytes)` to `file.checksum`) at the point it's actually about to
  install that file — after `RegistryCatalog::resolve` has already narrowed
  to the requested items and their transitive `registryDependencies`, not
  the whole catalog.
- `RegistrySourceLoader`'s HTTPS support derives a registry's file root from
  the *directory of the configured manifest URL*
  (`https_source_root`, lib.rs ~1431-1443) and resolves each file by
  `source_root.join(relative_path)` (lib.rs ~1370-1384). This mechanism is
  reused unchanged by this design, just pointed at a different relative
  path than a file's `source`.
- `tests/installation/awwwkshay-consumer/awwwkshay-registry/` is a real,
  tested, format-1 (pointer-based, no content) external registry. It must
  keep working unmodified.

## Goals / Non-Goals

**Goals:**
- Eliminate the 72 `include_bytes!` escapes so `adico-cli` is publishable.
- Make `adico list`/`adico view`/`adico add <item>` cost proportional to what
  they actually need, not the size of the configured registry's catalog.
- Keep the authored `registry/registry.json` and the existing format-1
  local/HTTPS pointer-based contract fully unchanged and readable.
- Reuse existing mechanisms (`https_source_root`, `RegistryManifest`,
  `RegistrySourceLoader`) rather than introducing new types where an
  existing one already fits.

**Non-Goals:**
- No registry HTTP server — everything here is static-file resolvable.
- No change to `registry/ui/*.rs` source, to any registry item's Rust code,
  or to `components.json`'s schema version.
- No Docker/nginx/Kubernetes work to serve `registry/generated/` — a
  dependent, separate infrastructure change. This design fixes what that
  infrastructure will need to serve, not how it's hosted.
- No `cargo publish` execution.

## Decisions

**D1 — `formatVersion: 2` is additive, and stays generated-output-only.**
`RegistryFile` gains `#[serde(default, skip_serializing_if =
"Option::is_none")] pub content: Option<String>`. `RegistryManifest::validate`
changes from an exact-match check to membership in a new
`const SUPPORTED_REGISTRY_FORMAT_VERSIONS: &[u32] = &[1, 2];`, and
`REGISTRY_FORMAT_VERSION` (the version anything *written* by this crate
uses) becomes `2`. `registry/registry.json` keeps declaring
`"formatVersion": 1` forever — maintainers never author inline content by
hand; only `cargo xtask registry build`'s output and the CLI's embedded
payload are format 2.
*Alternative considered*: express this as a `compatibility.cli`/`runtime`
semver range instead of a second format-version constant. Rejected —
`compatibility` already encodes semantic (feature/behavior) compatibility;
`format_version` is a structural schema-shape discriminant
(`deny_unknown_fields` makes old vs. new *shape* matter), a different axis
that shouldn't be conflated with it.

**D2 — reading a file prefers inline content, falling back to a source-path
fetch.** `RegistryFileReader::read` (CLI trait, `add.rs:20-24`) and
`RegistrySourceLoader::read_resolved_source`/`read_source_location` change
to receive the resolved `&RegistryFile` (not just its `source: &str`). When
`file.content` is `Some`, return its bytes directly with zero I/O; otherwise
fall back to exactly today's behavior (local disk read, HTTPS GET, or the
embedded not-supported error). Call sites already have the `RegistryFile` in
scope (e.g. `add.rs`'s `plan_source_install` loop), so this is a signature
change, not a new lookup. This trait is internal to `adico-cli`/
`adico-registry-core`, not consumer-facing, so changing its signature has no
compatibility concern.

**D3 — `RegistrySourceLoader::validate` drops its eager per-file content
checksum loop; content verification happens exactly once, at read time.**
The loop in `validate` that fetches and checksums *every file of every item*
is removed from the path `load()` calls automatically. Everything else in
`validate` stays: `format_version` support-set check, per-item and registry
`compatibility` checks, `registryDependencies` resolvability, duplicate
`target` detection, checksum *format* validation (`validate_checksum` — this
only checks the string is well-formed hex, not that it matches content), and
dependency-cycle detection — all of which need only the already-fetched
manifest, zero extra file fetches. The exhaustive per-file loop is not
deleted outright; it's extracted into a separately callable
`validate_all_content(&self, registry: &LoadedRegistry) ->
Result<(), RegistryError>` that `cargo xtask registry build`/`registry
validate` call explicitly against the local, free-to-read authored source —
preserving today's exhaustive drift protection for the one thing that most
needs it (the source of truth) while making the CLI's runtime `load()` (used
against potentially large remote registries) proportional.
Consequence: `adico list`/`adico view` stop fetching any file bytes at all
(they only ever needed manifest metadata). `adico add <item>` already
narrows to the requested items' transitive dependencies via
`RegistryCatalog::resolve` *before* `plan_source_install` reads/checksums
files (`add.rs:194-202`, unchanged) — so removing the eager loop makes cost
proportional to the resolved install set with no new verification code on
the install path.
*Alternative considered*: keep the eager loop but issue its requests
concurrently. Rejected — still O(files in the whole registry) network
requests regardless of concurrency, and adds an async/thread-pool dependency
to a currently synchronous, blocking-`reqwest` crate for no architectural
benefit.
*Risk*: for a local registry, fail-fast on a corrupted file now happens per
file at actual read time instead of exhaustively upfront in one `load()`
call. Mitigated by `validate_all_content` remaining available and being
exactly what `cargo xtask registry validate` already runs for the one
registry (the authored one) where upfront exhaustiveness matters and is
free.

**D4 — served-tree layout: a content-free index plus one content-bearing
document per item, both reusing `RegistryManifest`/`RegistryItem`
directly.** `<root>/index.json` is a full format-2 `RegistryManifest` whose
every `RegistryFile.content` is `None` — i.e., exactly the shape of today's
`registry.json`, just regenerated fresh instead of hand-authored, served at
a new location. This single fetch is everything `adico list`/`adico
view`/dependency resolution need. `<root>/<item-name>.json` is a lone
format-2 `RegistryItem` with every one of that item's `files[].content`
populated — fetched *lazily*, only for items actually in the resolved
install set. Item-document URLs resolve via the *existing*
`https_source_root`/`source_root.join(...)` mechanism (D4 reuses it
verbatim against `"{item_name}.json"` instead of a file's `source` path — no
new URL-construction code). For the official registry this makes
`https://adico.awwwkshay.com/r/index.json` the configured manifest URL and
`https://adico.awwwkshay.com/r/<name>.json` each item's document, matching
the proposal's stated shadcn-parity URL exactly.
`RegistrySourceLoader` gains one new method — fetch a single item's
content-bearing document, given its already-resolved `RegistryItem`/name:
for HTTPS/Local sources whose already-loaded item has no inline content, GET
`<source_root>/<name>.json` and use its `files[].content`; if the
already-loaded item *already* has content inline (e.g. a local registry
authored entirely as format 2, or the CLI's embedded payload), return it
unchanged with no further fetch. `add.rs`'s `plan_source_install` calls this
once per resolved item, immediately before reading that item's files.
The bespoke `GeneratedRegistryIndex` struct (`xtask/src/main.rs:61-68`) is
deleted — `build_registry` serializes the manifest itself (with content
stripped) as `index.json` instead of a separate name-to-path map, removing a
type that duplicated what `RegistryManifest` already provides.
*Alternative considered*: one monolithic content-bearing manifest fetched in
full (shadcn's own `registry.json` index actually does inline everything for
small registries). Rejected for `@adico`'s catalog size — that reintroduces
"one big fetch proportional to catalog size, not request size" for the
`list`/`view` path, the exact problem being fixed.

**D5 — `manifestDigest` keeps its current meaning; no redefinition needed.**
Because the index remains a real `RegistryManifest` document (D4), and
`LoadedRegistry::manifest_digest` (lib.rs:680) is already defined as the
sha256 of whatever manifest bytes were fetched, nothing about its meaning
changes — it's the digest of the index document, exactly as computed today.
The earlier open question (raised in `proposal.md`'s prior planning) about
needing to pick a new digest target dissolves once the index isn't replaced
by some other bespoke shape.

**D6 — fix the `$schema` placeholder.** The literal
`https://adico.dev/schema/components.json/v1`
(`packages/adico-cli/src/init.rs:260`, `add.rs:684`) becomes
`https://adico.awwwkshay.com/schema/components.json/v1`. This ships into
every consumer's generated `components.json`; actually hosting a schema
document at that URL is out of scope here (site infrastructure), but the
domain change is in scope since it's a one-line constant fix that shouldn't
wait on that infrastructure landing.

**D7 — official-registry resolution becomes HTTPS-first with an embedded
fallback, scoped narrowly to the official namespace.** `adico init`'s new
default for `@adico` (when not overridden by `--registry`) writes
`{"kind": "https", "url": OFFICIAL_REGISTRY_URL}` instead of
`{"kind": "embedded"}`, where `OFFICIAL_REGISTRY_URL` is one named
`pub const` in the CLI (`"https://adico.awwwkshay.com/r/index.json"`).
`configured_catalog`, specifically when resolving the namespace that equals
`RegistryNamespace::OFFICIAL`, attempts `loader.load` against that HTTPS
source first and, on a network-classed failure
(`RegistryError::NetworkRequest`), falls back to
`LoadedRegistry::from_embedded_manifest` using the committed
`packages/adico-cli/embedded/registry.json` payload. The CLI's reviewable
plan/result records which source actually served the official registry
(network vs. offline snapshot), satisfying the new
`adico-cli-installation` requirement's "Offline resolution reports its
source" scenario.
This fallback is **not** generalized to `RegistrySource::Https` in general:
an organization/third-party registry configured as `https` that fails to
resolve still fails with a clear network error, exactly as today — there is
no embedded snapshot for a third-party registry, and silently swallowing
that failure would be surprising. `{"kind": "embedded"}` also remains fully
available and unchanged in meaning (guaranteed no network attempt) for a
user who explicitly configures it — e.g. air-gapped CI — this design only
changes what `adico init` writes by *default*.
*Alternative considered*: HTTPS-only for the official registry, matching
shadcn exactly. Rejected as a sequencing hazard: it would require the
hosting site (a separate, dependent change) to be live before a published
`adico-cli` gives any user a working `adico add`.

**D8 — CI drift gate covers the committed embedded payload, not the
(no-longer-committed) served tree.** `registry/generated/` is `.gitignore`d
(regenerating ~585 KB of Rust source as escaped JSON strings on every
component-source change made diffs unreadable and doubled review noise);
whatever builds the hosting image runs `cargo xtask registry build` fresh
in its own build step (a separate, dependent change). What *is* committed —
`packages/adico-cli/embedded/registry.json`, required because a crates.io
build can't invoke xtask — needs its own staleness gate:
`cargo xtask registry build` (or a new `--check` mode of the same command)
fails CI if regenerating that file produces a diff from what's committed.
This directly closes the gap that let the prior drift incident (22/43
payloads missing) go undetected — that incident was exactly "generated
output disagreed with source and nothing checked."

**D9 — `docs/adico/organization-registry.md` documents both formats as
valid, doesn't require either.** A short addition states that a
third-party/organization registry may serve items either as plain
pointer-based files (format 1, today's documented static-file-mirror
contract, no server-side logic) or as content-bearing per-item documents
(format 2, this change's shape) — both are read identically by the
installer (D1). `tests/installation/awwwkshay-consumer/awwwkshay-registry/`
needs no changes; it demonstrates format 1 continues to work exactly as
documented.

**D4 refinement, found during implementation — `resolve_item_content` must
gate its fetch on format version, not just on missing content.** The
original wording of D4 (above) checks only whether an item's files already
carry content before deciding to fetch a per-item document. That's
insufficient: a format-1 registry's files *never* carry content by
definition, so an ungated check would attempt `<source_root>/<item>.json`
for every item of every existing format-1 registry — including
`tests/installation/awwwkshay-consumer/awwwkshay-registry/`, which has no
such document and doesn't need one, since its files are already fetched
correctly by `source` path. This was caught by
`cli_integration.rs`'s real fixture test failing during implementation, not
by design review — a good example of why this repo requires running the
actual fixtures, not just unit tests against fabricated data.
Fix: `ResolvedRegistryItem` gained a `format_version: u32` field (populated
from the same `LoadedRegistry` already in scope at both existing
construction sites in `RegistryCatalog`), and `resolve_item_content` takes
`format_version` as an explicit parameter, short-circuiting to "return the
item unchanged" whenever `format_version < 2` — before ever attempting a
fetch. A format-1 item's files keep resolving individually by `source`
path through the pre-existing `resolve_file_bytes` path, completely
unaffected by this method's existence.

## Risks / Trade-offs

- **[Risk]** Splitting the served tree into an index plus per-item documents
  means installing N items against the official HTTPS registry costs 1 (index,
  already fetched during resolution) + N (one per resolved item) requests,
  not 1 total. → **Mitigation**: this is the same shape shadcn itself uses
  (one `registry.json`-equivalent index fetch plus one `/r/<name>.json` per
  requested/dependency component) and is still strictly proportional to the
  request, the property this change exists to establish — not a regression
  against any existing proportional behavior.
- **[Risk]** `RegistryFileReader`/`RegistrySourceLoader`'s signature changes
  (D2, D4) touch every call site that reads a file. → **Mitigation**: both
  are internal traits with a single production implementor each
  (`ConfiguredRegistryReader`, `RegistrySourceLoader` itself); no consumer-
  facing API is affected, and existing `cli_integration.rs`/registry-core
  test coverage exercises every call site.
- **[Risk]** Local-registry fail-fast weakens slightly (D3's risk note).
  → **Mitigation**: `validate_all_content` remains available and is exactly
  what `cargo xtask registry validate` already runs for the registry where
  exhaustive upfront checking matters most (the authored source of truth).

## Migration Plan

No consumer-facing migration: format 1 stays fully readable, so an existing
`components.json`/`awwwkshay-registry`-style organization registry keeps
working unmodified. The one behavioral change a consumer will observe is
`adico init`'s new default for `@adico` (D7) — a fresh `init` after this
change writes `{"kind": "https", ...}` instead of `{"kind": "embedded"}`;
an already-`init`ed project's `components.json` is not rewritten
automatically and keeps whatever it already has. No database, no deployed
service, and nothing to roll back beyond reverting this change's commits —
`registry/registry.json` and every registry item's Rust source are
untouched throughout.
