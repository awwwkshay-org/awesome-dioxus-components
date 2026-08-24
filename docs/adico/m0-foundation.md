# M0 foundation decision record

Status: accepted for implementation

## Current workspace audit

The repository currently contains the full-stack Todo template described by
`docs/architecture.md`:

| Current path | Current responsibility | M0 disposition |
| --- | --- | --- |
| `apps/awesome-dioxus-components-api` | Axum, SQLx, PostgreSQL Todo API | Retired and removed: it is not an adico runtime boundary. |
| `apps/awesome-dioxus-components-ui` | Dioxus Todo UI, SSR BFF, web/native features | Retired and removed: `apps/docs`, `apps/playground`, and consumer examples replace its role. |
| `packages/shared` | Todo HTTP contracts | Unreferenced legacy package retained pending a separately authorized removal. |
| `docker-compose.yml`, legacy delivery/docs, and Todo OpenSpec specs | Todo-template support material | Retained pending a separately authorized removal or archival decision; they are not part of adico's supported runtime. |

The target adico packages and applications are the supported product surface in
M0. The two executable Todo-template applications were retired by an explicit
project decision because they do not participate in the registry/CLI vertical
slice. Remaining historical template material is isolated and must not acquire
new adico dependencies.

## M0 workspace boundaries

| Boundary | Responsibility | Explicit exclusions |
| --- | --- | --- |
| `packages/adico-cli` | The `adico` executable and user-facing commands | Registry semantics and primitive behavior. |
| `packages/adico-registry-core` | Registry schema, source resolution, validation, plans | File-system UX and CLI rendering. |
| `packages/adico-primitives` | Public owned headless runtime behavior | Styled registry components and installer logic. |
| `packages/adico-test-utils` | Reusable test fixtures/helpers | Production runtime behavior. |
| `packages/adico-xtask` | Repository automation and validation entry points | Consumer-facing installation behavior. |
| `registry/` | Authored, distributable source and metadata | A Cargo styled-component crate. |
| `apps/docs`, `apps/playground` | Maintained adico applications | External-consumer installation coverage. |
| `examples/` and `tests/installation/` | Consumer-style validation fixtures | Direct workspace imports of registry UI source. |

## Initial architectural constraints

- `adico-primitives` is the only adico runtime package installed component
  source may depend on for shared behavior.
- `adico-registry-core` owns source/namespace resolution for the embedded
  official registry, configured local registries, and static HTTPS registries.
- Registry components are authored under `registry/` and are never exposed as a
  styled component crate consumed by applications.
- No supported adico package depends on the retired API/UI template or on its
  shared contract package.
- Workspace-wide validation covers the supported adico applications, examples,
  and packages; remaining historical files are not runtime dependencies.

## M0 acceptance evidence

This record is accepted when the additive workspace skeleton, baseline test
matrix, provenance policy, and parity schema listed in the OpenSpec M0 tasks
exist and the workspace formatting check succeeds.

| Evidence | Artifact |
| --- | --- |
| Workspace audit and boundary decision | This record and the root `Cargo.toml` workspace members |
| Package/app/example skeletons | `packages/adico-*`, `apps/docs`, `apps/playground`, and `examples/*` |
| Validation matrix and suite boundaries | [`../validation.md`](../validation.md) and `tests/` |
| Toolchain compatibility decision | [`m0-toolchain-decisions.md`](m0-toolchain-decisions.md) |
| Dual-license and provenance policy | [`../../UPSTREAMS.md`](../../UPSTREAMS.md) and `provenance/` |
| Offline parity source of truth | [`../../parity.json`](../../parity.json) and [`parity.md`](parity.md) |
