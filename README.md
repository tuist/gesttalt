# Gesttalt

This scaffold builds native iOS and Android shells that share product logic through Rust.

- iOS uses Swift and links the Rust library as an `XCFramework`.
- Android uses Kotlin and loads the Rust library through the Java Native Interface, documented at https://docs.oracle.com/javase/8/docs/technotes/guides/jni/.
- Once wraps the build scripts so their inputs and outputs are explicit and cacheable.
- mise installs the command line tools, including the Once executable.

## Tooling

Install the project tools:

```sh
mise install
```

Build everything:

```sh
mise run build
```

Build one side:

```sh
mise run build:ios
mise run build:android
```

The Android build needs the Android Software Development Kit, documented at https://developer.android.com/studio, and the Android Native Development Kit, documented at https://developer.android.com/ndk. Set `ANDROID_HOME` or `ANDROID_SDK_ROOT`, and set `ANDROID_NDK_HOME` or install an Android Native Development Kit under `$ANDROID_HOME/ndk`.

## Layout

- `shared/` contains the Rust crate used by both apps.
- `ios/` contains the Swift iOS application.
- `android/` contains the Kotlin Android application.
- `scripts/` contains Once-annotated build scripts.

The `once.toml` file is reserved for Once workspace configuration. Build inputs and outputs live in the script headers.

