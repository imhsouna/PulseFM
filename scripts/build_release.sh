#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-pc-windows-msvc" "aarch64-pc-windows-msvc" "x86_64-unknown-linux-gnu")
APP_NAME="pulse-fm-rds-encoder"
DIST_DIR="${ROOT_DIR}/dist"

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
note() { printf "• %s\n" "$*"; }

mkdir -p "$DIST_DIR"

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

package_target() {
  local target="$1"
  local target_dir="${DIST_DIR}/${target}"
  rm -rf "$target_dir"
  mkdir -p "$target_dir"

  case "$target" in
    *apple-darwin)
      local bin="target/${target}/release/${APP_NAME}"
      if [[ -f "$bin" ]]; then
        cp "$bin" "$target_dir/"
        if [[ -x "${ROOT_DIR}/scripts/macos_bundle.sh" ]]; then
          "${ROOT_DIR}/scripts/macos_bundle.sh" "$target"
          cp -R "${ROOT_DIR}/dist/PulseFM.app" "$target_dir/" 2>/dev/null || true
        fi
        (cd "$target_dir" && zip -r "${APP_NAME}-${target}.zip" "${APP_NAME}" PulseFM.app >/dev/null 2>&1 || \
          (cd "$target_dir" && zip -r "${APP_NAME}-${target}.zip" "${APP_NAME}"))
      fi
      ;;
    *windows-msvc)
      local bin="target/${target}/release/${APP_NAME}.exe"
      if [[ -f "$bin" ]]; then
        cp "$bin" "$target_dir/"
        (cd "$target_dir" && zip -r "${APP_NAME}-${target}.zip" "${APP_NAME}.exe" >/dev/null 2>&1 || \
          (cd "$target_dir" && zip -r "${APP_NAME}-${target}.zip" "${APP_NAME}.exe"))
      fi
      ;;
    *unknown-linux-gnu)
      local bin="target/${target}/release/${APP_NAME}"
      if [[ -f "$bin" ]]; then
        cp "$bin" "$target_dir/"
        (cd "$target_dir" && tar -czf "${APP_NAME}-${target}.tar.gz" "${APP_NAME}")
      fi
      ;;
  esac
}

declare -a BUILT_TARGETS=()

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

  package_target "${target}"
  BUILT_TARGETS+=("${target}")
done

bold "Build complete"
note "Artifacts organized in ${DIST_DIR}"
for t in "${BUILT_TARGETS[@]}"; do
  note "${t}:"
  ls -1 "${DIST_DIR}/${t}" | sed 's/^/  - /'
done
