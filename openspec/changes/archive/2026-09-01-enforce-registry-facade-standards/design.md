## Context

See `proposal.md` for motivation. Current state, precisely:

- 37 of 45 `registry/ui/*.rs` files (`accordion` through `virtual_list`,
  excluding the 8 below) already `use adico_primitives::{...}`, and
  `registry.json` already lists `adico-primitives` in every one of those 37
  items' `cargoDependencies` — verified by cross-referencing both sources
  directly, not inferred.
- The other 8 — `button`, `badge`, `card`, `input`, `textarea`, `skeleton`,
  `item`, `pagination` — have no matching module in
  `packages/adico-primitives/src/`. A grep of all 8 files for
  `onkeydown|onkeyup|Key::|use_focus_trap|use_scroll_lock|document::eval|
  onpointerdown|onfocusout|use_signal|use_context_provider|use_context|
  use_effect` returns zero hits, confirming they are genuinely presentational.
- `registry/ui/*.rs` is not a workspace member and has no `Cargo.toml`.
  `packages/adico-cli/src/main.rs` embeds each file via a literal
  `include_bytes!("../../../registry/ui/<name>.rs")` — the compiler treats it
  as opaque bytes, so a syntax error or an unused-but-declared dependency in
  registry source is invisible to `cargo check`/`clippy`. The only mechanism
  that inspects registry source today is `cargo xtask registry validate`
  (rustfmt-checks each file, per `packages/adico-xtask/src/main.rs:251`) and
  `component_compat.rs` (introspects it for the parity ledger) — neither one
  is wired into `.github/workflows/ci.yml`, which runs only `fmt --check`,
  `check --locked --workspace`, `clippy`, and `test`.
- `packages/adico-xtask/src/primitive_compat.rs` and `component_compat.rs`
  already establish the pattern this change extends: a hand-maintained Rust
  `const` table plus a `sync`/`check`/(`diff`) trio that reads repo state and
  writes a committed `statics/*.json` ledger. `component_compat.rs:37-55`'s
  `find_primitive_modules` already parses `use adico_primitives::<module>`
  out of registry source — exactly the signal this change's classification
  needs — and `rust_introspect::introspect_file` already yields component/prop/
  hook data per file.
- Three concrete defects, verified by reading both the registry facade and
  the primitive it wraps:
  - `registry/ui/dialog.rs:50`, `sheet.rs:79`, `alert_dialog.rs:56` each
    render `style { "html {{ overflow: hidden; }}" }` unconditionally while
    open. `packages/adico-primitives/src/dialog.rs:245` and
    `alert_dialog.rs:258` already call the reference-counted
    `use_scroll_lock(open)` from `packages/adico-primitives/src/scroll_lock.rs`,
    whose own doc explicitly exists so "the first overlay to close never
    prematurely re-enables scrolling while a second one is still open" — the
    exact nested case `dialog.rs`'s inline comment claims the raw style
    "naturally handles," which a non-refcounted `overflow: hidden` cannot.
  - `registry/ui/mode_toggle.rs:36` holds `let mut open = use_signal(|| false)`
    and passes `open: Some(open())` / `on_open_change: move |v| open.set(v)`
    into `DropdownMenu`, whose own `use_controlled` (`primitives/menu.rs:104`)
    already manages open state when `open` is left `None` (uncontrolled).
  - `packages/adico-primitives/src/scroll_lock.rs:11-13` says modal scroll
    lock is "a real gap today, since neither currently locks scroll at all" —
    false as of `dialog.rs:245`/`alert_dialog.rs:258` above.
- Explicitly **not** defects, both confirmed by reading the code they refer
  to: the backdrop `onclick → set_open(false)` in `dialog.rs:56`/`sheet.rs:85`
  is the documented workaround for `use_outside_dismiss` being non-functional
  on web (`packages/adico-primitives/src/lib.rs:250-263` prescribes this exact
  technique); and `context_menu.rs:82`'s local
  `use_scroll_lock_while_open` is deliberate — `scroll_lock.rs:14-19` says in
  its own module doc "Do not migrate `context_menu` onto this primitive."
- A fourth defect (R4), found by grepping every `registry/ui/*.rs` file for
  `style {` and `style:` and reading each hit: 10 call sites across 7 files
  (`dialog.rs:55,71`, `alert_dialog.rs:60,75`, `sheet.rs:84,103`,
  `popover.rs:45`, `hover_card.rs:50`, `tooltip.rs:45`, `menubar.rs:50`) pass
  a raw CSS `style` string for `position`/`z-index`, where Tailwind already
  has an exact utility for every value used and, in four of the seven files,
  the same element's `class` list (built via `cn(&[...])`) already sets
  `z-50`. Two sub-cases, confirmed by reading the primitives each file wraps:
  - `dialog.rs`, `alert_dialog.rs`, `sheet.rs`, `menubar.rs` (7 of the 10
    call sites): purely redundant. `packages/adico-primitives/src/dialog.rs`
    and `alert_dialog.rs` never call `Positioner` — dialogs center via the
    `class` list's own `left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2`
    — and `menubar.rs`'s primitive (`packages/adico-primitives/src/menubar.rs`)
    doesn't call `Positioner` either. The inline `style` here duplicates
    values already present, or trivially expressible, in the Tailwind
    `class` string, with no functional risk from removing it.
  - `popover.rs`, `hover_card.rs`, `tooltip.rs` (3 of the 10 call sites):
    redundant *and possibly conflicting*. Each wraps a primitive built on
    the shared `Positioner` (`packages/adico-primitives/src/positioner.rs:392-395`),
    which renders its own element with a computed
    `style: "position: fixed; left: {x}px; top: {y}px;"` (or, before first
    measurement, `"position: fixed; visibility: hidden;"`). The registry
    facade's `style` prop is passed into the same primitive component
    (`PopoverContentProps`/`HoverCardContentProps`/`TooltipContentProps`,
    each `#[props(extends = GlobalAttributes)]`) and merges into the same
    rendered element's attribute list alongside the primitive's own
    computed `style`. Whether the browser/Dioxus attribute-merge order makes
    the registry's static `style` a no-op or lets it silently overwrite the
    primitive's computed `left`/`top` offset is not resolvable from source
    reading alone — it needs Playwright verification (see Risks). Either
    outcome is a reason to remove it, not keep it.
- Semantic-token audit, by grepping every `registry/ui/*.rs` file for raw
  hex/rgb colors and for Tailwind's default-palette color classes
  (`bg-white`, `text-white`, `bg-black`, `bg-slate-500`, etc.) and reading
  the context of every hit: the codebase is already largely token-compliant
  (semantic classes like `bg-background`, `text-foreground`, `bg-primary`,
  `border-input`, `ring-ring`, `bg-sidebar*` dominate), with five identified
  non-token uses, all legitimate on inspection, none a defect:
  - `button.rs`'s `Destructive` variant (`bg-destructive text-white ...`)
    and `dialog.rs`/`alert_dialog.rs`/`sheet.rs`'s overlay backdrop
    (`bg-black/80`) reproduce shadcn's own current published source
    exactly — verified against `statics/catalogs/shadcn.json`'s Button and
    Dialog entries and a direct read of shadcn's `new-york-v4` registry
    source at its pinned revision, both of which use these identical
    literal classes, not semantic tokens, in upstream shadcn itself.
  - `color_picker.rs`'s `AreaThumb` uses `border-white` plus an inline
    `box-shadow` for a visible outline against an arbitrary
    user-picked area color — inherently theme-independent, since the
    thumb must stay legible regardless of the active semantic palette.
  - `theme_switcher.rs`'s preset swatches (`bg-slate-500`, `bg-blue-500`,
    etc.) are explicitly documented in their own doc comment as
    "independent of the active light/dark mode" — they are literal previews
    of the preset palette options being offered, not themable UI chrome.
  - `tag_group.rs`'s `TagRemoveButton` hover tint
    (`hover:bg-black/10 dark:hover:bg-white/10`) is a subtle, mode-aware
    overlay tint independent of any accent color, a common pattern for
    "neutral" interactive-state affordances.
  No mechanical rule can distinguish "legitimate non-token color" from
  "missed token" by pattern alone — that is a judgment call, same as
  `primitive-usage`'s `exception` classification, and this change records
  the judgment rather than trying to encode it as a smarter pattern.

## Goals / Non-Goals

**Goals:**
- Give every current and future `registry:ui`/`registry:component` item a
  named, single classification for its behavior-ownership status, and a
  second, independent classification for its styling status (Tailwind-only
  usage, semantic-token usage).
- Make both classifications mechanically checkable, offline, with no new
  network dependency and no dependency on `statics/catalogs/*.json`.
- Gate both checks in CI, so drift is caught automatically rather than by
  author discipline (today's actual enforcement level for both).
- Fix the four defects the audit found (R1-R4), without expanding into a
  broader remediation pass.
- Record, per item, what it was checked against for styling/token
  inspiration — shadcn and dioxus-components, mirroring the dual-reference
  posture `reauthor-primitives-from-independent-spec` used for primitives —
  and correct any genuine divergence the audit surfaces, without claiming
  class-level parity was mechanically verified where it wasn't.

**Non-Goals:**
- Authoring new primitives so more of the 8 presentational items can
  delegate (e.g. a Pagination primitive with roving focus). That is
  primitives-authoring work with its own upstream-parity implications and
  belongs in a separate change.
- Relocating `registry/`. See Decisions below.
- Fixing `use_outside_dismiss`'s non-functional web behavior — it remains a
  named, recorded exception with its existing follow-up.
- A general "detect any duplicated primitive behavior" checker. Source-text
  analysis can reliably catch the one concrete pattern this change names
  (page-level scroll/overflow injection alongside a scroll-lock-owning
  primitive); it cannot reliably decide arbitrary behavioral duplication.
  Claiming otherwise would be exactly the overclaim
  `adico-component-validation`'s "Platform results are reported honestly"
  requirement forbids.
- Automated, mechanical class-level diffing against shadcn/dioxus-components.
  `statics/catalogs/shadcn.json` and `dioxus-components.json` record
  composition and prop tables, not raw Tailwind class text (confirmed by
  reading both files directly) — extracting and persisting raw class strings
  from upstream source was an explicit non-goal of the change that built
  those catalogs (`catalog-fetch-tooling`'s design.md: "Extracting adico's
  own per-component prop tables or composition... does not belong in the
  fetched catalogs"), and building a second fetch/diff pipeline just for
  Tailwind class text is out of proportion to what a one-time manual audit
  already accomplishes here. The dual-reference check in this change is a
  recorded manual audit, not new fetch/diff tooling.
- Extending `registry/schema.json`/`registry.json` to carry style-token
  metadata. Same reasoning as the behavior-ownership classification: this is
  an internal authoring invariant, not something a consumer or `adico add`
  codepath reads at install time.

## Decisions

**`registry/` stays at the repository root; it does not move to
`packages/adico-registry/`.** The move is mechanically cheap: exactly 46
`include_bytes!` literals in `packages/adico-cli/src/main.rs`, 6 `root.join`
call sites across `packages/adico-xtask/src/main.rs` and
`component_compat.rs:35`, and a handful of doc/spec mentions — no build
script, no glob, no CI job, no Tailwind `@source` entry, and no
`Cargo.toml` member list depends on the path. It is still the wrong move.
`packages/` is this workspace's Cargo-member namespace: every directory
inside it is a crate with a `Cargo.toml`. `registry/` is deliberately not a
crate — `registry/README.md` and `docs/adico/m0-foundation.md` record "a
Cargo styled-component crate" as a rejected alternative, precisely because
`adico` distributes registry source the way shadcn does (consumer-owned
copies via CLI install), not as a compiled dependency. Filing it under
`packages/` would put a non-crate, non-member directory directly beside the
real crate `packages/adico-registry-core`, inviting the exact failure
`CLAUDE.md`'s architecture rules name directly: "Never make a consumer
example or fixture import `registry/` source via a workspace path." It would
also force a rewrite of every `source` path and `checksum` in `registry.json`
plus the affected provenance records for no functional gain. *Alternative
considered*: move it for naming symmetry with `adico-registry-core`. Rejected
— symmetry does not outweigh reintroducing a workspace-path temptation this
repository has already designed against.

**One new `primitive_usage.rs` xtask module, not a new `registry.json` schema
field.** Classification lives in `adico-xtask`/`statics/`, not a new per-item
field in `registry/schema.json`. A schema field would be consumer-facing
(shipped in every installed `registry.json`, requiring `adico-registry-core`
changes and a `registry/generated/*` rebuild) for information that is purely
an internal authoring invariant — no consumer or `adico add` codepath needs to
read a component's classification at install time. *Alternative considered*:
add `"behaviorOwnership": "delegated"` to each item in `registry.json`.
Rejected — it would be dead weight in every consumer's installed manifest and
blur the boundary CLAUDE.md draws between registry semantics
(`adico-registry-core`) and internal tooling invariants
(`adico-xtask`/`statics/`).

**One record file per registry item, not one shared classification table.**
Unlike `primitive_compat.rs`'s single `UPSTREAM_COMPONENTS` Rust `const`
table, this change records each item's classification as its own committed
file, `statics/primitive_usage/<item-name>.json`
(`{ "classification": "delegated" | "presentational" | "exception",
"primitiveModules": [...], "reason": "...", "followUp": "..." }`, the last
two present only for `presentational`/`exception`) — one file per each of the
45 current `registry:ui`/`registry:component` items, named by the item's
`registry.json` name (e.g. `statics/primitive_usage/dropdown-menu.json`).
`primitive_usage.rs`'s `sync`/`check`/`diff` read the directory rather than a
Rust table. Per-item files, rather than one shared table, keep a
classification change to one component's behavior scoped to a one-file diff
instead of a hunk inside a large shared table, and let each record pair with
its own dedicated regression test (see below) rather than one bulk assertion
over the whole set. *Alternative considered*: keep the single-table pattern
`primitive_compat.rs` already established, for consistency with existing
xtask modules. Rejected for this specific check — `primitive_compat`/
`component_compat`'s tables encode upstream-parity judgment that changes in
batches per upstream release; this table encodes an authoring invariant that
changes one component at a time, and per-file records fit that granularity
better.

**One dedicated regression test per registry item, in addition to one test
per failing condition.** The failing-condition tests (task 1.4) use small
synthetic fixtures to prove the checker's logic is correct in general — that
each of the six rules can actually fail, not just pass by construction. They
do not, by themselves, prove that every *real* item in `registry/ui/*.rs`
today satisfies its own recorded classification. A second, per-item test
suite (`primitive_usage_<item>` for all 45 items) reads that item's real
source file and its own `statics/primitive_usage/<item>.json` record and
asserts they agree, so a future change to one component's imports that
silently invalidates its classification fails in that component's own named
test rather than only in an aggregate `check` run.

**A second, sibling xtask module (`styling_usage.rs`), not a third field bolted
onto `primitive_usage.rs`'s records.** Behavior-ownership and styling
compliance are independent axes — an item's primitive delegation can be
correct while its styling isn't, and vice versa (in practice today they
mostly move together, but nothing guarantees that stays true) — so this
change adds `statics/styling_usage/<item-name>.json` as its own directory,
`packages/adico-xtask/src/styling_usage.rs` as its own module, and
`styling-usage sync|check|diff` as its own CLI command family, built to the
exact same shape `primitive_usage.rs` establishes (one record per item, one
regression test per item, `sync`/`check`/`diff`, CI-gated). Each item's
styling record independently tracks: `tailwindOnly` (bool) with an optional
`styleException` list (each entry: the file/line-ish description of the
dynamic value and why it cannot be a static Tailwind class — e.g.
`progress.rs`'s computed indicator width, `sidebar.rs`'s
`--sidebar-width`/`--sidebar-width-icon` custom properties, `theme_builder.rs`'s
generated preview styles); `tokenCompliant` (bool) with an optional
`colorException` list (each entry: the literal class/value used and why —
e.g. `button.rs`'s destructive `text-white`, cited against the exact
upstream source it reproduces where applicable); and `inspiredBy` (which of
`shadcn`/`dioxus-components`/neither the item was checked against during
this change's audit, and a one-line note on what, if anything, was corrected
as a result). *Alternative considered*: fold styling fields into
`primitive_usage.rs`'s existing record schema. Rejected — it would conflate
two independently-failing checks into one file and one CI step, making a
styling-only failure harder to distinguish from a behavior-ownership failure
in CI output, and would force every future primitive-ownership-only change
to also touch styling records for no reason. **Styling records are authored
only after R4 is fixed, not before.** The R4 raw `style:` position/z-index
call sites are staticly expressible in Tailwind by construction — that is
exactly why they are a defect rather than a legitimate `styleException`
(which this change reserves for genuinely runtime-computed values). Recording
them as a temporary exception would make those 7 records violate the
requirement's own definition the moment they are committed. Fixing R4 first
(migration step 5, folded into the same pass as R1) means every
`statics/styling_usage/<item-name>.json` record is honest at authoring time:
no item needs a "known defect, fix pending" placeholder, and the CI gate is
correct from the moment it lands rather than needing a follow-up flip.

**Styling exceptions are judgment calls, recorded like `primitive-usage`'s
`exception` classification — not encoded as smarter pattern matching.**
`styling-usage check` cannot itself decide whether `button.rs`'s destructive
`text-white` is "correct because it matches upstream shadcn" versus "a missed
token" — that determination came from reading shadcn's actual source during
this change's audit, a one-time judgment recorded in the item's
`colorException` entry. The check's mechanical job is narrower and different:
verify that every non-token color use *has* a recorded exception with a
reason (and, for CI purposes, that a *new* non-token color introduced after
this change lands has no matching exception yet, and therefore fails). It
does not, and cannot, verify that a given exception's stated reason is
itself still true — that remains a human review responsibility at exception-
creation time, same as `primitive-usage`'s `exception` classification.

**This is an `adico-registry` requirement, not an `adico-existing-components`
one.** `adico-existing-components` scopes itself to a finite, closeable
project — its own header requirement says the 21-item hardening set "SHALL
reach complete applicable parity... before this change is complete." A rule
that must hold for every future registry item, including ones added after
that hardening project closes, belongs with `adico-registry`'s existing
requirements about what a registry item must describe and how it's validated.
`adico-existing-components`'s own scenario "First-wave primitives retain
their behavior ownership" (`openspec/specs/adico-existing-components/spec.md`)
is a narrower, time-boxed statement of the same idea for five named
components during that hardening effort — it is not superseded or
contradicted by this change's general, permanent rule; the two coexist.

**The scroll-style check is named narrowly, not framed as general
duplication detection.** `primitive-usage check`'s failing condition 5 only
flags a registry file that (a) injects `html { overflow: hidden; }` or an
equivalent page-level scroll/overflow style and (b) delegates to a primitive
whose module already owns scroll locking (currently `dialog`, `sheet`,
`alert_dialog`). The requirement in `specs/adico-registry/spec.md` is worded
to match exactly that check, not a general "SHALL NOT duplicate primitive
behavior" claim — the latter is not something source-text matching can
verify, and asserting it anyway would misrepresent what the tool actually
guarantees.

**Both checks run in CI, immediately.** `.github/workflows/ci.yml` does not
currently run `registry validate` or `registry build`, so a check that only
exists as a local command would enforce nothing beyond what already holds by
convention. Adding `primitive-usage check` and `styling-usage check` to
`ci.yml` is what actually changes the enforcement level from "author
discipline" to "gated," for both axes. Whether `registry validate`/
`registry build` should also join CI is a separate, pre-existing gap this
change does not resolve.

## Risks / Trade-offs

- **A hand-maintained classification record can drift from reality the same
  way the primitive-usage rule itself drifted before this change.** →
  `primitive-usage check` cross-validates every `statics/primitive_usage/<item>.json`
  record against both `registry/ui/*.rs` source (via the reused
  `find_primitive_modules` parsing) and `registry.json`'s `cargoDependencies`,
  in both directions, and each item additionally has its own named regression
  test, so a classification that no longer matches either source fails loudly
  — identifying the specific item — rather than silently going stale.
- **The interactive-behavior marker list for `presentational` items
  (`onkeydown`, `use_signal`, etc.) is a fixed set and could miss a future
  hand-rolled pattern it doesn't name.** → It is verified against all 8
  current presentational files with zero false positives; a new marker can be
  added to the list the same way a new upstream axis is added to `catalog::AXES`
  — a small, local, reviewable diff, not a rewrite.
  Accepted as a known, bounded limitation rather than pursued further, since a
  fully general "any behavior" detector is out of scope (see Non-Goals).
- **Fixing R1/R2 changes runtime behavior in three overlay components and one
  toggle**, not just internal bookkeeping. → Browser/Playwright evidence for
  scroll-lock behavior (including the nested-dialog refcount case) and a
  manual `dx serve` pass for `ModeToggle` are required before this change is
  considered complete, per `docs/validation.md`; a green `cargo test` alone is
  not sufficient evidence for either fix.
- **Fixing R4 (the raw `style:` position/z-index attributes) is not provably
  risk-free for the three `Positioner`-backed files** (`popover.rs`,
  `hover_card.rs`, `tooltip.rs`) — whether the existing inline style was a
  silent no-op or was actively overriding the primitive's computed offset is
  not resolvable from source reading alone. → Removing it and replacing it
  with the equivalent Tailwind class can only make positioning more correct,
  never less (the primitive's own computed `left`/`top` offset is
  unconditionally applied either way once the conflicting `style` prop is
  gone), but this still needs Playwright verification that each of the three
  content types opens anchored to its trigger at the expected side/alignment
  before the fix is considered complete — not just that `cargo test` passes.
- **The five recorded color exceptions could be wrong** — a human judgment
  call made once, during this change's audit, could be mistaken or could
  become stale as shadcn's upstream source changes. → `styling-usage check`
  only verifies that a non-token color has *a* recorded exception with a
  reason, not that the reason is still accurate; re-validating exception
  reasons against current upstream source is a manual review activity (e.g.
  during a future `primitive-usage`-style hardening pass), not something this
  change's mechanical check can or claims to do — stated explicitly rather
  than left implicit, per `adico-component-validation`'s honesty requirement.
- **Editing registry source changes checksums recorded in `registry.json` and
  every installed copy of the components touched by R1-R4.** → All installed
  copies are refreshed exclusively through `adico add`/the existing
  `scripts/refresh-basic-example.sh` pattern, never hand-edited, per
  `CLAUDE.md`'s architecture rule that consumer-style fixtures only change
  through the CLI installation path.

## Migration Plan

1. Add `packages/adico-xtask/src/primitive_usage.rs` (per-item record
   loading, `sync`/`check`/`diff`) and wire the CLI arms in `main.rs`, reusing
   `component_compat.rs`'s primitive-import parsing (extracted to a shared
   location) and `rust_introspect`.
2. Run `primitive-usage sync`, which scaffolds one
   `statics/primitive_usage/<item>.json` record per item from detected
   imports; hand-review and correct each record against the 45-item audit
   above (in particular, mark the identified `exception` items rather than
   accepting the scaffolded `delegated`/`presentational` default), then
   commit all 45 files.
3. Add unit tests proving each of the six failing conditions actually fails
   (not just that today's tree passes), plus one dedicated regression test
   per item (`primitive_usage_<item>`, 45 total) pinning that item's real
   source against its own record.
4. Add `cargo run -p adico-xtask -- primitive-usage check` to
   `.github/workflows/ci.yml`; prove the gate fires by temporarily removing
   `adico-primitives` from one item's `cargoDependencies` and observing the
   job go red, then reverting that test change.
5. Fix R1-R4 together for the files that carry more than one defect, to avoid
   two rounds of checksum churn on the same fixtures: `dialog.rs`, `sheet.rs`,
   and `alert_dialog.rs` each lose both their inline scroll-lock style (R1)
   and their raw `style:` position/z-index props (R4, replaced by Tailwind
   classes, bumping each content element's z-index to `z-[51]` where the
   inline value genuinely exceeded the class's `z-50`); `popover.rs`,
   `hover_card.rs`, `tooltip.rs`, and `menubar.rs` lose their R4-only raw
   `style:` props (the first three need no class addition, since their
   `Positioner`-backed primitive already computes position; `menubar.rs`
   needs an added `absolute` utility class, since its primitive computes no
   position itself). Separately fix R2 (`mode_toggle.rs` redundant signal)
   and R3 (`scroll_lock.rs` stale doc). Add Playwright coverage for the
   scroll-lock refcount behavior (R1) and for Popover/HoverCard/Tooltip still
   anchoring correctly post-removal (R4). Run `cargo xtask registry build`
   and refresh every installed copy of the 8 touched components through the
   CLI.
6. Add `packages/adico-xtask/src/styling_usage.rs` (same shape as
   `primitive_usage.rs`) and wire its CLI arms; run `styling-usage sync` to
   scaffold one `statics/styling_usage/<item>.json` record per item, authored
   against the tree as already fixed by step 5 — no item needs a temporary
   "known defect" exception, since R4 is already gone. Hand-review and
   correct each record against the styling/token audit above; commit all 45
   files. Add unit tests per failing condition plus one dedicated regression
   test per item (`styling_usage_<item>`, 45 total).
7. Add `cargo run -p adico-xtask -- primitive-usage check` and
   `cargo run -p adico-xtask -- styling-usage check` to
   `.github/workflows/ci.yml`; prove each gate fires independently (remove
   `adico-primitives` from one item's `cargoDependencies` for the first;
   reintroduce a raw `style` block or a non-token color with no recorded
   exception for the second; observe each job go red; revert both test
   changes).
8. Record the `inspiredBy` dual-reference audit outcome per item from a
   direct read of shadcn's and dioxus-components' pinned upstream source
   (network access required for this one-time audit step; the committed
   catalogs alone don't carry raw class text — see Non-Goals). Correct any
   genuine divergence found; commit the updated `statics/styling_usage/*`
   records and any resulting source fixes (with their own `registry build`
   and CLI-refresh pass).
9. Update `CLAUDE.md`, `docs/development.md`, `docs/validation.md` (command
   matrix), and `docs/architecture.md` (registry-location rationale and the
   Tailwind-only/semantic-token styling rule).
10. Run the full validation matrix (below) plus Playwright verification for
    the R1/R2/R4 behavior changes (including that the three
    `Positioner`-backed components in R4 still anchor correctly); run
    `openspec validate enforce-registry-facade-standards --strict`.

Rollback: every step is additive or a small, independently revertible source
edit; reverting the commit(s) for steps 5, 8, and 9 restores the prior
(redundant but harmless) registry source with no data loss, since each
`statics/primitive_usage/<item>.json` and `statics/styling_usage/<item>.json`
record's derived fields are fully regenerable from repo state via their
respective `sync` command (only the hand-authored judgment fields —
`classification`/`reason`/`followUp` and
`tailwindOnly`/`tokenCompliant`/exceptions/`inspiredBy` — would need
re-review, and those are preserved by `sync`, not overwritten).

## Open Questions

None — the scope, mechanism, and enforcement points were resolved during
proposal review (audit-and-enforce over migration, xtask subcommands over
schema fields, no registry relocation, a manual recorded audit over new
fetch/diff tooling for the dual-reference styling check).
