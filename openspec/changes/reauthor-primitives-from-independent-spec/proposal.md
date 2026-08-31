## Why

`packages/adico-primitives/src` (71 files, 24,510 lines) is currently a fork of
`DioxusLabs/dioxus-components` @ `bf007c15d0cf4d04d3181cc46cf12325aa773955`. 64 files /
22,051 lines are recorded as imported across 9 `provenance/records/*.json` entries;
6,794 of those lines are byte-identical to their import commit, and 24 files have had zero
edits since import. adico's stated boundary is to own its primitive behavior outright
(`docs/architecture.md`, `design.md` §8a) and to synthesize the best of Base UI and
`dioxus-primitives` rather than depend on either — that boundary is not yet true, and the
repo carries a whole license-attribution apparatus (provenance records, SPDX headers,
`third_party/` license copies) to track the gap honestly in the meantime.

That apparatus cannot simply be deleted: MIT and Apache-2.0 both require notices to travel
with derivative works, and the code is still derivative. The apparatus can only be retired
once the code it documents is genuinely rewritten. This change does that rewriting; retiring
the apparatus itself is an explicitly separate, later change (`remove-provenance-tracking`),
gated on this one reaching zero imported files.

## What Changes

- Re-author every one of the 64 files still recorded as imported from
  `DioxusLabs/dioxus-components`, each against a spec derived from the WAI-ARIA Authoring
  Practices Guide, this repo's own `statics/primitive_compatibility.json` parity data, and
  the pinned `statics/catalogs/base-ui.json` / `statics/catalogs/dioxus-primitives.json`
  inventories — not from the upstream implementation text.
- Close the feature/prop parity gaps `cargo xtask primitive-compat diff` reports against
  both Base UI and dioxus-primitives as each file is rewritten, so the result is the union of
  both references' capabilities rather than only what the original fork carried.
- **BREAKING (internal module layout only, not the crate's public API):** flatten every
  per-primitive directory to a single file — `select/` (9 files) → `select.rs`, `combobox/`
  (8 files) → `combobox.rs`, `virtual/` + `virtual_list.rs` (5 files) → `virtual_list.rs`,
  `color_picker.rs` + `color_picker/color_naming.rs` → `color_picker.rs`. Consumers only see
  `adico_primitives::<name>::*`, which is unaffected.
- Add unit and/or Playwright test coverage for every rewritten file before rewriting it — 23
  of the 24 zero-churn imported files currently have zero unit tests, so this change also
  closes that coverage gap as a side effect.
- As each file's rewrite is verified complete, remove its provenance header and its entry
  from the owning `provenance/records/*.json` record; delete a record once its `localPaths`
  is empty. `cargo xtask provenance check` is kept running throughout as the burn-down gate.
- `packages/adico-cli/src/css.rs` (derived from `Wombosvideo/tw-animate-css`) is re-derived
  from the Tailwind v4 `@theme`/`@keyframes` primitives, independently of the primitives
  waves above.
- Wherever a rewrite changes a primitive's public API to close a parity gap, the
  corresponding `registry/ui/*.rs` facade, `registry/registry.json` entry, and generated
  registry output are updated in the same task (vertical slice per `CLAUDE.md`).

**Explicitly not in this change:**
- Deleting `provenance/`, `third_party/`, SPDX headers wholesale, or the `RegistryItem`
  registry-schema `provenance` field — that is `remove-provenance-tracking`, which cannot
  start until this change is fully archived (`provenance check` reports zero imported files).
- `primitive-compat`, `component-compat`, `catalog fetch`, and everything under
  `statics/catalogs/` and `statics/*_compatibility.json` — this change only *consumes* their
  output as a design input; the tooling itself is untouched.
- Tasks `7.8d`/`7.8e` (select/combobox and menu-family migrations) and `7.9` (Base-UI-parity
  tier) in `build-adico-component-ecosystem` are referenced as prerequisites/inputs for two
  of this change's waves, not duplicated here.

## Capabilities

### New Capabilities
- `adico-primitives-authorship`: every file in `adico-primitives` is authored from an
  independent behavioral specification (WAI-ARIA APG, pinned Base UI / dioxus-primitives
  inventories, this repo's own tests) rather than ported from another project's source text,
  and every primitive is a single file.

### Modified Capabilities
(none — `adico-workspace-and-provenance` is a delta inside the still-active, unarchived
`build-adico-component-ecosystem` change and is not yet synced to `openspec/specs/`; it is
intentionally left untouched here and will be updated as an implementation task of the
follow-up `remove-provenance-tracking` change once this one is archived.)

## Impact

- **Code:** `packages/adico-primitives/src/**` (64 of 71 files rewritten, 4 directories
  flattened), `packages/adico-cli/src/css.rs`.
- **Data:** `provenance/records/*.json` shrinks incrementally as files clear; deleted
  entirely only by the follow-up change.
- **Registry:** `registry/ui/*.rs`, `registry/registry.json`, `registry/generated/**` updated
  wherever a rewrite changes public API surface.
- **Tests:** new/extended coverage in `packages/adico-primitives` unit tests and
  `tests/playwright/*.spec.ts`.
- **No consumer-facing breakage expected:** `adico_primitives::<primitive>::*` public paths
  are preserved; `cargo xtask primitive-compat check` is the acceptance gate proving this per
  wave.
- **Known gaps carried forward, not solved here:** no native/desktop or Android/iOS test
  fixture exists (`docs/validation.md`); this change cannot claim coverage on those targets.
