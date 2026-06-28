#!/bin/bash
# once input "../android/**/*"
# once output "../build/android/app/"
# once output "../android/app/build/outputs/"
# once env "PATH"
# once env "ANDROID_HOME"
# once env "ANDROID_SDK_ROOT"
# once cwd ".."
set -euo pipefail

source scripts/common.sh

require_tool gradle

if [[ ! -d android/app/src/main/jniLibs ]]; then
  echo "Missing Android native libraries. Run mise run build:shared:android first." >&2
  exit 1
fi

rm -rf build/android/app
mkdir -p build/android/app

gradle -p android assembleDebug

cp android/app/build/outputs/apk/debug/app-debug.apk build/android/app/Gesttalt-debug.apk
