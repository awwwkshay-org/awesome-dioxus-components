# CI and delivery

The current CI workflow validates Rust formatting, the workspace, Clippy, and
tests. It does not build the retired Todo API/UI container images.

As the registry and CLI arrive, CI will add the checks defined in
[`validation.md`](validation.md): registry/provenance validation, CLI
installation fixtures, Dioxus web and SSR/hydration checks, Playwright
accessibility coverage, visual regression, and explicit organization-registry
fixtures. Network-dependent upstream synchronization remains an explicit
maintainer action; ordinary CI uses checked-in snapshots.

Publishing CLI binaries, primitive crates, docs applications, and registry
artifacts is deferred until the associated M2/M3 release work is implemented.
