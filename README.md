# Gesttalt

This scaffold builds native iOS and Android shells that share product logic through Rust.

- iOS uses Swift and links the Rust library through the Once build graph.
- Android uses Kotlin and packages the Rust library through the Once build graph.
- Once owns the target graph so inputs, outputs, and platform edges are explicit and cacheable.
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

The Android app task builds a pinned Once source snapshot until the next Once release includes the Android application fix needed by this target graph.

Run a target directly:

```sh
once build AppleApp
once run AppleApp
mise run once:snapshot
build/tools/once/538bab10210ce8177e31e3d2e343f38fce265f35/once build AndroidApp
```

The Android build needs the Android Software Development Kit, documented at https://developer.android.com/studio, and the Android Native Development Kit, documented at https://developer.android.com/ndk. `mise install` provisions the Android command line tools used by the Once targets.

## Layout

- `shared/` contains the Rust crate used by both apps.
- `ios/` contains the Swift iOS application.
- `android/` contains the Kotlin Android application.
- `once.toml` contains the Once target graph.
