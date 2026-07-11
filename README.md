# Shellow

Shellow is a native mobile SSH terminal client prototype. The product direction is iOS and Android apps with a shared Rust core for SSH sessions, terminal state, and GPU-backed terminal rendering.

## Current Milestone

- Native iOS SwiftUI shell in `apps/ios`.
- Native Android Jetpack Compose shell in `apps/android`.
- Shared Rust core in `crates/shellow-core`, exported through `shellow-ffi`.
- `russh`, `libghostty-vt`, and `wgpu` compile into the Apple and Android Rust slices. VT parsing/state is now routed directly through the official `libghostty-vt` Rust crate (`libghostty-vt-sys` vendored Zig build) instead of the previous compatibility layer.
- `libghostty-vt-sys` is patched locally in `crates/libghostty-vt-sys` for mobile vendored builds: iOS device/simulator target mapping, arm64 simulator CPU selection, and Android lib-vt font-stack pruning.
- Profiles combine a saved host with a default Terminal or Codex workspace, open directly from the Profiles screen, and expose host-scoped session switching after connection.
- Codex profiles connect over SSH to a durable remote `codex app-server` daemon: Shellow starts the already-bootstrapped daemon idempotently and proxies JSON-RPC over the current SSH channel, so closing the mobile transport no longer owns the daemon lifetime. Run `codex app-server daemon bootstrap --remote-control` once on each remote host before the first Codex connection; Shellow does not silently change that persistent security setting.
- `shellow renderer` routes through the shared Rust renderer lifecycle API, reports terminal frame metrics/dirty rows/signatures, initializes a persistent wgpu device/queue on first frame, uploads a glyph atlas texture plus dirty-row buffer data, reuses the device on later frames, and submits native terminal-frame passes: Metal on iOS, Vulkan on Android. The atlas now has an explicit raster backend boundary: native builds prefer `fontdue-system-font-rasterizer` by loading `SHELLOW_RENDERER_FONT_PATH` or a platform monospace system font, fall back to `procedural-cell-rasterizer` when no parseable font is available, and keep `font-shaping-glyph-atlas` as the target backend. Native builds now shape text with `rustybuzz-terminal-shaper`, store shaped font glyph IDs in the shared atlas, rasterize those IDs with `fontdue::rasterize_indexed`, and fall back to `terminal-cell-cluster-layout` only when shaping is unavailable. iOS passes the `MTKView` `CAMetalLayer` handle into Rust, which creates/configures a `wgpu::Surface`, presents the attach probe, sends selection/search ranges through `shellow_engine_set_renderer_overlay_json`, and presents visible terminal grid frames plus renderer-owned overlays through the Rust-owned surface. Android hosts a `SurfaceView`, converts its `Surface` to an `ANativeWindow` in JNI, holds that native window for Rust, drives the same viewport render-frame and renderer-overlay APIs, and keeps Compose terminal rows as transparent interaction/accessibility hit targets while visible terminal content comes from Rust/wgpu.
- Android debug APK packages both `arm64-v8a` and `x86_64` JNI/native Rust libraries.
- iOS simulator live SSH has been verified against a reachable password-auth host: `russh` opens the PTY shell, remote output is parsed by `libghostty-vt`, the native `wgpu` surface presents terminal frames, and first-use host-key pinning persists back to the profile.
- Architecture boundaries documented for the native GPU terminal surface path and remaining runtime proof work.
- Terminal capability checklist lives in `docs/TERMINAL_CAPABILITIES.md`.

## Target Architecture

```text
iOS SwiftUI / Android Compose app shell
  -> native terminal surface host
  -> C ABI bridge / JNI bridge
  -> russh session actor
  -> libghostty-vt VT state
  -> shared persistent wgpu terminal renderer lifecycle API
  -> fontdue/system-font glyph atlas / font-shaping atlas target
  -> rustybuzz terminal shaper / terminal-cell fallback layout
  -> shared renderer overlay cell-range API
  -> native wgpu terminal surface
```

## Build

The native build products are intentionally git-ignored: iOS regenerates
`apps/ios/Frameworks/ShellowCore.xcframework`, and Android regenerates
`apps/android/app/src/main/jniLibs/`. Run the Rust build script for the platform
before opening/building the mobile shell from a clean clone.

### iOS

Open `apps/ios/Shellow.xcodeproj`, or build from this folder with:

```sh
./scripts/build-ios-rust.sh
xcodebuild -project apps/ios/Shellow.xcodeproj -scheme Shellow -destination 'platform=iOS Simulator,name=iPhone 17' build
```

### Android

Build the Android Rust shared libraries, then build the debug APK:

```sh
./scripts/build-android-rust.sh
cd apps/android
./gradlew :app:assembleDebug
```

Ghostty's vendored build requires Zig 0.15.2. The Rust build scripts automatically prefer Homebrew's `zig@0.15` when available, reuse `work/zig-global-cache`, reuse any already-fetched Ghostty source through `GHOSTTY_SOURCE_DIR`, and use `ReleaseFast` for the vendored `libghostty-vt` build. If Zig is missing:

```sh
brew install zig@0.15
```

The APK is written to:

```text
apps/android/app/build/outputs/apk/debug/app-debug.apk
```

Store release CI setup lives in `docs/STORE_RELEASE_SETUP.md`; Chinese setup
notes live in `docs/STORE_RELEASE_SETUP.zh-CN.md`.

Run on a connected ADB device:

```sh
adb connect <host>:<port>
android run --device=<host>:<port> --apks=apps/android/app/build/outputs/apk/debug/app-debug.apk --activity=xyz.zinglix.shellow.MainActivity
```

Inside the demo terminal, try:

```sh
shellow integrations
shellow ghostty
shellow ssh
shellow renderer
```

For live SSH, add or tap a password-based host profile, enter a password, optionally provide a startup command, then connect. Terminal input is routed through the live `russh` PTY and remote bytes are rendered through `libghostty-vt` before the native viewport displays them.

## License

Shellow is licensed under the Apache License, Version 2.0. Third-party
dependencies and bundled assets retain their own licenses.
