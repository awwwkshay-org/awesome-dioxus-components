Design references (D1-D9) are defined in `design.md`'s Decisions section.
Every task below states its own verification command; run the shared
baseline (`cargo fmt --all --check`, `cargo clippy --locked -p adico-cli -p
adico-primitives -p adico-registry-core -p adico-test-utils -p adico-xtask
--all-targets -- -D warnings`, `cargo test --locked` for the same five
crates) before considering any group done, not just at the very end.

## 1. Registry-core: additive format 2 schema (D1)

- [x] 1.1 Add `SUPPORTED_REGISTRY_FORMAT_VERSIONS: &[u32] = &[1, 2]` and bump
      `REGISTRY_FORMAT_VERSION` to `2` (the version this crate *writes*);
      change `RegistryManifest::validate`'s format check from an exact match
      to `SUPPORTED_REGISTRY_FORMAT_VERSIONS.contains(&self.format_version)`.
      Verify: existing format-1 fixtures in
      `packages/adico-registry-core/src/lib.rs`'s test module and
      `tests/compile/registry/*.json` still parse and validate; add one new
      test asserting format 2 also validates and format 3 (unsupported) is
      rejected with the existing `UnsupportedFormat` error naming both the
      actual and supported versions.
- [x] 1.2 Add `content: Option<String>` to `RegistryFile` with
      `#[serde(default, skip_serializing_if = "Option::is_none")]`. Verify:
      round-trip (de)serialization tests for both a format-1 file entry (no
      `content` key present at all in the JSON, `content` deserializes to
      `None`) and a format-2 entry (`content` present, deserializes to
      `Some`), asserting the format-1 case's serialized output contains no
      `content` key at all (not `"content": null`).
- [x] 1.3 Update `registry/schema.json` to declare `content` as an optional
      string property on the file definition, alongside the existing
      `additionalProperties: false`/`required` fields. Verify:
      `openspec validate` and any JSON-schema-consuming tooling still accept
      both an authored format-1 item (no `content`) and a hand-written
      format-2 fixture (with `content`).

## 2. Registry-core: proportional resolution cost (D2, D3)

- [x] 2.1 Extract the per-item, per-file content-checksum loop out of
      `RegistrySourceLoader::validate` into a new public
      `validate_all_content(&self, registry: &LoadedRegistry) ->
      Result<(), RegistryError>`, leaving `validate` with only the
      structural checks (format-version support, compatibility,
      `registryDependencies` resolvability, duplicate-target detection,
      checksum-format validation, cycle detection). `load` continues to call
      `validate` only. Verify: existing checksum-mismatch tests move to
      exercise `validate_all_content` directly and still pass; a new test
      confirms `load()` against an HTTPS source with one deliberately
      wrong-checksummed *unrequested* file still succeeds (proving `load`
      itself no longer touches file bytes).
- [x] 2.2 Change `RegistryFileReader::read` (`packages/adico-cli/src/add.rs`)
      and `RegistrySourceLoader::read_resolved_source`/
      `read_source_location` to accept the resolved `&RegistryFile` (not
      `source: &str`); when `file.content` is `Some`, return its bytes with
      no I/O; otherwise fall back to the existing local-disk/HTTPS-GET/
      embedded-unsupported behavior unchanged. Update the two call sites
      (`add.rs`'s `plan_source_install`, `ConfiguredRegistryReader::read`) to
      pass the `RegistryFile` already in scope. Verify:
      `cargo test -p adico-registry-core` and
      `cargo test -p adico-cli` pass; a new test confirms reading a
      format-2 file with inline content triggers zero calls against a
      call-counting `RegistryHttpClient` test double.
- [x] 2.3 Add a call-counting `RegistryHttpClient` test double (if one does
      not already exist in test helpers) and write an integration-style test
      against `RegistrySourceLoader`/`RegistryCatalog` asserting: (a)
      resolving/listing a registry with N items issues exactly one HTTP GET
      (the index/manifest), regardless of N; (b) `adico add <item>` against
      that same registry issues exactly one additional GET per resolved item
      (the requested item plus its transitive `registryDependencies`), not
      one per file and not one per catalog item. This test directly proves
      the new `adico-registry` requirement "Registry resolution cost is
      proportional to requested items."

- [x] 2.4 (found during 2.1's implementation, not originally enumerated)
      `cargo xtask registry validate`'s `load_registry_manifest` stopped
      exhaustively checksumming file content once the eager loop moved out
      of `validate()` -- confirmed by deliberately corrupting a checksum in
      `registry/registry.json` and observing `registry validate` wrongly
      pass. Fixed by having `load_registry_manifest` call
      `validate_all_content` explicitly after `load`, exactly as design D3
      already assumed xtask would. Verify: same manual corruption now fails
      with a checksum-mismatch message; a pinned regression test exists at
      `packages/adico-xtask/src/main.rs`'s
      `registry_validate_still_catches_a_tampered_checksum`, using a new
      dedicated fixture at `tests/compile/registry/checksum-mismatch-source/`.

## 3. Registry-core: per-item content fetch (D4)

- [x] 3.1 Add a method to `RegistrySourceLoader` that, given an already-
      resolved `RegistryItem`/its name, returns a content-bearing version of
      that item: if the item's files already carry `content` (already
      resident, e.g. embedded or a format-2 local registry), return it
      unchanged with no fetch; otherwise, for HTTPS/Local sources, fetch
      `<source_root>/<item-name>.json` (reusing `https_source_root`/
      `source_root.join(...)` exactly as file resolution already does),
      parse it as a `RegistryItem`, and use its content-bearing files.
      Verify: a unit test against each of the three `RegistryLocation`
      variants (Embedded returns unchanged with zero calls; Local reads
      `<dir>/<item-name>.json` from disk; Https issues exactly one GET to
      the expected joined URL).
- [x] 3.2 Wire `add.rs`'s `plan_source_install` to call this method once per
      resolved item, immediately before reading that item's files, replacing
      direct access to `resolved.item.files` where those files might lack
      content. Verify: `cargo test -p adico-cli` passes, including
      `cli_integration.rs`'s existing local-registry and HTTPS-registry
      installation tests.

## 4. adico-xtask: served tree and embedded payload generation (D4, D8)

- [x] 4.1 Delete the bespoke `GeneratedRegistryIndex` struct
      (`packages/adico-xtask/src/main.rs:61-68`); change `build_registry` to
      write the manifest itself (format 2, every item's `files[].content`
      set to `None`) as `registry/generated/index.json`, and one
      content-bearing `RegistryItem` (format 2, `files[].content` populated
      from the authored source file's actual bytes) per item at
      `registry/generated/<item-name>.json` — directly under the served
      root, not nested under `items/`. Verify:
      `cargo run -p adico-xtask -- registry build` succeeds and
      `registry/generated/index.json`'s items match `registry/registry.json`
      1:1 with no `content` keys; a sampled `registry/generated/button.json`
      contains `button.rs`'s exact byte-for-byte source as its file's
      `content`.
- [x] 4.2 Add `registry/generated/` to `.gitignore` and `git rm -r --cached
      registry/generated` (it is currently tracked). Verify:
      `git status` shows no `registry/generated/*` files after a fresh
      `cargo run -p adico-xtask -- registry build`.
- [x] 4.3 Add generation of `packages/adico-cli/embedded/registry.json` to
      `build_registry` (or a new xtask subcommand) — a single format-2
      manifest with every item's `files[].content` populated from the
      authored source, replacing the previous 72 `include_bytes!` payload.
      This file **is** committed. Verify: `git diff
      packages/adico-cli/embedded/registry.json` after a clean rebuild shows
      no unexpected changes; the file parses as a valid format-2
      `RegistryManifest` whose every item has every file's `content`
      present.
- [x] 4.4 Add a drift-checking mode (`--check` flag on `registry build`, or
      reuse `registry validate`) that fails if regenerating
      `packages/adico-cli/embedded/registry.json` produces a diff from the
      committed copy. Wire this into `.github/workflows/ci.yml` as a
      required step. Verify: deliberately edit a `registry/ui/*.rs` file
      without regenerating the embedded payload, confirm the check fails
      naming the stale file; regenerate and confirm it passes.

## 5. adico-cli: remove the 72-arm embed and wire HTTPS-first resolution (D4, D6, D7)

- [x] 5.1 Replace `ConfiguredRegistryReader`'s 72-arm `include_bytes!` match
      (`packages/adico-cli/src/main.rs` ~525-737) and its
      `(RegistryLocation::Embedded {..}, _) => Err(...)` catch-all with a
      single `include_bytes!("../embedded/registry.json")` loaded via
      `EmbeddedRegistry::new`/`LoadedRegistry::from_embedded_manifest` as
      today, and a read path that looks up the resolved item's matching
      `RegistryFile.content` directly (per task 2.2 — no more matching on
      `source` strings). Verify: `cargo build -p adico-cli` succeeds with
      zero `include_bytes!` occurrences for `registry/ui/*` or
      `registry/lib/*` remaining in `main.rs` (`grep -c
      'include_bytes!("../../../registry' packages/adico-cli/src/main.rs`
      returns `0`); `cargo package --list -p adico-cli` no longer needs
      `registry/` files (task 6.1 confirms this fully).
- [x] 5.2 Add `pub const OFFICIAL_REGISTRY_URL: &str =
      "https://adico.awwwkshay.com/r/index.json";` as a single named
      constant. Change `configured_catalog`'s handling of the namespace
      equal to `RegistryNamespace::OFFICIAL` to attempt
      `loader.load(&official_ns, &RegistrySource::Https { url:
      OFFICIAL_REGISTRY_URL.into() })` first, falling back to
      `LoadedRegistry::from_embedded_manifest` on a
      `RegistryError::NetworkRequest`. Do not change behavior for any other
      namespace or for `@adico` explicitly configured as
      `{"kind": "embedded"}` (that path continues to skip the network
      entirely, unchanged). Verify: a test using a `RegistryHttpClient` test
      double that always errors confirms `adico add @adico/button` still
      succeeds via the embedded fallback; a test double that succeeds
      confirms the network path is preferred when available.
- [x] 5.3 Change `adico init`'s default `@adico` registry entry (when not
      overridden by `--registry`) from `{"kind": "embedded"}` to
      `{"kind": "https", "url": OFFICIAL_REGISTRY_URL}`. Verify:
      `cli_integration.rs`'s fresh-init test asserts the new default; a
      project that already ran `init` before this change and has a
      committed `components.json` is not silently rewritten (confirm `init`
      remains non-destructive on an existing config, per the existing
      "Initialization prepares a supported project" requirement).
- [x] 5.4 Make the CLI's reviewable plan/result state whether the official
      registry was resolved from the network or the offline fallback (new
      `adico-cli-installation` requirement, "Offline resolution reports its
      source"). Verify: a new `cli_integration.rs` test asserts the reported
      source differs between the two `RegistryHttpClient` test-double
      scenarios from 5.2.
- [x] 5.5 Change the `$schema` literal in `packages/adico-cli/src/init.rs:260`
      and `add.rs:684` from `https://adico.dev/schema/components.json/v1` to
      `https://adico.awwwkshay.com/schema/components.json/v1`. Verify:
      `cli_integration.rs`'s existing assertions on generated
      `components.json` content are updated and pass.

## 6. Publishability verification (closes the original blocker)

- [x] 6.1 Run `cargo package --list -p adico-cli` and confirm it lists no
      path outside `packages/adico-cli/`. This is the concrete proof the
      original blocker (B1) is resolved — `cargo check`/`cargo build` do not
      catch this class of failure. Verified: `cargo package --list -p
      adico-cli --allow-dirty` lists exactly `Cargo.toml`, `README.md`,
      `embedded/registry.json`, `src/*.rs`, `tests/cli_integration.rs` --
      zero paths outside the crate, and `embedded/registry.json` is present.
      A full `cargo package`/`--verify` build cannot succeed yet: cargo's
      isolated verify build resolves `adico-registry-core` (a path+version
      dependency, now required by cargo's own packaging rules) against real
      crates.io in a temp workspace disconnected from this repo's root
      `Cargo.toml`/`[patch.crates-io]`, and that crate genuinely isn't
      published. This is a real, structural prerequisite of the separate
      crates.io-publishing workstream (Workstream C), not a defect in this
      change -- `--list` (which doesn't need dependency resolution) is the
      achievable and sufficient proof that B1 (paths escaping the crate
      directory) is fixed.
- [x] 6.2 Add `packages/adico-cli/embedded/` to the crate's packaged file
      set if not automatically included (check `Cargo.toml`'s `include`/
      `exclude`, if any). Verify: `cargo package --list -p adico-cli`
      includes `embedded/registry.json`.

## 7. Documentation and fixtures (D9)

- [x] 7.1 Add a short section to `docs/adico/organization-registry.md`
      stating that a third-party/organization registry may serve either
      format 1 (pointer-based, today's documented static-file-mirror
      contract) or format 2 (content-bearing per-item documents, this
      change's shape) — both are read identically by the installer. Verify:
      the doc no longer implies format 1 is the only supported shape.
- [x] 7.2 Confirm `tests/installation/awwwkshay-consumer/` needs no content
      changes (it stays format 1) and still passes end-to-end: `adico init
      --default-registry @awwwkshay --registry @awwwkshay=awwwkshay-registry
      && adico add card @adico/button && cargo build` inside that fixture.
      Verify: the fixture's existing integration test in
      `cli_integration.rs` passes unmodified.
- [x] 7.3 Update `CLAUDE.md`'s `adico-xtask` command list and
      `docs/development.md`'s registry-generation section to describe the
      two generated artifacts (served tree, not committed; embedded
      payload, committed) and the new drift-check step. Verify: both docs
      accurately describe the commands that exist after this change (no
      stale claims about `registry/generated/` being committed).

## 8. Full validation sweep

- [x] 8.1 Run the complete baseline: `cargo fmt --all --check`, `cargo check
      --locked --workspace`, `cargo clippy --locked -p adico-cli -p
      adico-primitives -p adico-registry-core -p adico-test-utils -p
      adico-xtask --all-targets -- -D warnings`, `cargo test --locked` for
      the same five crates, `cargo run -p adico-xtask -- registry validate`,
      the new drift-check from 4.4, and `openspec validate --all --strict`.
      Verify: all pass with zero warnings/failures.
- [x] 8.2 Rebuild and reinstall the existing consumer fixtures
      (`examples/basic-spa`, `examples/basic-ssr`,
      `tests/installation/awwwkshay-consumer`) against the locally built
      `adico` binary and confirm `cargo build` succeeds for each — proving
      the format-1 and now-format-2 official registry paths both still
      produce installable, compilable output.
