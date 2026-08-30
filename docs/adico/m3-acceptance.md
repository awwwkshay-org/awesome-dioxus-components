# M3 acceptance (task 4.9)

Status: complete

This records task 4.9: every upstream item classified suitable for current
reuse (`EXISTING_SHADCN_EQUIVALENT` or `EXISTING_DIOXUS_EXTRA` in
[`upstreams/dioxus-components/inventory.md`](../../upstreams/dioxus-components/inventory.md))
is installable through `adico`, or has a documented blocking primitive/parity
exception. It closes out the migration effort recorded in
[`m3-migration-queue.md`](m3-migration-queue.md) and its five wave reports.

## Classification accounting

`upstreams/dioxus-components/inventory.md` classifies 45 upstream styled
components at the pinned revision `bf007c15d0cf4d04d3181cc46cf12325aa773955`:

| Classification | Count | Disposition |
| --- | --- | --- |
| `EXISTING_SHADCN_EQUIVALENT` | 38 | 37 installed as standalone `registry:ui` items; 1 (`separator`) is a documented exception (below) |
| `EXISTING_DIOXUS_EXTRA` | 6 | 5 installed as standalone `registry:ui` items (Wave 5, task 4.6); 1 (`form`) is a documented exception (below) |
| `NEEDS_PARITY_UPDATES` | 1 (`navbar`) | Out of M3 scope by definition — not "suitable for current reuse"; see below |

37 + 5 = 42 upstream items are installable today. `registry/registry.json`
carries 45 items in total: those 42, the shared `cn` utility, and two
adico-original additions delivered during this same M3-era work (`mode-toggle`
and `theme-switcher`, task 4.8g) that are not upstream `dioxus-components`
items and are therefore outside this accounting.

## Documented exceptions

- **`separator`** (`EXISTING_SHADCN_EQUIVALENT`) has no standalone
  `registry:ui` item. It is consumed exclusively as a primitive dependency of
  `sidebar` (Wave 4) — see
  [`m3-wave4-migration.md`](m3-wave4-migration.md) — and every wave report
  that mentions it (`m3-wave2-migration.md`, this doc's own wave-queue
  cross-reference) explicitly scopes a standalone item as out of that task's
  batch. This is a real, permanent decision, not an oversight: the primitive
  is owned and provenance-tracked, it is simply not independently
  distributable as separator has no meaningful shadcn UI beyond the primitive
  itself in this codebase's current composition.
- **`form`** (`EXISTING_DIOXUS_EXTRA`) is excluded from migration entirely.
  Upstream's `component.rs` is a two-line re-export with an empty `docs.md`;
  its only real content is a demo of a native `<form>` element plus the
  already-migrated Checkbox primitive. There is no Form component/primitive
  to fork — see [`m3-wave5-migration.md`](m3-wave5-migration.md) for the full
  evidence.
- **`navbar`** (`NEEDS_PARITY_UPDATES`) is not part of M3's "suitable for
  current reuse" set by its own classification — `inventory.md` records that
  it "is a navigation-menu candidate but has different upstream
  naming/composition and therefore enters M4 as `NEEDS_PARITY_UPDATES`."
  Nothing in M3 blocks it; it is intentionally deferred to the gap-closing
  milestones (M4 hardening and, for the shadcn Navigation Menu gap itself,
  the M7 complex-component milestone), not silently dropped.

## Installability evidence

Every one of the 42 upstream-sourced items (plus `cn`, `mode-toggle`, and
`theme-switcher`) is proven installable through the real `adico` binary, not
just declared in `registry.json`:

- `examples/basic-spa` and `examples/basic-ssr` each install all 45 registry
  items via `adico add --all` (task 4.8) and compile/run successfully,
  including a real SSR+hydration pass for `basic-ssr`
  (`tests/playwright/fullstack.spec.ts`).
- `tests/installation/*-consumer` fixtures cover the vertical slice and every
  migration wave individually (Waves 1-5), each built through the real `adico`
  binary into a clean fixture.
- `apps/playground` installs the 21 items migrated through task 4.8e's
  hardening pass; it has not yet been refreshed to the full 45-item set added
  since (Waves 2-5, `mode-toggle`, `theme-switcher`) — this is a known,
  named gap, not part of M3's completion bar, and is left open for a future
  playground refresh pass rather than claimed here.

## Verification

```
cargo xtask registry validate
cargo xtask provenance check
cargo xtask parity
cargo fmt --all --check
cargo check --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

| Check | Result |
| --- | --- |
| `cargo xtask registry validate` | passed: 45 item payload(s) in `@adico` |
| `cargo xtask provenance check` | passed: 8 imported record(s), 63 source unit(s) |
| `cargo xtask parity` | passed: 38/38 registry items classified, 6 extras excluded, 0 unclassified |
| `cargo fmt --all --check` | passed |
| `cargo check --locked --workspace` | passed |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | passed |
| `cargo test --locked --workspace` | passed |

## M3 acceptance statement

Every upstream `dioxus-components` item classified `EXISTING_SHADCN_EQUIVALENT`
or `EXISTING_DIOXUS_EXTRA` is either installable through `adico` today (42 of
44) or has a documented, permanent blocking exception recorded above and in
its originating wave report (`separator`, `form`). The one remaining
classified item (`navbar`) is out of M3's scope by its own
`NEEDS_PARITY_UPDATES` classification and is explicitly deferred, not
omitted. `cargo xtask registry validate` passes. M3 is complete; subsequent
work (M4 hardening, M5+ shadcn gap-closing) proceeds against this migrated
base.
