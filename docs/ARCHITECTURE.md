# Shellow Architecture

## Product Goal

Shellow is a mobile SSH terminal client. The app ships as native iOS and Android shells around a shared Rust runtime.

## Foundation Requirements

- Native iOS shell: SwiftUI for navigation, UIKit bridge for the future terminal surface.
- Native Android shell: Jetpack Compose for navigation and JNI for the Rust bridge.
- Secure host profiles: iOS Keychain and Android Keystore for private key material and secrets.
- SSH core: Rust `russh`, wrapped in a session actor owned by the Rust core.
- Terminal engine: Shellow's `TerminalEngine` boundary now uses the official `libghostty-vt` Rust crate directly. The crate builds and links Ghostty's VT library through `libghostty-vt-sys` and Zig 0.15.2, and `ghostty_adapter` translates Ghostty terminal state, modes, styled cells, cursor state, scrollback, title, bell, and OSC 52 effects into Shellow's shared terminal snapshot model. Shellow carries a small local `libghostty-vt-sys` patch for mobile vendored builds until those target mappings are upstreamed.
- Renderer: custom terminal-grid renderer over `wgpu`, with a glyph-atlas raster backend boundary, a glyph layout/shaping backend boundary, dirty-row updates, renderer-owned selection/search overlays, and a native surface attach boundary. The shared Rust core now owns a persistent terminal renderer runtime that consumes the current grid snapshot plus overlay cell ranges, reports dirty rows/cell metrics/frame signatures/overlay counts/glyph-atlas backend status/glyph-layout backend status, initializes one native `wgpu` device/queue on first frame, uploads a glyph atlas texture plus dirty-row buffer data, reuses the device on later frames, accepts iOS `CoreAnimationLayer` and Android native-window surface descriptors through FFI/JNI, creates/configures real `wgpu::Surface` targets for supported native builds, and presents visible terminal grid frames through the Rust-owned surface while SwiftUI/Compose remain responsible for hit testing and accessibility. Native builds now prefer a real `fontdue-system-font-rasterizer` atlas from `SHELLOW_RENDERER_FONT_PATH` or platform monospace system fonts, with procedural cells retained only as a no-font fallback. Native builds shape terminal text with `rustybuzz-terminal-shaper`, map HarfBuzz clusters back to terminal cell ranges, store `FontGlyph(u16)` atlas keys, and rasterize shaped glyph IDs with `fontdue::rasterize_indexed`; non-native or no-font builds keep `terminal-cell-cluster-layout` as the fallback.
- FFI: thin C ABI for the current shared bridge; Swift consumes it directly, Android consumes it through JNI. UniFFI remains an option for broader control-plane APIs later.

## iOS v1 Scope

- Run as a native SwiftUI app on iPhone simulator.
- Show a terminal-first app shell with profiles and settings.
- Use a Rust-built `ShellowCore.xcframework` for session snapshots and demo commands.
- Compile `russh`, `libghostty-vt`, and `wgpu` into the iOS Rust slice.
- Avoid committing to platform-only rendering paths that would be thrown away later.

## Android v1 Scope

- Run as a native Jetpack Compose app from `apps/android`.
- Reuse the same Rust `shellow-ffi` ABI through a small `shellow_jni` wrapper.
- Package Android Rust slices as `libshellow_ffi.so` for `arm64-v8a` and `x86_64`.
- Use `wgpu` with the Vulkan backend for Android renderer probes.
- Keep the app shell behavior aligned with the iOS prototype: terminal, hosts, settings, demo commands, and password-shell connection flow.

## Rust Boundary

The Rust side will eventually expose:

- Session lifecycle: create, connect, disconnect, resize, send input, clear visible terminal output, and reset local terminal display state.
- Profile-safe inputs: host, port, username, auth reference, known-host policy.
- Terminal events: dirty rows, cursor state, title changes, bell, connection state.
- Renderer lifecycle: create persistent renderer, inspect renderer info, produce frames, attach native surface, resize, draw frame, detach.

Swift/Kotlin own presentation, platform credential storage, and lifecycle. Rust owns SSH, terminal correctness, and terminal rendering.

## Current Integration State

- `shellow-core`: owns the terminal snapshot model, integration report, `ghostty_adapter` VT backend boundary, persistent terminal renderer runtime, terminal render-frame model, demo command handling, and `RusshSessionActor`.
- `shellow-ffi`: exposes a C ABI consumed by Swift through `ShellowCore.xcframework` and by Android through `shellow_jni`, including `shellow_engine_render_frame_viewport_json`, `shellow_engine_renderer_info_json`, `shellow_engine_set_renderer_overlay_json`, `shellow_engine_attach_core_animation_layer_json`, `shellow_engine_attach_android_native_window_json`, and `shellow_engine_detach_renderer_surface_json` for the shared renderer lifecycle boundary.
- `russh`: compiled into the native iOS slice; `RusshSessionActor` supports one-shot PTY exec plus a long-lived password shell actor for interactive input/output.
- `libghostty-vt`: first-class terminal backend in the integration report, Cargo feature surface, and Shellow VT adapter boundary. `official-libghostty-vt-rs` is enabled by default and by `native-integrations`; the older `ghostty-vt`, `libghostty-vt`, and `libghostty-vt-link` feature names are aliases to the same official Rust crate path for command compatibility. The integration report now identifies the contract as `libghostty-vt-rs-0.2.0` and reports the vendored Zig sys build state. The local sys patch adds iOS target mapping, the arm64 iOS simulator `apple_a17` CPU selection needed by Zig 0.15's simdutf build, and an Android lib-vt-only font backend setting so Android does not pull Ghostty's unrelated fontconfig/harfbuzz discovery stack.
- `wgpu`: compiled into native Apple and Android slices; `shellow renderer` now routes through the shared Rust renderer lifecycle API, creates one native GPU device/queue on the first frame, uploads a glyph atlas texture with `write_texture`, packs dirty terminal rows into a GPU buffer with `write_buffer`, reuses the device on later frames, consumes renderer overlay cell ranges for selection/search/active-search highlights, reports the active glyph atlas backend (`fontdue-system-font-rasterizer` when a platform font is parseable, otherwise `procedural-cell-rasterizer`) separately from the target backend (`font-shaping-glyph-atlas`), reports the active glyph layout backend (`rustybuzz-terminal-shaper` when shaping is available, otherwise `terminal-cell-cluster-layout`) separately from the same shaping target, creates/configures an iOS `wgpu::Surface` from the `CoreAnimationLayer`, creates/configures an Android Vulkan `wgpu::Surface` from an `ANativeWindow` on Android native builds, presents attach probes, renders visible terminal cell/glyph/overlay vertices into the Rust-owned native surface, and keeps offscreen terminal-frame render passes available on Metal/Vulkan.
- iOS terminal viewport: hosts an `MTKView`/`CAMetalLayer` surface as the primary grid drawing layer, passes the layer pointer and drawable size into Rust's renderer surface attach ABI, sends viewport-relative selection/search ranges into Rust through `shellow_engine_set_renderer_overlay_json`, and asks Rust to present the current terminal viewport through `shellow_engine_render_frame_viewport_json`. The transparent SwiftUI layer remains responsible for row selection, drag selection, mouse-reporting hit targets, accessibility, and a hidden `UIKeyInput` responder for terminal-area direct soft/hardware keyboard input.
- iOS terminal controls: SwiftUI exposes terminal-viewport direct input, copy/search/paste, clear terminal, reset terminal, jump-to-bottom, auto-follow for new output while search is inactive, shared readline-style local prompt editing including cursor movement, middle insertion/deletion, `Ctrl-A`/`Ctrl-E`/`Ctrl-K`/`Ctrl-U`/`Ctrl-W`, `Ctrl-R` reverse history search, and the special-key toolbar including nano/tmux control keys `^B`, `^O`, and `^X`; the former visible staging input field has been removed from the primary terminal path, and clear/reset call the same Rust C ABI as Android rather than mutating Swift-only state.
- iOS transcript export: SwiftUI writes the current visible terminal text to UTF-8 `.txt` files in the app Documents `Shellow-Transcripts` directory using the same snapshot-derived text as Copy Terminal.
- iOS display settings: stored in `UserDefaults` as JSON and applied to SwiftUI history rows, VT grid rows, Rust surface sizing, and terminal resize calculations.
- Android terminal viewport: hosts a `SurfaceView` behind the terminal grid, converts the platform `Surface` into a retained `ANativeWindow` in JNI, attaches it to Rust's Android native-window renderer ABI, sends viewport-relative selection/search ranges into Rust through `shellow_engine_set_renderer_overlay_json`, asks Rust to present terminal frames through `shellow_engine_render_frame_viewport_json`, and keeps Compose grid rows transparent for interaction/accessibility hit targets rather than terminal text drawing. The Compose layer applies persisted display settings to font size, line height, and terminal resize calculations, preserves pointer input and selection controls, and hosts a hidden focused `BasicTextField` sentinel for IME text/backspace direct input into the terminal stream.
- Android terminal controls: Compose exposes terminal-viewport direct input, copy/search/paste, clear terminal, reset terminal, a Bottom jump control, auto-follow for new output while search is inactive, shared readline-style local prompt editing including cursor movement, middle insertion/deletion, `Ctrl-A`/`Ctrl-E`/`Ctrl-K`/`Ctrl-U`/`Ctrl-W`, `Ctrl-R` reverse history search, hardware-keyboard native key events, and the special-key toolbar including nano/tmux control keys `^B`, `^O`, and `^X` through JNI calls into the shared Rust session; the visible staging input field is no longer part of the normal terminal path.
- Android transcript export: Compose writes the current visible terminal text to UTF-8 `.txt` files in app-specific Documents storage using the same snapshot-derived text as Copy Terminal.
- iOS Hosts UI: password and private-key profiles open live SSH sheets, store optional password/private-key/passphrase secrets in Keychain by stable profile UUID, start interactive shells, and route terminal input through the Rust bridge.
- Android Hosts UI: password and private-key profiles open Compose SSH dialogs, encrypt optional password/private-key/passphrase secrets with Android Keystore AES-GCM before storing ciphertext in `SharedPreferences`, start interactive shells through JNI, and route terminal input through the same Rust bridge.

## Remaining Demo Work

- Keep tightening the `libghostty-vt` adapter around persistent terminal state and renderer-driven incremental snapshots. The vendored Zig build now produces the iOS XCFramework and Android `arm64-v8a`/`x86_64` JNI slices; Android runtime device proof still needs a reachable ADB device.
- Extend the `rustybuzz-terminal-shaper` path with fallback font chains, emoji/color glyph handling, RTL/bidi terminal policies, and tighter terminal-cell positioning; `terminal-cell-cluster-layout` remains the no-shaper fallback rather than the normal native renderer path.
- Verify the Android `SurfaceView`/`ANativeWindow`/Vulkan `wgpu::Surface` path on a reachable device, then tune z-order and frame pacing for the Rust-owned viewport renderer.
- Verify end-to-end TOFU and saved-secret reconnect behavior against a reachable live SSH server.
