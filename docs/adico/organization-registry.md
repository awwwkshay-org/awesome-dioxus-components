# Authoring and switching to an organization registry

A consumer application can configure a named registry alongside (or instead
of) the official `@adico` one, and set it as the default namespace a bare
`adico add <item>` resolves against. This is the CLI-level mechanism, not a
new package or fork of `adico` itself -- the `awwwkshay-registry` fixture
under `tests/installation/awwwkshay-consumer/` is a complete, real, tested
example of everything below.

## 1. Author the registry manifest and source

An organization registry is a directory containing one `registry.json`
manifest and the `.rs` source files it references, in the same shape as this
repo's own `registry/` (see `packages/adico-registry-core/src/lib.rs`'s
`RegistryManifest`/`RegistryItem` types for the full schema):

```json
{
  "formatVersion": 1,
  "namespace": "@awwwkshay",
  "name": "Awwwkshay Dioxus Registry",
  "description": "...",
  "compatibility": { "cli": ">=0.1.0" },
  "items": [
    {
      "name": "card",
      "type": "registry:ui",
      "description": "...",
      "files": [
        {
          "source": "ui/card.rs",
          "targetRoot": "ui",
          "target": "card.rs",
          "checksum": "<sha256 of ui/card.rs>"
        }
      ],
      "registryDependencies": ["@adico/cn"],
      "cargoDependencies": [{ "crate": "dioxus", "version": "=0.7.9" }],
      "style": { "utilities": ["cn"] },
      "moduleExports": [{ "targetRoot": "ui", "module": "card", "reexport": true }],
      "documentation": { "slug": "card" }
    }
  ]
}
```

Notes specific to authoring your own registry, not the official one:

- `namespace` must start with `@` and is what consumers reference the
  registry by (`@awwwkshay/card`, or a bare `card` once it's the default --
  see step 3).
- `registryDependencies` can reference items in *another* registry with a
  fully qualified `@namespace/item` address (`@adico/cn` above) --
  organization registries are expected to build on top of the official one,
  not duplicate its utilities.
- Every file's `checksum` is a plain `sha256` hex digest of that exact
  source file's bytes (`shasum -a 256 <file>` on macOS/Linux); the CLI
  verifies it before installing, so a stale checksum after an edit fails
  installation loudly rather than silently installing drifted source.
- There is no `cargo xtask registry build`/`registry validate` equivalent
  for an external organization registry -- those are specific to this
  repo's own `@adico` registry's embedded-source build. Validate your own
  registry by installing it into a real fixture (step 4) and running
  `cargo check`/`cargo test` against the result, the same way this repo
  validates `@adico` itself against `tests/installation/*`.

## 2. Choose a location: local path or static HTTPS

`RegistryLocation` (`packages/adico-registry-core/src/lib.rs`) supports three
kinds; only the latter two are relevant to an organization registry:

- **`local`**: a filesystem path, relative to the consumer's own project
  root, to the directory containing your `registry.json`. Simplest for a
  registry that lives in the same monorepo or is vendored via a git
  submodule/subtree.
- **`https`**: a static HTTPS URL serving the same directory layout (a
  `registry.json` plus its referenced source files) -- any static file host
  works (S3/GCS bucket, GitHub Pages, a CDN), no server-side logic required.
  `manifest_url`/`source_root` are both plain URLs.

## 3. Configure and default-switch it in `components.json`

```json
{
  "registries": {
    "@adico": { "kind": "embedded" },
    "@awwwkshay": { "kind": "local", "path": "awwwkshay-registry" }
  },
  "defaultRegistry": "@awwwkshay"
}
```

For an `https` source, replace the `local` entry with
`{ "kind": "https", "url": "https://.../registry.json" }` (the CLI resolves
each item's own source files relative to that manifest URL's own directory).

Either author `components.json` by hand as above, or use `adico init`'s own
flags to generate it:

```sh
adico init \
  --registry @awwwkshay=awwwkshay-registry \
  --default-registry @awwwkshay
```

(`adico init --registry <@namespace>=<embedded|relative-path|https-url>` accepts
a relative filesystem path or an `https://` URL in place of `embedded`.)

Setting `defaultRegistry` to your own namespace means every *bare*
`adico add <item>` (no `@namespace/` prefix) resolves against it first --
you can still install an official-registry item explicitly with its
`@adico/` prefix (`adico add @adico/button`) regardless of which registry is
default, and your own registry's items can declare `@adico/...`
`registryDependencies` that resolve correctly either way (step 1).

## 4. Validate it

Configuring a registry is only proven correct once a real installation
succeeds and compiles -- create a `tests/installation/*`-shaped fixture (a
standalone `Cargo.toml` + `Dioxus.toml` + hand-authored `src/main.rs`,
matching the convention every fixture in this repo already follows), run
`adico init`/`adico add` against it for real, then `cargo check`/`cargo
test`. `tests/installation/awwwkshay-consumer` is exactly this, checked in
as a living example: it installs both `@adico/cn` (official) and
`@awwwkshay/card` (organization, which itself depends on `@adico/cn`) into
one consumer and compiles.
