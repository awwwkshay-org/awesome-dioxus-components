## 1. Rename the surviving fixtures

- [x] 1.1 `git mv examples/basic examples/basic-spa`; rename its `Cargo.toml`
  `[package] name` to `adico-example-basic-spa`, `Dioxus.toml`
  `[application] name` to `adico-basic-spa`, `Dioxus.toml` `[web.app] title`
  to `"adico basic-spa example"`, and the `h1` text in `src/main.rs` from
  `"adico basic example"` to `"adico basic-spa example"`; verify `cargo
  metadata` lists the renamed crate and no `examples/basic` path or bare
  `"adico basic"` string remains.
- [x] 1.2 `git mv examples/fullstack examples/basic-ssr`; rename its
  `Cargo.toml` `[package] name` to `adico-example-basic-ssr` and the `h1`
  text in `src/main.rs` from `"adico fullstack example"` to `"adico
  basic-ssr example"` (this fixture has no `Dioxus.toml` to rename — see
  design.md Decision 2/4); verify `cargo metadata` lists the renamed crate
  and no `examples/fullstack` path or bare `"adico fullstack"` string
  remains.
- [x] 1.3 Update root `Cargo.toml` `[workspace].members` to
  `examples/basic-spa` and `examples/basic-ssr`; verify `cargo check
  --workspace` resolves both members.

## 2. Delete the unwired and desktop fixtures

- [x] 2.1 Delete `examples/dashboard`, `examples/forms`,
  `examples/kitchen-sink`, `examples/web` and their `[workspace].members`
  entries in root `Cargo.toml`; verify `cargo metadata` no longer lists
  `adico-example-dashboard`, `adico-example-forms`,
  `adico-example-kitchen-sink`, or `adico-example-web`.
- [x] 2.2 Delete `examples/desktop` and its `[workspace].members` entry;
  verify `cargo metadata` no longer lists `adico-example-desktop`.

## 3. Update parity evidence

- [x] 3.1 In `parity.json`, update button's `examples` dimension evidence to
  `["examples/basic-spa", "examples/basic-ssr"]` (drop `examples/desktop`);
  update dialog's and select's `examples` dimension evidence to
  `["examples/basic-spa", "examples/basic-ssr"]`; verify every remaining
  evidence string under an `examples` dimension names one of the two
  surviving directories.
- [x] 3.2 Update button's `web` and `ssrHydration` evidence and dialog's and
  select's `ssrHydration` evidence from `examples/basic`/`examples/fullstack`
  wording to `examples/basic-spa`/`examples/basic-ssr`; verify no dimension
  evidence string in `parity.json` still contains the old path text.
- [x] 3.3 Flip button's `desktop` dimension to `passed: false` with a note
  recording that `examples/desktop` was removed by this change and native
  desktop-target validation is a named, tracked gap (`cargo xtask parity`
  has no `parity` subcommand implemented today, so this is not
  CI-enforced yet); verify by inspecting `parity.json` directly that no
  `desktop` dimension across any component still has `passed: true` with
  `examples/desktop` evidence.

## 4. Update scripts and docs

- [x] 4.1 Update `scripts/refresh-basic-example.sh` to `cd examples/basic-spa`
  (keep the script scoped to the SPA fixture per design.md Decision 4; do not
  point it at `examples/basic-ssr`); verify it successfully refreshes
  `examples/basic-spa`.
- [x] 4.2 Update `tests/playwright/README.md` and the comment in
  `tests/playwright/fullstack.spec.ts` from `examples/fullstack` to
  `examples/basic-ssr`; verify the Playwright fullstack spec still passes
  when served from the renamed directory.
- [x] 4.3 Update `docs/adico/m2-vertical-slice.md` (including its embedded
  command block: `cargo check -p adico-example-fullstack ...` →
  `adico-example-basic-ssr`, `cd examples/fullstack` → `cd examples/basic-ssr`,
  `cargo build -p adico-example-desktop` line removed with a note) and
  `docs/validation.md` references from `examples/basic`/`examples/fullstack`/
  `examples/desktop` and `adico-example-basic`/`adico-example-fullstack`/
  `adico-example-desktop` to the renamed/removed fixtures, including the
  desktop-gap note; verify no remaining reference to a deleted example path
  or a removed crate name.
- [x] 4.4 Review `examples/README.md`; its current description stays
  accurate generically, so update it only if it names specific fixtures by
  the old directory names, otherwise leave unchanged. (Reviewed — the file
  is fixture-name-agnostic; left unchanged.)

## 5. Amend the open ecosystem change

- [x] 5.1 Update `openspec/changes/build-adico-component-ecosystem/tasks.md`
  items 4.7, 6.4, 8.5, 9.2, 9.4, 9.5, 10.4, and 11.3 to reference
  `examples/basic-spa`, `examples/basic-ssr`, and `tests/installation/*`
  instead of `kitchen-sink`, `dashboard`, `forms`, and `web`; verify no
  unchecked task in that file still names a deleted example directory.
- [x] 5.2 Update `openspec/changes/build-adico-component-ecosystem/design.md`
  §10 ("Examples, testing, and rollout") to describe the two-fixture example
  surface instead of the progressive web/desktop/fullstack/forms/dashboard
  rollout; verify the section is internally consistent with this change's
  design.md.

## 6. Workspace and CI validation

- [x] 6.1 Regenerate `Cargo.lock` for the reduced/renamed workspace member
  set; verify `cargo check --workspace --locked` succeeds. (Renaming/removing
  workspace members did not change the resolved dependency graph, so
  `Cargo.lock` was already consistent — confirmed via `cargo check
  --workspace --locked`, which fails on any lockfile drift.)
- [x] 6.2 Run a repository-wide grep for every old crate name — both removed
  (`adico-example-dashboard`, `adico-example-desktop`, `adico-example-forms`,
  `adico-example-kitchen-sink`, `adico-example-web`) and renamed
  (`adico-example-basic` without a `-spa`/`-ssr` suffix, i.e.
  `grep -rn 'adico-example-basic\b'`, and `adico-example-fullstack`) — and
  the old paths (`grep -rn 'examples/basic\b'` to exclude
  `examples/basic-spa`, plus `examples/fullstack`, `examples/dashboard`,
  `examples/desktop`, `examples/forms`, `examples/kitchen-sink`,
  `examples/web`); and the old bare display/title strings (`"adico basic"`,
  `"adico fullstack"`) inside `examples/basic-spa` and `examples/basic-ssr`
  themselves; verify zero remaining matches outside `Cargo.lock` history/git
  metadata. (Zero matches outside this change's own planning docs, which
  describe the migration by name as expected.)
- [x] 6.3 Build `examples/basic-spa` for `wasm32-unknown-unknown` with
  default features, and `examples/basic-ssr` with `--no-default-features
  --features server` and separately `--no-default-features --features web
  --target wasm32-unknown-unknown`; verify all three builds succeed. (All
  three passed; additionally ran the renamed `tests/playwright/fullstack.spec.ts`
  live against a real `dx serve --platform web` instance of `examples/basic-ssr`
  — passed.)
- [x] 6.4 Run `openspec validate consolidate-examples --strict`; verify it
  reports no errors before this change is marked ready to apply. (Passed;
  also re-validated `build-adico-component-ecosystem --strict` after amending
  its tasks.md/design.md — passed.)
