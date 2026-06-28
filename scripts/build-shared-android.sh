#!/bin/bash
# once input "../scripts/common.sh"
# once input "../shared/Cargo.toml"
# once input "../shared/src/**/*.rs"
# once output "../build/android/shared/"
# once output "../android/app/src/main/jniLibs/"
# once env "PATH"
# once env "ANDROID_API_LEVEL"
# once env "ANDROID_HOME"
# once env "ANDROID_SDK_ROOT"
# once env "ANDROID_NDK_HOME"
# once env "ANDROID_NDK_ROOT"
# once env "CARGO_HOME"
# once env "RUSTUP_HOME"
# once cwd ".."
set -euo pipefail

source scripts/common.sh

require_tool cargo

api_level="${ANDROID_API_LEVEL:-24}"
ndk_home="$(find_android_ndk)"
host_tag="$(android_host_tag "$ndk_home")"
toolchain_bin="$ndk_home/toolchains/llvm/prebuilt/$host_tag/bin"

rm -rf build/android/shared android/app/src/main/jniLibs
mkdir -p build/android/shared android/app/src/main/jniLibs

android_targets=(
  "aarch64-linux-android arm64-v8a aarch64-linux-android"
  "armv7-linux-androideabi armeabi-v7a armv7a-linux-androideabi"
  "i686-linux-android x86 i686-linux-android"
  "x86_64-linux-android x86_64 x86_64-linux-android"
)

for entry in "${android_targets[@]}"; do
  read -r rust_target android_abi linker_prefix <<<"$entry"
  linker="$toolchain_bin/${linker_prefix}${api_level}-clang"
  if [[ ! -x "$linker" ]]; then
    echo "Missing Android linker: $linker" >&2
    exit 1
  fi

  ensure_rust_target "$rust_target"
  export "$(cargo_linker_env_name "$rust_target")=$linker"
  cargo build --manifest-path shared/Cargo.toml --target "$rust_target" --release

  mkdir -p "android/app/src/main/jniLibs/$android_abi"
  cp "target/$rust_target/release/libshared.so" "android/app/src/main/jniLibs/$android_abi/libshared.so"
done

cp -R android/app/src/main/jniLibs build/android/shared/jniLibs
