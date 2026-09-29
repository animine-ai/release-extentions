#!/usr/bin/env bash
set -euo pipefail
task_target_dir="${1:-target/fixture}"
export RUSTFLAGS="-C target-feature=-bulk-memory,-reference-types,-multivalue,-simd128 -C link-arg=--no-entry -C link-arg=--export-memory -C link-arg=--max-memory=33554432 --remap-path-prefix=$(pwd)=/arex"
cargo +1.95.0 build --locked --release --target wasm32v1-none --target-dir "$task_target_dir" -p arex-fixture
python tools/normalize_wasm.py "$task_target_dir/wasm32v1-none/release/arex_fixture.wasm"
