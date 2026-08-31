# Repository scripts

Keep non-Cargo helper entry points here. Repeatable Rust automation belongs in
`packages/adico-xtask` — including the Base UI/shadcn compatibility tracking
(`cargo xtask baseui-compat`/`shadcn-compat`), which lives there rather than
here since it's Rust, not a non-Cargo helper.
