# Parity status (M10, task 11.5)

This is a point-in-time snapshot for the docs/playground audience, not a
live page (`apps/docs`'s own page routing is unimplemented as of this
change -- a separate, not-yet-approved OpenSpec change). The real,
always-current source of truth is the machine-generated `statics/*.json`
files this snapshot is copied from; regenerate and re-read those directly
rather than trusting this file once it's stale (there is no CI check that
keeps this specific markdown file in sync with `statics/`, unlike the
`.json` files themselves, which are CI-gated -- see `docs/development.md`'s
"Maintainer workflows" section).

There is no `parity.json`, no `cargo xtask parity`, and no single
multi-dimension completion ledger -- that system was deliberately removed
2026-08-31 (`design.md` §9's own removal record) in favor of the
current-state snapshots below, which track *what's built right now*, not a
17-dimension audit trail.

## As of 2026-09-04 (`statics/component_compatibility.json`, `synced_at`)

Regenerate with `cargo run -p adico-xtask -- component-compat sync`; verify
it's current with `cargo run -p adico-xtask -- component-compat check`
(CI-gated as of task 11.4).

| Axis | Built | Not applicable | Not started | Total upstream |
| --- | --- | --- | --- | --- |
| shadcn (`registry/`) | 55 | 1 | 5 | 61 |
| dioxus-components (`registry/`) | 42 | 1 | 2 | 45 |

shadcn's 5 `not_started` are `chart`, `direction`, `field`, `form`, `sonner`
-- `chart` was explicitly scoped out of this change (no Rust charting
crate exists with the required feature set; task 9.1's own audit); the
other four were simply never scheduled into any milestone. shadcn's 1
`not_applicable` is `separator` (built as a primitive-only concern with no
separate styled registry item, per its own recorded exception).

## As of the same date (`statics/primitive_compatibility.json`)

Regenerate with `cargo run -p adico-xtask -- primitive-compat sync`; verify
with `cargo run -p adico-xtask -- primitive-compat check` (CI-gated).

| Axis | Built | Partial | Not started | Total upstream | adico-only extras |
| --- | --- | --- | --- | --- | --- |
| Base UI (`adico-primitives/`) | 34 | 3 | 0 | 37 (+4 utils) | 11 |
| dioxus-primitives (`adico-primitives/`) | 40 | -- | 2 | 42 | 21 |

"adico-only extras" are primitives/components this ecosystem ships with no
upstream equivalent at all (e.g. this change's own new `message_scroller`
primitive) -- a positive count, not a gap.

## Per-item classification (not a build-status axis, but load-bearing for
## trusting the numbers above)

Every one of the 66 `registry:ui`/`registry:component` items also carries
two independently-verified, CI-gated records (`cargo run -p adico-xtask --
primitive-usage check` / `styling-usage check`, both 66/66 passing as of
this date): whether it correctly delegates owned behavior to
`adico-primitives` (or is presentational, or is a recorded exception with a
reason), and whether its styling is Tailwind-only using semantic tokens.
These are what keep the "built" counts above honest -- an item only counts
as "built" once its own source exists *and* both classification checks
pass against it.

## Known, recorded gaps (not silently omitted)

- **Desktop/mobile**: no fixture exists for any component (removed
  2026-08-30, `docs/validation.md`'s own surface table). Not measured for
  any of the 66 items, including this change's own newest ones.
- **Visual-regression suite**: does not exist despite `docs/validation.md`
  claiming otherwise (found while validating M10, task 11.3's own note).
- **Keyboard/accessibility Playwright coverage**: real but partial --
  14 specs under `tests/playwright/`, covering roughly waves 1-5 by name,
  not all 66 items.
- **RTL**: only applicable to (and only tested for) the small set of
  primitives with keyboard navigation (`accordion`, `toolbar`; hand-rolled
  in `menubar`/`calendar`).

See `openspec/changes/build-adico-component-ecosystem/tasks.md`'s own
tasks 11.1-11.3 completion notes for the full reasoning behind each of
these.
