## 1. shadcn `cva()` variant-group extraction

- [x] 1.1 In `packages/adico-xtask/src/catalog/shadcn.rs`, add a
      `parse_cva_variant_groups(source: &str, alias: &str) -> Vec<Prop>`
      function: find `const <alias> = cva(` in `source`, parse its
      `variants: { <group>: { <key>: <value>, ... }, ... }` object into one
      `Prop` per group (`name` = group name, `type_name` = the quoted-union
      of its keys, `default` = the matching entry in `defaultVariants`, if
      present). Verify with a unit test using a `cva`-shaped fixture (two
      variant groups, a `defaultVariants` block) alongside the existing
      `DIALOG_FIXTURE` test, asserting the exact `variant`/`size` prop list
      shown in design.md.
- [x] 1.2 Wire it into `parse_shadcn_source`: when a signature's type
      expression matches `VariantProps<typeof (\w+)>`, call
      `parse_cva_variant_groups` for that alias and merge its output into
      the part's `props` list alongside any `& { ... }` augmentation props
      already parsed (do not replace the existing augmentation parsing).
      Verify with a unit test combining `VariantProps<typeof
      buttonVariants>` and a trailing `& { asChild?: boolean }` on one
      signature, asserting the part's `props_source` is `Explicit` with
      three props (`variant`, `size`, `asChild`).
- [x] 1.3 Verify `cargo test -p adico-xtask` passes, including the existing
      `trigger_is_pure_passthrough`/`content_has_explicit_augmentation_and_composition`/
      `no_match_yields_unavailable` tests (unchanged behavior for shapes
      this task doesn't touch).

## 2. Radix reference resolution (no new axis)

- [x] 2.1 Add the component-id and part-id alias tables from design.md's
      Decisions section (as fixed `&[(...)]` constants, not computed, each
      entry verified against the committed `base-ui.json` and covered by
      its own pinned unit test naming its expected target — not just its
      existence), plus a `resolve_inherits_from(reference: &str, base_ui:
      &CatalogSnapshot) -> ResolvedProps` function that: parses
      `"<axis>.<component>.<part>"`; if `axis` has no catalog of its own
      (`vaul`, `cmdk`, `@shadcn/react/*`) or is a shadcn-internal
      self-reference (`@/registry/*`), returns "unresolved" with that
      specific reason before attempting any Base UI lookup; if `axis ==
      "radix"`, applies the component alias then the part alias
      (component-scoped aliases checked before generic ones), and for a
      known composition-judgment case (`menubar`'s non-`root` sub-parts,
      `alert-dialog.action`/`cancel`, `toggle-group.item`, or a genuine
      absence) returns "unresolved" with that case's specific reason;
      otherwise looks up the (possibly-aliased) component+part in
      `base_ui`'s entries, returning that part's real
      `PropsSource::Explicit` props on a match or a generic "unresolved"
      reason on none. If `axis` is anything else (already `"base-ui"`, or
      a future direct match), look it up directly with no aliasing. Verify
      with unit tests covering: a direct match (no alias needed), a
      component-alias match (`dropdown-menu` → `menu`), a part-alias match
      (`content` → `popup`), the `radio-group.root` → `radio.group` /
      `radio-group.item` → `radio.root` correctness fix, a third-party-axis
      reference (`vaul.*`) and a self-reference (`@/registry/*`) each
      getting their own reason, a `menubar` non-root sub-part getting its
      composition-judgment reason while `menubar.root` still resolves
      directly, and a genuine non-match (`command` — no Base UI entry)
      returning "unresolved" with a reason. **Done** — see
      `packages/adico-xtask/src/catalog/radix_aliases.rs`.
- [x] 2.2 **Done.** Verified with two permanent regression tests in
      `catalog/radix_aliases.rs` (`real_shadcn_radix_references_match_the_measured_90_19_split`,
      `real_non_radix_references_get_specific_reasons_not_a_base_ui_message`)
      that run the actual resolver over the committed
      `statics/catalogs/shadcn.json`/`base-ui.json` rather than a
      throwaway harness, so future catalog drift fails a real test.
      Against the catalog fetched in task 5.1 (revision
      `7c9eaba1c0a6404c990c144a654792e3313c650d`, refreshed 2026-09-05):
      **88 of 107** `radix.*` references resolve, 19 do not, matching
      design.md's named residual list exactly (`menubar`'s 10 non-root
      sub-parts, `alert-dialog.action`/`cancel`, `toggle-group.item`, and
      the genuine absences
      `aspect-ratio`/`slot`/`label`/`popover.anchor`/`navigation-menu.indicator`),
      and each of the 30 non-`radix` references gets its own
      third-party/self-reference reason rather than a Base UI-shaped
      message. (Pre-`cva`-fix these were 109/90/32; the `cva` fix itself
      moved 2 `radix.*` and 2 other-axis references to `explicit` — see
      design.md's Decisions section.)

## 3. Dioxus inline-component wiring fix (both axes)

- [x] 3.1 In `packages/adico-xtask/src/catalog/dioxus_shared.rs`'s shared
      `parts_from_introspection` (used by both the `dioxus-primitives` and
      the non-bespoke parts of the `dioxus-components` fetchers), fall back
      to `introspection.props.get(component)` — the inline
      function-argument-component shape `rust_introspect.rs` already
      understands — when no `{component}Props` struct exists, instead of
      leaving the part `PropsSource::Unavailable`.
      `catalog/dioxus_components.rs`'s own bespoke builder (which does not
      call the shared helper) keeps its equivalent fallback. Verify with a
      unit test in `dioxus_shared.rs` against a fixture inline component
      asserting `explicit` with its parameter list, alongside the existing
      equivalent test in `dioxus_components.rs`. **Done.**
- [x] 3.2 **Done.** `cargo test -p adico-xtask` passes (165 tests).
      Re-fetching both axes (network, revision
      `bf007c15d0cf4d04d3181cc46cf12325aa773955`) dropped `unavailable`
      well below the pre-fix baselines: `dioxus-components` **136 → 77**
      of 235 parts; `dioxus-primitives` **7 → 2** of 161 parts (same total
      part counts before/after, confirming an apples-to-apples
      comparison).

## 4. `prop-parity sync|check|diff`

- [x] 4.1 Add `packages/adico-xtask/src/prop_parity.rs` implementing the
      prop-name normalization table from design.md (casing, `className` →
      `class`, React-only structural props, Dioxus-structural props), the
      `present | missing | intentional_difference | adico_extension`
      classifier, and the reason tables from design.md's "Reasons ... live
      in `prop_parity.rs`" decision — fixed `const` tables keyed by
      `(item, part, prop)` for item-specific reasons and by `prop` alone
      for the shared structural-props reasons, looked up at generation
      time and never read back from a generated JSON file. Verify with
      unit tests: a prop present under both names after normalization, a
      `className`→`class` match, a `render` prop classified
      `intentional_difference` with its fixed reason, a genuine miss
      classified `missing`, and an `adico_extension` prop (e.g. `radius`)
      getting its reason from the item-keyed table rather than requiring
      one to be supplied by the caller.
- [x] 4.2 Implement `sync`: for each of the 66 `registry:ui` items, load
      its `FileIntrospection` (via `registry_introspect.rs` +
      `rust_introspect.rs`); map the item to each axis's matching
      component using the same kebab-case identity match
      `component_compat.rs`'s `build_catalog_axis` already uses (reusing
      its `SHADCN_EXCEPTIONS`/`DIOXUS_COMPONENT_EXCEPTIONS` tables, not a
      new mapping table), and map parts on both sides with
      `catalog/case.rs`'s `part_id_for`; join against each applicable
      axis's resolved props (using `resolve_inherits_from` from section 2
      when an axis entry is `inherits_from`); and write
      `statics/prop_parity/<item>.json` matching design.md's output shape,
      using `write_if_changed` (`main.rs`'s existing helper) rather than
      an unconditional `fs::write`, so an unchanged item produces no diff.
      An axis with no matching component for an item records
      `matchedComponent: null` with an empty `parts` array. Verify by
      running `cargo run -p adico-xtask -- prop-parity sync` and
      inspecting `statics/prop_parity/switch.json` against design.md's
      worked example.
- [x] 4.3 Implement `check`: regenerate in memory, diff against committed
      output, exit non-zero on any difference, run fully offline. Verify
      by hand-editing one `statics/prop_parity/*.json` file and confirming
      `check` fails and names the file; revert and confirm it passes; run
      `sync` twice in succession against an unchanged tree and confirm the
      second run's `write_if_changed` calls write no file (idempotence) —
      not merely that the file's content is unchanged.
- [x] 4.4 Implement `diff`: print the would-be change without writing.
      Verify by editing a component's props and confirming `diff` shows
      the classification change.
- [x] 4.5 Wire the subcommand into `packages/adico-xtask/src/main.rs`
      (dispatch arms + the usage string), following the exact
      `styling-usage`/`playground-controls` pattern. Verify `cargo run -p
      adico-xtask -- prop-parity` with no subcommand prints
      `usage: cargo xtask prop-parity sync|check|diff` and the top-level
      `cargo run -p adico-xtask` usage string lists it.

## 5. Regenerate and commit

- [x] 5.1 **Done.** Fetched all three axes (network):
      `shadcn` @ `7c9eaba1c0a6404c990c144a654792e3313c650d`,
      `dioxus-components`/`dioxus-primitives` @
      `bf007c15d0cf4d04d3181cc46cf12325aa773955`.
      `statics/catalogs/shadcn.json`'s `button` entry now includes
      `variant`/`size` alongside `asChild`, confirmed. Before/after
      `explicit`/`inherits_from`/`unavailable` part-count split (326 total
      parts unchanged, confirming apples-to-apples): shadcn **46/141/139 →
      57/137/132**. The volume driver was indeed parts other than `Button`
      moving out of `unavailable` via the native-tag-plus-`cva()` shape
      (e.g. `alert.root`), not only `Button` gaining two props — confirmed
      by `shadcn.rs`'s new
      `native_tag_passthrough_with_cva_variant_groups_is_explicit` test.
      dioxus-components: **19/80/136 → 78/80/77** of 235 (see task 3.2 for
      the `unavailable` drop; `dioxus-primitives`: 154/0/7 → 159/0/2 of
      161).
- [x] 5.2 **Done.** Ran `cargo run -p adico-xtask -- prop-parity sync`
      against the refreshed catalogs, generating all 66
      `statics/prop_parity/*.json` files (to be committed alongside this
      change). `cargo run -p adico-xtask -- prop-parity check` passes
      cleanly.

## 6. CI and docs

- [x] 6.1 **Done.** Added a "Check prop-parity records aren't stale" step
      to `.github/workflows/ci.yml` immediately after the existing
      `playground-controls check` step, matching its exact shape.
- [x] 6.2 **Done.** Documented `prop-parity sync|check|diff` in
      `docs/development.md` and added it as a required row in
      `docs/validation.md`'s validation matrix.
- [x] 6.3 **Done**, with one pre-existing, unrelated failure reported
      rather than silently skipped. `cargo fmt --all --check`: pass.
      `cargo check --locked --workspace`: pass. `cargo clippy --locked
      --workspace --all-targets -- -D warnings`: **fails**, but in
      `examples/basic-spa/src/components/ui/data_table.rs` and
      `examples/basic-ssr/src/components/ui/data_table.rs`
      (`collapsible_if`) — installed consumer copies of the `data-table`
      registry item last touched in M8 (commit `1f386a7`), untouched by
      this change and confirmed via `git status`/`git log` to predate it;
      out of scope to fix here. The narrower, CLAUDE.md/`docs/validation.md`-
      documented baseline clippy command
      (`-p adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask`) passes clean. `cargo test --locked
      -p adico-cli -p adico-primitives -p adico-registry-core -p
      adico-test-utils -p adico-xtask`: pass (zero failures across every
      crate, including all 66 `prop-parity`/`primitive-usage`/
      `styling-usage` per-item regression tests). `openspec validate
      extend-upstream-prop-evidence --strict`: pass. Also re-ran
      `primitive-compat sync`/`component-compat sync` after task 5.1's
      catalog refresh (both had gone stale relative to the refreshed
      `statics/catalogs/*.json`, an expected downstream consequence, not a
      defect) and confirmed `check` passes for both.
