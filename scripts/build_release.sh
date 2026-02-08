#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-pc-windows-msvc" "aarch64-pc-windows-msvc" "x86_64-unknown-linux-gnu")

is_macos() {
  [[ "$(uname -s)" == "Darwin" ]]
}

build_with_cargo() {
  local target="$1"
  echo "[build] ${target}"
  cargo build --release --target "${target}"
}

build_windows_msvc() {
  local target="$1"
  if ! command -v cargo-xwin >/dev/null 2>&1; then
    echo "[skip] ${target} (cargo-xwin not installed)"
    echo "       Install: cargo install cargo-xwin"
    return
  fi
  echo "[build] ${target} (cargo-xwin)"
  cargo xwin build --release --target "${target}"
}

build_linux_gnu() {
  local target="$1"
  local toolchain="stable-x86_64-unknown-linux-gnu"
  if command -v cross >/dev/null 2>&1; then
    if ! rustup toolchain list | grep -q "^${toolchain}"; then
      echo "[setup] ${toolchain}"
      rustup toolchain add "${toolchain}" --profile minimal --force-non-host
    fi
    echo "[build] ${target} (cross + docker)"
    DOCKER_DEFAULT_PLATFORM=linux/amd64 cross build --release --target "${target}"
    return
  fi
  echo "[skip] ${target} (cross not installed)"
  echo "       Install: cargo install cross"
  echo "       Requires: Docker Desktop running"
}

for target in "${TARGETS[@]}"; do
  if ! rustup target list | grep -q "^${target} (installed)"; then
    echo "[skip] ${target} (not installed)"
    continue
  fi

  case "${target}" in
    x86_64-apple-darwin|aarch64-apple-darwin)
      build_with_cargo "${target}"
      ;;
    x86_64-pc-windows-msvc|aarch64-pc-windows-msvc)
      if is_macos; then
        build_windows_msvc "${target}"
      else
        build_with_cargo "${target}"
      fi
      ;;
    x86_64-unknown-linux-gnu)
      if is_macos; then
        build_linux_gnu "${target}"
      else
        build_with_cargo "${target}"
      fi
      ;;
    *)
      build_with_cargo "${target}"
      ;;
  esac
done

echo "Done. Artifacts are in target/<triple>/release/"
