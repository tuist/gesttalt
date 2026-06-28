#!/bin/bash
set -euo pipefail

require_tool() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "Missing required tool: $name" >&2
    exit 1
  fi
}

ensure_rust_target() {
  local target="$1"
  if command -v rustup >/dev/null 2>&1; then
    rustup target add "$target" >/dev/null
  fi
}

find_android_ndk() {
  local candidate
  for candidate in "${ANDROID_NDK_HOME:-}" "${ANDROID_NDK_ROOT:-}"; do
    if [[ -d "$candidate/toolchains/llvm/prebuilt" ]]; then
      echo "$candidate"
      return 0
    fi
  done

  local sdk_root
  local ndk_dir
  for sdk_root in "${ANDROID_HOME:-}" "${ANDROID_SDK_ROOT:-}"; do
    if [[ ! -d "$sdk_root/ndk" ]]; then
      continue
    fi
    for ndk_dir in "$sdk_root"/ndk/*; do
      if [[ -d "$ndk_dir/toolchains/llvm/prebuilt" ]]; then
        candidate="$ndk_dir"
      fi
    done
  done

  if [[ -n "${candidate:-}" ]]; then
    echo "$candidate"
    return 0
  fi

  echo "Android Native Development Kit not found. Set ANDROID_NDK_HOME or install it under ANDROID_HOME/ndk." >&2
  return 1
}

android_host_tag() {
  local ndk_home="$1"
  local tag
  for tag in darwin-arm64 darwin-x86_64 linux-x86_64 linux-aarch64; do
    if [[ -d "$ndk_home/toolchains/llvm/prebuilt/$tag/bin" ]]; then
      echo "$tag"
      return 0
    fi
  done

  echo "No supported Android Native Development Kit host toolchain found in $ndk_home." >&2
  return 1
}

cargo_linker_env_name() {
  local target="$1"
  local upper
  upper="$(printf '%s' "$target" | tr '[:lower:]-' '[:upper:]_')"
  printf 'CARGO_TARGET_%s_LINKER' "$upper"
}
