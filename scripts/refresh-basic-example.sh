#!/usr/bin/env sh
set -eu

# Rebuild the local installer, then refresh this consumer-style example only
# through its public commands. Installed UI source remains committed so the
# example compiles without invoking the script.
cargo build -p adico-cli --locked
cd examples/basic
../../target/debug/adico init
../../target/debug/adico add button dialog select
