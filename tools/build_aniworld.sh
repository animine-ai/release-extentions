#!/usr/bin/env bash
set -euo pipefail
task_target_dir="${1:-target/aniworld}"
export RUSTFLAGS="-C target-feature=-bulk-memory,-reference-types,-multivalue,-simd128 -C link-arg=--no-entry -C link-arg=--export-memory -C link-arg=--max-memory=33554432 --remap-path-prefix=$(pwd)=/arex"
CARGO_PROFILE_RELEASE_OPT_LEVEL=3 cargo +1.95.0 build --locked --release --target wasm32v1-none --target-dir "$task_target_dir" -p arex-aniworld
task_module="$task_target_dir/wasm32v1-none/release/arex_aniworld.wasm"
python tools/normalize_wasm.py "$task_module"
[[ "$(wasm-opt --version)" == 'wasm-opt version 133 (version_133)' ]]
# Optimize the provider for execution speed while retaining the frozen MVP host
# profile; strip Rust's unused allocation/panic formatting tables as well.
wasm-opt "$task_module" --mvp-features -O3 --strip-debug -o "$task_module.optimized"
mv "$task_module.optimized" "$task_module"
