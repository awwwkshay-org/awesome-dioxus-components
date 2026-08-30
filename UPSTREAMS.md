# Upstream sources and provenance

This repository is dual-licensed under MIT and Apache-2.0. Source imported,
forked, or materially ported from an upstream project must also retain every
applicable upstream license and notice.

## Required record

Every imported source unit has one JSON record under `provenance/records/` that
conforms to `provenance/schema.json`. A record identifies the immutable upstream
revision, original and local paths, license obligations, import date, and every
known local divergence. Multiple local files may use a single record only when
they were imported together and have identical origin and license details.

## Workflow

1. Record an immutable commit SHA before copying or porting source.
2. Copy the upstream license and NOTICE material required for the source.
3. Add a provenance record and, where required, a changed-file notice.
4. Update the record when code is ported, cherry-picked, or materially changed.
5. Run `cargo xtask provenance check` once M1 introduces that command. Until
   then, validate records against the checked-in schema during review.

## Initial upstream candidates

| Upstream | Intended use | License | Status |
| --- | --- | --- | --- |
| `DioxusLabs/dioxus-components` | Owned Dialog/Select primitive fork and future styled starting points | MIT OR Apache-2.0 | Dialog/Select closure imported from `bf007c15d0cf4d04d3181cc46cf12325aa773955`; see `provenance/records/adico-primitives-dialog-select.json`. |
| `shadcn-ui/ui` | Catalog and behavior/parity reference | MIT | Snapshot/reference only; do not copy source without a record. |

Company-curated registry authors are responsible for provenance records and
license notices in their own registry source. Adico preserves a registry item's
declared provenance in installation plans and does not imply that a company
item is official adico source.

## M1 evidence

The pinned upstream inventory, owned Dialog/Select closure, target compilation
results, and offline refresh command are recorded in
[`docs/adico/m1-primitive-ownership.md`](docs/adico/m1-primitive-ownership.md).

## M3 evidence

The existing-component migration queue and its per-wave migration records
(provenance, registry entries, installation fixtures, and browser test
results) are recorded in:

- [`docs/adico/m3-migration-queue.md`](docs/adico/m3-migration-queue.md) — the full queue and its scope decisions
- [`docs/adico/m3-wave1-migration.md`](docs/adico/m3-wave1-migration.md)
- [`docs/adico/m3-wave2-migration.md`](docs/adico/m3-wave2-migration.md)
- [`docs/adico/m3-wave3-migration.md`](docs/adico/m3-wave3-migration.md)
- [`docs/adico/m3-wave4-migration.md`](docs/adico/m3-wave4-migration.md)
- [`docs/adico/m3-wave5-migration.md`](docs/adico/m3-wave5-migration.md) — Dioxus-only extras (task 4.6); `form` is excluded there with evidence
- [`docs/adico/m3-acceptance.md`](docs/adico/m3-acceptance.md) — M3 acceptance record (task 4.9): classification accounting, documented exceptions (`separator`, `form`, `navbar`), and installability evidence

## M4 evidence

- [`docs/adico/m4-parity-audit.md`](docs/adico/m4-parity-audit.md) — M4 parity audit (task 5.1): evidence-backed comparison of the 38 tracked components against upstream shadcn and this repo's own rendered output, across the `api`, `visual`, `variants`, `states`, `darkMode`, `rtl`, `responsive`, `desktop`, `docs`, and `ssrHydration` dimensions deferred from M3
