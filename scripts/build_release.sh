#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-pc-windows-msvc" "aarch64-pc-windows-msvc" "x86_64-unknown-linux-gnu")

for target in "${TARGETS[@]}"; do
  if rustup target list | grep -q "^${target} (installed)"; then
    echo "[build] ${target}"
    cargo build --release --target "${target}"
  else
    echo "[skip] ${target} (not installed)"
  fi
done

echo "Done. Artifacts are in target/<triple>/release/"
