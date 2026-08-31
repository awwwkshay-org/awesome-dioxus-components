# statics/

Generated tracking snapshots -- not shippable package or registry content.
Regenerate, don't hand-edit:

- `primitive_compatibility.json` -- `cargo xtask primitive-compat sync|check|diff`
- `component_compatibility.json` -- `cargo xtask component-compat sync|check`

See `packages/adico-xtask/src/primitive_compat.rs` and
`packages/adico-xtask/src/component_compat.rs` for what each command derives
versus hand-maintains.
