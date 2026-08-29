## Context

`examples/` currently has seven workspace members. `basic`, `fullstack`, and
`desktop` are real: each has `adico.lock`/`components.json` and is
initialized/refreshed through the real `adico` CLI, and each is named as
evidence in `parity.json` and `docs/adico/m2-vertical-slice.md`. `dashboard`,
`forms`, `kitchen-sink`, and `web` are `dioxus::launch` stubs (10-line
`main.rs`, no lockfile, no installed components) left over from
`build-adico-component-ecosystem`'s original M0 scaffolding, which planned to
grow them progressively as later milestones (M4–M11) landed more components.
See proposal.md for the "why" of shrinking the set now.

`basic` is asymmetric with `fullstack` today: `basic` has `Dioxus.toml` and a
project-root `tailwind.css` (the Tailwind v4 pipeline input, per
`build-adico-component-ecosystem` task 4.8a), while `fullstack` has neither.
`tests/playwright/README.md` still says to `dx serve --platform web` from
`examples/fullstack` directly.

## Goals / Non-Goals

**Goals:**

- Reduce `examples/` to exactly two CLI-installed fixtures with clear,
  distinct roles: `basic-spa` (web-only) and `basic-ssr` (server + web).
- Keep every remaining fixture's parity evidence accurate — no dangling
  `parity.json` paths, no doc/script referencing a deleted directory.
- Record the loss of native-desktop build evidence explicitly instead of
  quietly dropping it or fabricating replacement evidence.
- Leave `build-adico-component-ecosystem`'s open milestones referencing a
  consistent example surface.

**Non-Goals:**

- Replacing desktop validation with a new fixture. The proposal is explicit:
  two examples "for now." A future change can reintroduce desktop coverage.
- Restyling, re-theming, or changing which components `basic-spa`/`basic-ssr`
  install. Only the directory/crate identity changes for the SPA and SSR
  fixtures; their installed content (Button, Dialog, Select) is unchanged.
- Rewriting `build-adico-component-ecosystem`'s milestone scope or component
  roadmap — only its references to now-deleted example directories are
  re-pointed.

## Decisions

### 1. Rename in place rather than delete-and-recreate

`examples/basic` → `examples/basic-spa` and `examples/fullstack` →
`examples/basic-ssr` are directory/crate renames (`git mv`, plus a
`[package] name` edit in each `Cargo.toml`), not new fixtures built from
scratch. This preserves each fixture's existing `adico.lock`,
`components.json`, and installed component source as-is, so no re-install
step or new CLI run is required to keep them buildable. Only file paths, the
crate name, and every reference to the old path change.

Alternative considered: delete both and re-`adico init`/`adico add` fresh
copies under the new names. Rejected — it re-introduces CLI-install risk
(conflict handling, checksum drift) for zero behavioral benefit, since the
installed content does not change.

### 2. basic-ssr inherits basic-spa's Tailwind bootstrap gap, not a fix

`examples/basic-ssr` (renamed from `fullstack`) keeps its current lack of a
project-root `tailwind.css`/`Dioxus.toml` styling bootstrap. Closing that gap
is `build-adico-component-ecosystem` task 4.8a's tracked follow-on, not part
of this rename-and-prune change — adding it here would mix an example-surface
reduction with an unrelated styling-pipeline fix. `tests/playwright/README.md`
and `docs/adico/m2-vertical-slice.md` are updated to the new path only; their
`dx serve --platform web` instructions are otherwise unchanged.

### 3. Desktop dimension moves to an explicit named gap, not silent removal

`examples/desktop` is button's only `desktop` dimension evidence in
`parity.json` (`passed: true`). Dialog and select already record
`desktop: passed: false` with no evidence. Deleting the fixture without
updating `parity.json` would leave button's record claiming a passing
validation against a nonexistent path. `cargo xtask parity` is planned (per
`build-adico-component-ecosystem/design.md` §9) to fail CI on malformed or
incomplete records, but `packages/adico-xtask/src/main.rs` does not implement
a `parity` subcommand today, so nothing currently enforces this
automatically — the correction has to be made by hand, verified by directly
inspecting `parity.json`. Button's `desktop` entry is therefore flipped to
`passed: false` with a note: fixture removed by this change, native
desktop-target validation is an explicit tracked gap.

Alternative considered: keep `examples/desktop` and shrink only the four
placeholder stubs, landing at three examples instead of two. Rejected —
the user's request is explicit ("only two"; "basic-spa, basic-ssr"), and
desktop validation is out of scope for what those two names cover.

### 4. Refresh script stays scoped to basic-spa; basic-ssr keeps no refresh path

`scripts/refresh-basic-example.sh` currently hardcodes `cd examples/basic`
and installs `button dialog select`. It is updated in place to point at
`examples/basic-spa` and keeps its current name — only the target directory
changes, so this stays a rename, not a new script.

`examples/basic-ssr` (renamed from `fullstack`) is not given a refresh path
by this change. Per Decision 2, `basic-ssr` still lacks the project-root
`Dioxus.toml`/`tailwind.css` bootstrap that `adico init` writes for `basic`;
running the installer against it today would newly introduce that bootstrap,
which is task 4.8a's tracked follow-on, not this change's scope. `fullstack`
already had no refresh script before this change, so this is a documented
pre-existing gap carried forward under the new name, not a regression.

Alternative considered: generalize the script to accept a directory argument
so one script could refresh both fixtures. Rejected for now — doing so would
require deciding whether running it against `basic-ssr` should also write
the Tailwind bootstrap, which is exactly the scope this change defers to
4.8a. Revisit once 4.8a lands.

### 5. Amend the open ecosystem change's planning documents, not its specs

`build-adico-component-ecosystem` is an in-progress, unarchived change whose
`tasks.md` (items 4.7, 6.4, 8.5, 9.2, 9.4, 9.5, 10.4, 11.3) and `design.md`
§10 name `kitchen-sink`, `dashboard`, `forms`, and `web` as future validation
surfaces for components not yet migrated. Those are planning documents, not
specs — this change edits them directly (re-pointing at `basic-spa`,
`basic-ssr`, and `tests/installation/*`) rather than filing a spec delta,
since no `openspec/specs/*` capability currently governs example-directory
membership. `adico-example-fixtures` (this change's new capability) becomes
that governing spec going forward.

## Risks / Trade-offs

- [Native desktop build regresses silently after removal] → `parity.json`
  records the gap explicitly (Decision 3); no future change may mark
  `desktop: passed: true` without new fixture evidence.
- [`build-adico-component-ecosystem` milestones reference deleted paths after
  this change lands] → Task in this change's tasks.md amends that change's
  `tasks.md`/`design.md` §10 in the same PR, not as a follow-up.
- [`Cargo.lock` drifts from the renamed/removed workspace members, breaking
  `--locked` CI builds] → Tasks include regenerating the lockfile and running
  `cargo check --workspace --locked` before completion.
- [Renamed crate name breaks an external reference this change's grep missed]
  → Tasks include a final repository-wide grep for both old crate names
  (`adico-example-basic`, `adico-example-fullstack`, `adico-example-desktop`,
  `adico-example-dashboard`, `adico-example-forms`, `adico-example-kitchen-sink`,
  `adico-example-web`) and old paths after the rename, before marking the
  change complete.

## Migration Plan

1. Rename `examples/basic` → `examples/basic-spa` and `examples/fullstack` →
   `examples/basic-ssr` (directory + crate name); update workspace members.
2. Delete `examples/dashboard`, `examples/desktop`, `examples/forms`,
   `examples/kitchen-sink`, `examples/web` and their workspace-member entries.
3. Update `parity.json` evidence paths and button's `desktop` dimension.
4. Update `scripts/refresh-basic-example.sh`, `tests/playwright/README.md`,
   `docs/adico/m2-vertical-slice.md`, `docs/validation.md`.
5. Amend `build-adico-component-ecosystem`'s `tasks.md` and `design.md` §10.
6. Regenerate `Cargo.lock`; run `cargo check --workspace --locked` and the
   affected Playwright suite.

No rollback beyond `git revert`; nothing here touches deployed/runtime state.
