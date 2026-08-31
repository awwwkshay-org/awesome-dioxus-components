## Why

`registry/ui/*.rs` is source-owned Dioxus that consumers copy verbatim; every
piece of reusable interactive behavior it needs is supposed to come from
`adico-primitives`, keeping registry source a thin styled facade. That rule
already holds for 37 of the 45 current registry UI items — each imports
`adico_primitives` and declares the crate in `registry.json`'s
`cargoDependencies`, and the two agree item-for-item today. But nothing states
the rule or checks it. `registry/ui/*.rs` is not a Cargo crate and not a
workspace member (`adico-cli` only `include_bytes!`s it as opaque bytes into
the embedded registry), so `cargo check` never compiles it and CI never runs
`cargo xtask registry validate` — a component could stop delegating, or drift
out of sync with its declared dependency, with nothing noticing.

Auditing the current set while writing this proposal also surfaced three small,
concrete defects: `dialog.rs`, `sheet.rs`, and `alert_dialog.rs` each inject a
raw `html { overflow: hidden; }` style even though the primitive each wraps
already provides a reference-counted `use_scroll_lock`; `mode_toggle.rs` keeps
a redundant open-state signal alongside the `DropdownMenu` primitive that
already owns it; and `scroll_lock.rs`'s own doc comment is stale, claiming a
gap that no longer exists. These are recorded and fixed here because they are
exactly what the missing check would have caught, but they are a small,
separable phase — this change is primarily about the rule and the check, not a
remediation pass.

Auditing every `registry/ui/*.rs` file's styling while writing this proposal
surfaced a second, related gap: 10 call sites across 7 files
(`dialog.rs`, `alert_dialog.rs`, `sheet.rs`, `popover.rs`, `hover_card.rs`,
`tooltip.rs`, `menubar.rs`) pass a raw CSS `style: "position: fixed; ...
z-index: N;"` string to a content component, even though every one of those
files already builds its class list through `cn(&[...])` and Tailwind has an
exact utility for each value used (`fixed`/`absolute`, `inset-0`, `z-50`).
Three of those seven (`popover.rs`, `hover_card.rs`, `tooltip.rs`) wrap a
primitive built on the shared `Positioner` (`packages/adico-primitives/src/positioner.rs:392-395`),
which already computes and sets its own `position: fixed; left: …px; top:
…px;` inline style on the same element; the registry facade's `style` prop
flows into that same element through `#[props(extends = GlobalAttributes)]`
attribute merging, so depending on Dioxus's attribute-merge order this either
does nothing or silently clobbers the primitive's own computed offset — worth
Playwright verification either way, since it is not evident from source
reading alone. This is not primitive-delegation drift; it's raw CSS in a
codebase where CLAUDE.md's Tailwind pipeline is the only intended styling
mechanism, and it belongs in the same audit-and-enforce treatment as the
primitive-ownership rule above, so this proposal now covers registry styling
discipline too — hence its retitling.

Separately, spot-checking every non-token color usage across `registry/ui/*.rs`
(raw hex, Tailwind's default palette classes, or literal `white`/`black`)
found the codebase already mostly disciplined about semantic theme tokens
(`bg-background`, `text-foreground`, `bg-primary`, etc.), with a small number
of uses that are legitimate, not defects — `button.rs`'s destructive variant
`text-white` and the overlay `bg-black/80` in `dialog.rs`/`alert_dialog.rs`/
`sheet.rs` both reproduce shadcn's own current upstream source exactly (verified
against `statics/catalogs/shadcn.json`'s Dialog/Button entries and shadcn's
published `new-york-v4` registry, which use the identical literal classes,
not tokens); `color_picker.rs`'s `border-white` thumb and `theme_switcher.rs`'s
fixed preset swatches are inherently theme-independent by design (a color
picker's own selection indicator, and fixed palette-preview swatches). There
is no dedicated rule distinguishing these legitimate cases from a real
regression today — again, an audit-and-enforce gap this proposal now closes,
following the same declare-and-verify shape as the primitive-ownership rule.

Separately: `registry/` cannot move into `packages/adico-registry/`.
`packages/` is this workspace's Cargo-member namespace — every directory in it
is a crate — while `registry/` is deliberately not a crate
(`docs/adico/m0-foundation.md` records "a Cargo styled-component crate" as a
rejected alternative). Relocating it there would invite the exact failure
`CLAUDE.md`'s architecture rules name directly: something importing registry
source via a workspace path instead of the CLI installer. This proposal makes
no change to `registry/`'s location; the reasoning is recorded in `design.md`.

## What Changes

- Define a per-item behavior-ownership classification for every
  `registry:ui`/`registry:component` item: `delegated` (imports and declares
  `adico-primitives`), `presentational` (no interactive behavior, recorded
  with a one-line reason), or `exception` (behavior a primitive could own but
  deliberately doesn't, recorded with a reason and a follow-up).
- Add `cargo xtask primitive-usage sync|check|diff`, a new, fully offline
  `adico-xtask` command (no `statics/catalogs/*.json` read) that classifies
  every item from `registry.json` + `registry/ui/*.rs`, reading and writing
  one record file per item — `statics/primitive_usage/<item-name>.json`, 45
  files total — rather than one shared table, modeled on the existing
  `primitive_compat.rs`/`component_compat.rs` sync/check/diff pattern and
  reusing `component_compat.rs`'s existing `use adico_primitives::<module>`
  parsing and `rust_introspect`.
- Add one dedicated regression test per registry item (45 tests total,
  `primitive_usage_<item>`) asserting that item's real source agrees with its
  own record, in addition to the checker's own per-failing-condition tests.
- `primitive-usage check` fails when: an item has no record, or a record has
  no matching `registry.json` item; a record lists an `adico_primitives`
  module with no matching file in `packages/adico-primitives/src/`; whether
  `registry.json` declares the `adico-primitives` cargo dependency disagrees,
  in either direction, with whether an item's record lists any primitive
  modules (checked regardless of `delegated`/`exception` label, since an
  `exception` item may still genuinely depend on the crate for its
  non-exceptional behavior); a `presentational` record's item contains an
  interactive-behavior marker or an `adico_primitives` import; a registry
  file injects a page-level scroll/overflow style while its record lists a
  primitive module that owns scroll locking; or a `presentational`/`exception`
  record has an empty reason (or an `exception` record an empty follow-up).
- Add `cargo run -p adico-xtask -- primitive-usage check` to
  `.github/workflows/ci.yml`, so the rule is actually gated rather than
  relying on author discipline — `registry validate`/`registry build` are not
  in CI today and this change does not add them, but the new check is.
- Fix the three defects found during the audit: remove the redundant inline
  scroll-lock style from `dialog.rs`/`sheet.rs`/`alert_dialog.rs` (the
  wrapped primitive's `use_scroll_lock` already covers it); remove
  `mode_toggle.rs`'s redundant open-state signal; correct `scroll_lock.rs`'s
  stale doc comment.
- Define a second per-item classification, alongside behavior-ownership:
  each item's styling record declares whether it styles exclusively through
  Tailwind utility classes (`tailwind-only`) or has a documented, bounded
  exception (a dynamic value that cannot be a static Tailwind class — a
  computed percentage, a runtime CSS custom property — recorded with a
  reason), and separately whether every themable color it uses is a semantic
  design token (`token-compliant`) or has a documented, bounded exception
  (recorded with a reason and, where applicable, the exact upstream source it
  reproduces).
- Add `cargo xtask styling-usage sync|check|diff`, a new, fully offline
  `adico-xtask` command mirroring `primitive-usage`'s exact shape: one record
  file per item under `statics/styling_usage/<item-name>.json` (45 files) and
  one dedicated regression test per item (45 tests,
  `styling_usage_<item>`), plus the aggregate `check`/`diff` pair.
  `styling-usage check` fails when: an item has no record; a `tailwind-only`
  item's source contains a `style { ... }` block or a `style:` attribute
  whose value has no dynamic (runtime-computed) content; a `token-compliant`
  item's source contains a raw hex/rgb color or a Tailwind default-palette
  color class not on the recorded exception list; or a styling/token
  exception has an empty reason.
- Add `cargo run -p adico-xtask -- styling-usage check` to
  `.github/workflows/ci.yml` alongside `primitive-usage check`.
- Fix the newly found styling defect: replace the 10 raw `style:` position/
  z-index call sites in `dialog.rs`, `alert_dialog.rs`, `sheet.rs`,
  `popover.rs`, `hover_card.rs`, `tooltip.rs`, and `menubar.rs` with the
  equivalent Tailwind utility classes (`fixed`/`absolute`, `inset-0`,
  `z-50`/`z-[51]`), removing the `style` prop entirely rather than
  reformatting it, since for the three `Positioner`-backed components
  (`popover`, `hover_card`, `tooltip`) the primitive already computes and
  sets its own position style and the registry facade's copy was at best
  redundant and at worst conflicting.
- For each of the 45 current registry items, record in its styling record
  what it was checked against for idiomatic-styling and token-usage
  inspiration — shadcn's current pinned source (already fetched into
  `statics/catalogs/shadcn.json`'s composition/prop data; raw class strings
  require a fresh, one-time read of shadcn's pinned `new-york-v4` source
  during this audit, since the committed catalog does not persist raw class
  text) and dioxus-components' preview implementation (`statics/catalogs/dioxus-components.json`,
  same caveat) — following the same inspired-not-copied posture
  `reauthor-primitives-from-independent-spec` used for primitives. Correct
  any genuine divergence found (a missing token, a missing state class, an
  idiom either upstream handles better); record deliberate adico-specific
  departures (like the `Verified` badge variant, which neither upstream has)
  as intentional, not as gaps.
- Regenerate `registry.json`/`registry/generated/*` checksums and refresh the
  installed copies in `apps/playground`, `examples/basic-{spa,ssr}`, and the
  affected `tests/installation/*` fixtures through the CLI (never by hand)
  for every file touched by the behavior-ownership and styling fixes above.

## Capabilities

### New Capabilities
- none (this proposal adds requirements to the existing `adico-registry`
  capability; see Modified Capabilities)

### Modified Capabilities
- `adico-registry`: adds four requirements — every registry UI/component item
  declares a behavior-ownership classification and a styling classification
  (Tailwind-only usage, semantic token usage), and both classifications are
  mechanically verified offline and gated in CI.
- (none against `adico-primitives` or `adico-primitives-authorship`: both
  exist only as unsynced deltas inside still-open changes —
  `build-adico-component-ecosystem` and the complete-but-unarchived
  `reauthor-primitives-from-independent-spec` — and are not yet in
  `openspec/specs/`, so neither can take a `## MODIFIED Requirements` block
  here. This change does not touch their content.)

## Impact

- `packages/adico-xtask/src/primitive_usage.rs`: new module (per-item record
  loading, `sync`/`check`/`diff`, unit tests for each failing condition, and
  one regression test per item).
- `packages/adico-xtask/src/styling_usage.rs`: new module, same shape as
  `primitive_usage.rs`, for the styling/token classification.
- `packages/adico-xtask/src/main.rs`: new `primitive-usage sync|check|diff`
  and `styling-usage sync|check|diff` CLI arms and usage text.
- `packages/adico-xtask/src/component_compat.rs`: extract the existing
  `use adico_primitives::<module>` source-parsing helper so `primitive_usage`
  can reuse it instead of duplicating it.
- `.github/workflows/ci.yml`: run `primitive-usage check` and
  `styling-usage check`.
- `registry/ui/dialog.rs`, `sheet.rs`, `alert_dialog.rs`, `mode_toggle.rs`,
  `popover.rs`, `hover_card.rs`, `tooltip.rs`, `menubar.rs`;
  `packages/adico-primitives/src/scroll_lock.rs`.
- `registry/registry.json`, `registry/generated/*` (regenerated checksums via
  `cargo xtask registry build`).
- `apps/playground/src/components/ui/{dialog,sheet,alert_dialog,mode_toggle,popover,hover_card,tooltip,menubar}.rs`,
  `examples/basic-spa/src/components/ui/{dialog,mode_toggle}.rs`,
  `examples/basic-ssr/src/components/ui/{dialog,mode_toggle}.rs`,
  `tests/installation/{dialog-consumer,wave1-consumer,wave2-risk-consumer,theme-consumer}/src/components/ui/*.rs`
  (all CLI-refreshed, not hand-edited).
- `statics/primitive_usage/<item-name>.json` and
  `statics/styling_usage/<item-name>.json`: 45 new committed records each, one
  per current `registry:ui`/`registry:component` item.
- `CLAUDE.md`, `docs/development.md`, `docs/validation.md`: add
  `primitive-usage sync|check|diff` and `styling-usage sync|check|diff` to
  the command matrix.
- `docs/architecture.md`: record why `registry/` stays at the repository root
  rather than moving into `packages/`, and record the Tailwind-only/
  semantic-token styling rule alongside the existing per-project Tailwind
  pipeline description.
- No impact on `adico-registry-core`'s public schema/types — both
  classifications live in per-item hand-maintained `statics/` records read by
  `adico-xtask`, not a new `registry.json`/`schema.json` field.
