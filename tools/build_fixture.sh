#!/usr/bin/env bash
set -euo pipefail
task_target_dir="${1:-target/fixture}"
export RUSTFLAGS="-C target-feature=-bulk-memory,-reference-types,-multivalue,-simd128 -C link-arg=--no-entry -C link-arg=--export-memory -C link-arg=--max-memory=33554432 --remap-path-prefix=$(pwd)=/arex"
cargo +1.95.0 build --locked --release --target wasm32v1-none --target-dir "$task_target_dir" -p arex-fixture
python tools/normalize_wasm.py "$task_target_dir/wasm32v1-none/release/arex_fixture.wasm"
[[ "$(wasm-opt --version)" == 'wasm-opt version 133 (version_133)' ]]
task_module="$task_target_dir/wasm32v1-none/release/arex_fixture.wasm"
# Rust's unused allocation/panic formatters retain an initialized table. Binaryen
# removes that dead code; the unmodified host structural/native gates remain final.
wasm-opt "$task_module" --mvp-features -Oz --strip-debug -o "$task_module.optimized"
mv "$task_module.optimized" "$task_module"
