# Building Shellow

Shellow has native SwiftUI and Jetpack Compose applications backed by a shared
Rust core. A build therefore has two stages: compile the Rust library for the
target platform, then build the native application that links it.

## Repository layout

```text
apps/ios                 SwiftUI application
apps/android             Jetpack Compose application
crates/shellow-core      SSH, terminal state, Codex, and rendering core
crates/shellow-ffi       C ABI and JNI-facing library
crates/libghostty-vt-sys Vendored mobile build integration for libghostty-vt
scripts                  Platform Rust build scripts
```

The generated `apps/ios/Frameworks/ShellowCore.xcframework` and Android
`apps/android/app/src/main/jniLibs/` contents are intentionally ignored by Git.
Re-run the appropriate Rust build script after changes to the Rust crates or when
starting from a clean clone.

## Common prerequisites

- Git
- Rust 1.96 or newer, installed through [rustup](https://rustup.rs/)
- Zig 0.15.2
- Internet access for the initial Cargo, Gradle, and Ghostty dependency downloads

On macOS, Zig can be installed with Homebrew:

```sh
brew install zig@0.15
```

The build scripts automatically prefer Homebrew's `zig@0.15` binary. They also
reuse `work/zig-global-cache` and any previously fetched Ghostty source. To use a
separate Ghostty checkout, set `GHOSTTY_SOURCE_DIR` to a directory containing its
`build.zig` file.

Clone the repository:

```sh
git clone https://github.com/ZingLix/Shellow.git
cd Shellow
```

## iOS

### Requirements

- macOS
- Xcode with the iOS 17 SDK or newer
- Xcode command-line tools
- Rust targets for iOS devices and Apple Silicon simulators

Install the Rust targets:

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
```

### Build the shared Rust framework

From the repository root, run:

```sh
./scripts/build-ios-rust.sh
```

The script builds release-mode static libraries with native integrations enabled
for both targets, then combines them into:

```text
apps/ios/Frameworks/ShellowCore.xcframework
```

The generated simulator slice is ARM64 only. Use an Apple Silicon simulator, or
extend the build script with an x86_64 simulator target when building on an Intel
Mac.

### Build and run the app

Open the project in Xcode:

```sh
open apps/ios/Shellow.xcodeproj
```

Select the `Shellow` scheme and an iOS 17 or newer simulator or device, then run
the app. You can also build from the command line:

```sh
xcodebuild \
  -project apps/ios/Shellow.xcodeproj \
  -scheme Shellow \
  -destination 'platform=iOS Simulator,name=iPhone 17' \
  build
```

Running on a physical device requires selecting your development team and a
valid signing configuration in Xcode.

### Run on a physical iPhone or iPad

The Rust framework created by `build-ios-rust.sh` already includes the
`aarch64-apple-ios` device slice. No separate Rust build is needed for a physical
device.

1. Connect the device to the Mac with a cable, or enable wireless connection in
   Xcode after pairing it once.
2. On the device, enable Developer Mode under **Settings > Privacy & Security >
   Developer Mode**. The device restarts when this is enabled.
3. Open `apps/ios/Shellow.xcodeproj` and select the `Shellow` target.
4. Under **Signing & Capabilities**, select your Apple Developer team. Xcode can
   manage the development certificate and provisioning profile automatically.
5. If `xyz.zinglix.shellow` is not available to your team, choose a unique bundle
   identifier for your local build, such as `com.example.shellow`.
6. Select the connected iPhone or iPad as the run destination and press **Run**.

The first launch may require trusting the developer certificate on the device.
For a free Apple ID, development provisioning is temporary and has additional
device and capability limits. A paid Apple Developer Program membership is
required for TestFlight and App Store distribution.

A command-line device build is also possible after signing has been configured
in Xcode:

```sh
xcodebuild \
  -project apps/ios/Shellow.xcodeproj \
  -scheme Shellow \
  -configuration Debug \
  -destination 'generic/platform=iOS' \
  -allowProvisioningUpdates \
  DEVELOPMENT_TEAM='<your-team-id>' \
  build
```

This command builds for a generic device; using Xcode is the simplest way to
install and launch it on a specific connected device.

### Build an iOS release archive

First build `ShellowCore.xcframework`, configure the Apple Developer team and
bundle identifier in Xcode, and set an appropriate version and build number.
Then create a Release archive from **Product > Archive**. When the archive
finishes, Xcode Organizer can validate it, distribute an Ad Hoc build, or upload
it to App Store Connect and TestFlight.

The equivalent archive command is:

```sh
./scripts/build-ios-rust.sh

xcodebuild archive \
  -project apps/ios/Shellow.xcodeproj \
  -scheme Shellow \
  -configuration Release \
  -destination 'generic/platform=iOS' \
  -archivePath "$PWD/build/Shellow.xcarchive" \
  -allowProvisioningUpdates \
  DEVELOPMENT_TEAM='<your-team-id>' \
  CODE_SIGN_STYLE=Automatic \
  CURRENT_PROJECT_VERSION='<build-number>' \
  MARKETING_VERSION='<version>'
```

The archive is written to `build/Shellow.xcarchive`. Exporting or uploading it
requires a distribution certificate and an App Store Connect or Ad Hoc
provisioning profile. The repository's `Release Mobile` GitHub Actions workflow
automates the App Store Connect archive and upload when its signing secrets are
configured.

## Android

### Requirements

- Android SDK 36
- Android NDK 27.1.12297006
- CMake 3.22.1
- JDK 17
- Rust targets for ARM64 devices and x86_64 emulators

Android Studio can install the SDK, NDK, and CMake components from its SDK
Manager. Install the Rust targets with:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
```

The Rust build script locates the Android SDK using `ANDROID_HOME` or
`ANDROID_SDK_ROOT`, falling back to `~/Library/Android/sdk`. It expects the NDK at
`ndk/27.1.12297006` unless `ANDROID_NDK_HOME` is set explicitly:

```sh
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.1.12297006"
```

Linux hosts are supported. The script detects common x86_64 and ARM64 NDK host
toolchains; `ANDROID_NDK_PREBUILT_HOST_TAG` can override the detected tag when
using a nonstandard NDK layout.

### Build the shared Rust libraries

From the repository root, run:

```sh
./scripts/build-android-rust.sh
```

This produces `libshellow_ffi.so` for both supported Android ABIs and installs
them at:

```text
apps/android/app/src/main/jniLibs/arm64-v8a/libshellow_ffi.so
apps/android/app/src/main/jniLibs/x86_64/libshellow_ffi.so
```

### Build the debug APK

```sh
cd apps/android
./gradlew :app:assembleDebug
```

The APK is written to:

```text
apps/android/app/build/outputs/apk/debug/app-debug.apk
```

Debug builds use the application ID `xyz.zinglix.shellow.debug` and the label
`Shellow Debug`, allowing them to coexist with a store-installed release.

From the repository root, install the APK with ADB:

```sh
adb install -r apps/android/app/build/outputs/apk/debug/app-debug.apk
```

### Build an Android release

Android release signing is configured entirely through environment variables:

```sh
export ANDROID_KEYSTORE_FILE='/absolute/path/to/upload-key.jks'
export ANDROID_KEYSTORE_PASSWORD='<keystore-password>'
export ANDROID_KEY_ALIAS='<key-alias>'
export ANDROID_KEY_PASSWORD='<key-password>'
export ANDROID_VERSION_CODE='1'
export ANDROID_VERSION_NAME='0.1.0'
```

`ANDROID_KEY_PASSWORD` falls back to `ANDROID_KEYSTORE_PASSWORD` when both
passwords are the same. `ANDROID_VERSION_CODE` must increase for every Google
Play upload.

Build the Rust libraries first, then produce the signed release artifacts:

```sh
./scripts/build-android-rust.sh
cd apps/android
./gradlew :app:assembleRelease :app:bundleRelease
```

The outputs are:

```text
apps/android/app/build/outputs/apk/release/app-release.apk
apps/android/app/build/outputs/bundle/release/app-release.aab
```

Use the APK for direct installation and the AAB for Google Play. Verify the APK
before distributing it:

```sh
$ANDROID_HOME/build-tools/36.0.0/apksigner verify --verbose \
  apps/android/app/build/outputs/apk/release/app-release.apk
```

If build-tools 36 is installed under a different patch version, adjust the
`apksigner` path. Without all required signing variables, Gradle has no release
signing configuration; do not distribute that output. The `Release Mobile`
GitHub Actions workflow builds the AAB and uploads it to the selected Google Play
track when the repository release secrets are configured.

## Checks and tests

Run Rust formatting, compile checks, and unit tests from the repository root:

```sh
cargo fmt --check
cargo check --workspace
cargo test --workspace
```

Android JVM tests and Kotlin compilation can be checked independently:

```sh
cd apps/android
./gradlew :app:testDebugUnitTest :app:compileDebugKotlin
```

Platform builds exercise the `native-integrations` feature and cross-compilation
paths that a host-only Cargo check does not cover. Before submitting a
platform-specific change, build the Rust library and native app for that platform.

## Troubleshooting

### Rust reports that a target is not installed

Install the target named in the error with `rustup target add`. The complete
target commands for iOS and Android are listed above.

### Zig is missing or has the wrong version

Confirm that Zig 0.15.2 is visible:

```sh
zig version
```

The scripts find a keg-only Homebrew `zig@0.15` automatically. For other
installations, put the correct `zig` binary on `PATH`.

### The Android build cannot find NDK Clang

Verify that NDK 27.1.12297006 is installed and that `ANDROID_NDK_HOME` points to
its root. The expected compiler is under:

```text
$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/<host-tag>/bin/
```

If the NDK uses a different prebuilt directory name, set
`ANDROID_NDK_PREBUILT_HOST_TAG` to that name.

### Xcode cannot find `ShellowCore.xcframework`

Run `./scripts/build-ios-rust.sh` before opening or building the Xcode project.
The framework is generated locally and is not included in a clean clone.

### Xcode reports an x86_64 linker error

The standard iOS script produces an ARM64 simulator slice. Select an Apple
Silicon simulator destination instead of an x86_64 destination.

### Android reports a missing `libshellow_ffi.so`

Run `./scripts/build-android-rust.sh` before the Gradle build and verify that both
files listed in the Android output section exist.
