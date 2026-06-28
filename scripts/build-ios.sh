#!/bin/bash
# once input "../ios/**/*"
# once input "../shared/include/**/*"
# once output "../build/ios/app/"
# once env "PATH"
# once env "DEVELOPER_DIR"
# once cwd ".."
set -euo pipefail

source scripts/common.sh

require_tool xcodebuild

if [[ ! -d ios/Vendor/SharedRust.xcframework ]]; then
  echo "Missing ios/Vendor/SharedRust.xcframework. Run mise run build:shared:ios first." >&2
  exit 1
fi

rm -rf build/ios/app build/ios/DerivedData
mkdir -p build/ios/app

xcodebuild \
  -project ios/Gesttalt.xcodeproj \
  -scheme Gesttalt \
  -configuration Debug \
  -sdk iphonesimulator \
  -destination 'generic/platform=iOS Simulator' \
  -derivedDataPath build/ios/DerivedData \
  CODE_SIGNING_ALLOWED=NO \
  build

cp -R build/ios/DerivedData/Build/Products/Debug-iphonesimulator/Gesttalt.app build/ios/app/
