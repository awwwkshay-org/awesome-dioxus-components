## Why

`adico-cli` cannot be published to crates.io today: the official registry is
embedded via 72 `include_bytes!("../../../registry/...")` calls in
`packages/adico-cli/src/main.rs` that reach outside the crate directory, and
`cargo package`/`cargo publish` only archive files inside it. Separately,
resolving any HTTPS or local registry today eagerly checksums every file of
every item before dependency resolution starts
(`RegistrySourceLoader::validate`, `packages/adico-registry-core/src/lib.rs`)
— against an HTTPS registry the size of `@adico`, that is one GET per file in
the entire catalog just to run `adico list`, not the one-fetch-per-requested-
item cost a shadcn-style registry should have. `registry/generated/` already
exists in a shape close to shadcn's `/r/<name>.json`, produced by `cargo xtask
registry build`, but nothing reads it and the CI check named "Validate
registry payloads aren't stale" does not actually check it — which already
caused a real drift incident (22 of 43 generated payloads silently missing).

This change adopts shadcn's model — each item served as a self-contained JSON
document with its file content inlined, over plain static HTTPS, with no
registry server — so the registry can be published as ordinary static files
and `adico add <item>` costs proportionally to what was requested.

## What Changes

- **BREAKING (registry format, additive)**: introduce `formatVersion: 2` for
  *generated* registry output only. A format-2 `RegistryFile` entry carries an
  inlined `content` field (the file's full UTF-8 source) alongside its
  existing `source`/`checksum`. The hand-authored `registry/registry.json`
  keeps using format 1 (`source` + `checksum` pointers) unchanged — maintainers
  never author inline content by hand. `adico-registry-core` accepts both
  format 1 and format 2 on read; anything it generates is format 2.
- Make `RegistrySourceLoader`'s validation lazy/per-item instead of eager over
  the whole registry, so resolving and installing N requested items costs
  proportional network/IO, not the size of the entire catalog.
- `cargo xtask registry build` emits two artifacts from the same
  `registry/registry.json` source of truth:
  - a served tree (index + per-item document with inlined content) — not
    committed to git; regenerated wherever the registry is actually served
  - `packages/adico-cli/embedded/registry.json`, a single committed format-2
    payload replacing all 72 `include_bytes!` arms with one, restoring
    `adico-cli`'s publishability
- The official `@adico` registry becomes HTTPS-first (fetching from a single
  named base-URL constant) with the embedded payload as an offline/network-
  failure fallback, rather than embedded-only.
- Add a CI gate that fails if `packages/adico-cli/embedded/registry.json` is
  stale relative to a fresh `cargo xtask registry build` run — closing the gap
  that let the prior drift incident happen silently.
- Fix the placeholder `$schema` URL (`https://adico.dev/...`, a domain nobody
  controls) written into every consumer's generated `components.json` to point
  at the domain that will actually serve it.
- Document that a third-party/organization registry may serve either format
  (1 or 2) — the existing static-file-mirror contract in
  `docs/adico/organization-registry.md` is not being narrowed.
- Decide and record what `adico.lock`'s `manifestDigest` means once the
  registry is an index plus independent per-item documents rather than one
  monolithic manifest blob (it is written today but never verified against).

**Explicitly not part of this change**: no registry HTTP API/server of any
kind — the registry stays static JSON, matching shadcn and this repo's own
documented "no server-side logic required" contract for organization
registries. No Docker/nginx/Kubernetes work to actually serve the output (a
separate, dependent infrastructure change). No `cargo publish` execution, no
CLI release automation, no community/license files (already done).

## Capabilities

### New Capabilities

*(none — this extends existing registry and installation behavior)*

### Modified Capabilities

- `adico-registry`: registry items support an additive `formatVersion: 2`
  inline-content file representation alongside the existing pointer-based
  format; registry build output includes a served, shadcn-style tree in
  addition to the existing manifest; registry validation cost during
  resolution is proportional to requested items, not the whole catalog.
- `adico-cli-installation`: the official registry is resolved HTTPS-first
  against a configured base URL with an embedded fallback, rather than
  embedded-only; the embedded fallback payload is a single generated file
  instead of per-file compiled-in source.

## Impact

- `packages/adico-registry-core/src/lib.rs`: `RegistryFile`, `RegistryManifest`
  validation/`formatVersion` handling, `RegistrySourceLoader::validate`/`load`,
  `RegistryHttpClient`/`StaticHttpsClient`, `https_source_root`.
- `packages/adico-cli/src/main.rs`: `ConfiguredRegistryReader` (removes the
  72-arm `include_bytes!` match), `configured_catalog`.
- `packages/adico-cli/src/init.rs`, `add.rs`: `$schema` URL, `adico.lock`
  `manifestDigest` handling.
- `packages/adico-xtask/src/main.rs`: `build_registry`, `validate_registry`.
- `registry/generated/` (no longer committed), new
  `packages/adico-cli/embedded/registry.json` (committed).
- `docs/adico/organization-registry.md`.
- `.github/workflows/ci.yml`: the registry-staleness gate.
- No changes to `registry/registry.json`'s authored format, to
  `tests/installation/awwwkshay-consumer/awwwkshay-registry/` (stays format 1
  and must keep working unmodified), or to any registry item's Rust source.
