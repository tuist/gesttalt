#!/bin/bash
# once input "../scripts/common.sh"
# once input "../shared/Cargo.toml"
# once input "../shared/src/**/*.rs"
# once input "../shared/include/**/*"
# once output "../build/ios/shared/"
# once output "../ios/Vendor/SharedRust.xcframework/"
# once env "PATH"
# once env "CARGO_HOME"
# once env "RUSTUP_HOME"
# once env "DEVELOPER_DIR"
# once cwd ".."
set -euo pipefail

source scripts/common.sh

require_tool cargo
require_tool lipo
require_tool xcodebuild

rm -rf build/ios/shared ios/Vendor/SharedRust.xcframework
mkdir -p build/ios/shared ios/Vendor

ios_device_target="aarch64-apple-ios"
ios_simulator_targets=("aarch64-apple-ios-sim" "x86_64-apple-ios")

ensure_rust_target "$ios_device_target"
cargo build --manifest-path shared/Cargo.toml --target "$ios_device_target" --release

simulator_libraries=()
for target in "${ios_simulator_targets[@]}"; do
  ensure_rust_target "$target"
  cargo build --manifest-path shared/Cargo.toml --target "$target" --release
  simulator_libraries+=("target/$target/release/libshared.a")
done

lipo -create "${simulator_libraries[@]}" -output build/ios/shared/libshared-ios-simulator.a

xcodebuild -create-xcframework \
  -library "target/$ios_device_target/release/libshared.a" \
  -headers shared/include \
  -library build/ios/shared/libshared-ios-simulator.a \
  -headers shared/include \
  -output ios/Vendor/SharedRust.xcframework
