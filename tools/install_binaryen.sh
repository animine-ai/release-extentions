#!/usr/bin/env bash
set -euo pipefail
task_tool_dir="${1:?provide installation directory}"
mkdir -p "$task_tool_dir"
task_archive="$task_tool_dir/binaryen.tar.gz"
curl --fail --location --retry 3 --output "$task_archive" https://github.com/WebAssembly/binaryen/releases/download/version_133/binaryen-version_133-x86_64-linux.tar.gz
printf '%s  %s\n' 2dc9c7813f5375db93d96ead4b78222fcc3e2677bbb832297af4797782a37489 "$task_archive" | sha256sum --check --strict
tar -xzf "$task_archive" --strip-components=1 -C "$task_tool_dir"
"$task_tool_dir/bin/wasm-opt" --version
