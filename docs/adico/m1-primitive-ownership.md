# M1 primitive ownership acceptance record

Status: accepted for implementation

## Audited upstream baseline

- Upstream: `DioxusLabs/dioxus-components`
- Immutable revision: `bf007c15d0cf4d04d3181cc46cf12325aa773955`
- Styled component inventory: 45 items
- Primitive source inventory: 65 Rust/JS/TS source units
- Checked-in snapshot: [`../../upstreams/dioxus-components/catalog.json`](../../upstreams/dioxus-components/catalog.json)
- Human classification/dependency audit: [`../../upstreams/dioxus-components/inventory.md`](../../upstreams/dioxus-components/inventory.md)

`cargo xtask upstream dioxus-components` validates and reports the checked-in
snapshot without network access. A maintainer refreshes from an explicitly
supplied local clone:

```sh
cargo xtask upstream dioxus-components \
  --source /path/to/dioxus-components \
  --refreshed-at YYYY-MM-DD
```

The command only reports the diff until `--write` is supplied.

## Owned initial primitive closure

The first vertical slice remains **Button, Dialog, and Select**. Button needs
no headless primitive in the initial slice. Dialog and Select now use the owned
`adico-primitives` closure:

| Public facade | Retained internal support |
| --- | --- |
| `dialog` | IDs, controlled state, Escape/outside-dismiss handling, focus-trap asset and adapter |
| `select` | collection, listbox, selectable, selection, typeahead, Select component submodules |

All 17 imported source units have immutable revision headers and the record at
[`../../provenance/records/adico-primitives-dialog-select.json`](../../provenance/records/adico-primitives-dialog-select.json).
The upstream MIT and Apache-2.0 license texts are retained under
[`../../third_party/dioxus-components/`](../../third_party/dioxus-components/).

## Independent verification

| Check | Result |
| --- | --- |
| `cargo xtask provenance check` | 1 imported record and 17 source units validated |
| `cargo test -p adico-primitives --locked` | 14 inherited unit tests, 2 public-facade compile tests, and 14 doctests passed |
| `cargo test -p adico-test-utils --locked` | shared Select typeahead keyboard helper passed |
| `cargo check -p adico-primitives --locked --no-default-features` | SSR-safe default build passed |
| `cargo check -p adico-primitives --locked --features desktop` | desktop-capability compile passed |
| `cargo check -p adico-primitives --locked --no-default-features --features web --target wasm32-unknown-unknown` | WebAssembly web-capability compile passed |

The web/desktop capabilities isolate DOM evaluation and focus trapping. The
default feature set uses SSR-safe adapters. Browser interaction behavior remains
an M2 vertical-slice requirement and is not yet claimed as complete parity.
