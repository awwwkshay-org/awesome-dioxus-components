# Project guide

This repository is no longer a generic full-stack Todo template. It is the
implementation workspace for `adico`: source-owned Dioxus components,
registry tooling, and the `adico` CLI.

Use the active OpenSpec change as the implementation guide. In particular, do
not bypass the delivery sequence by adding missing shadcn components before the
owned primitive, registry, and CLI vertical slice is working.

Company registries are a first-class design requirement: a consumer may choose
its organization registry as the default while still explicitly installing an
official `@adico/...` item. See [`architecture.md`](architecture.md) and the
[OpenSpec design](../openspec/changes/build-adico-component-ecosystem/design.md).
