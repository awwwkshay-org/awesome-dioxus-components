# Repository scripts

Keep non-Cargo helper entry points here. Repeatable Rust automation belongs in
`packages/adico-xtask` — including the primitive/component compatibility
tracking (`cargo xtask primitive-compat`/`component-compat`, writing to
`statics/`), which lives there rather than here since it's Rust, not a
non-Cargo helper.
