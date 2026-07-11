# Shellow Architecture

Shellow is a native mobile SSH terminal and remote Codex client for iOS and
Android. The platform applications own the user experience and device
integration, while a shared Rust core owns network sessions, terminal state,
and rendering.

## Repository layout

```text
apps/ios                 SwiftUI application and C ABI bridge
apps/android             Jetpack Compose application and JNI bridge
crates/shellow-core      SSH, terminal, renderer, and Codex session logic
crates/shellow-ffi       C ABI and Android JNI exports
crates/libghostty-vt-sys Mobile build patch for libghostty-vt
scripts                  Native Rust build scripts
```

Generated XCFrameworks and Android shared libraries are build artifacts and are
not committed.

## System boundaries

### Native applications

SwiftUI and Jetpack Compose own navigation, profile editing, settings,
accessibility, clipboard interaction, keyboard and pointer input, and platform
lifecycle events. Secrets remain on the device: iOS uses Keychain and Android
uses Keystore-backed encrypted storage.

The native applications do not parse terminal escape sequences or implement
SSH. They send user actions to Rust and render state returned in snapshots.

### Native bridge

`shellow-ffi` exposes a narrow C ABI. Swift calls it directly through the
generated iOS framework; Android calls the same operations through JNI. Values
that cross this boundary are handles, primitive values, byte buffers, or JSON
snapshots.

The bridge is intentionally thin. Session behavior and state transitions belong
in `shellow-core` so both platforms receive the same semantics.

### Shared Rust core

`shellow-core` owns:

- SSH connection and interactive shell lifecycle through `russh`.
- Terminal emulation and persistent VT state through `libghostty-vt`.
- Terminal snapshots, dirty rows, cursor and mode state, scrollback, title,
  bell, and OSC 52 effects.
- Selection and search overlay ranges consumed by the renderer.
- Codex proxy sessions, JSON-RPC messages, threads, turns, and approvals.
- Platform-independent session state and error reporting.

Host profiles and secrets remain native concerns; authenticated connection
parameters are passed into the core when a session starts.

### Terminal renderer

The Rust renderer consumes terminal snapshots and draws the visible grid with
`wgpu`. It owns the GPU device, queue, glyph atlas, shaped glyph layout, dirty
row uploads, and selection/search overlays. Native applications retain an
interaction and accessibility layer over the rendered grid.

On iOS, an `MTKView` supplies a `CAMetalLayer`. On Android, a `SurfaceView`
supplies an `ANativeWindow`. These platform surfaces are attached to the shared
renderer through the native bridge. Metal and Vulkan are selected by `wgpu` on
their respective platforms.

Text shaping uses `rustybuzz` and glyph rasterization uses `fontdue` when a
platform monospace font is available. A simpler cell layout and procedural
rasterizer remain fallbacks for environments without a usable font.

## Session flows

### Terminal

```text
keyboard / pointer / clipboard
        -> native application
        -> C ABI or JNI
        -> Rust SSH session
        -> libghostty-vt state
        -> terminal snapshot
        -> wgpu renderer and native accessibility layer
```

The Rust session receives remote bytes, updates the VT state, and exposes a new
snapshot. Input follows the reverse path and is written to the SSH channel.
Resize events originate from native layout and update both the remote PTY and
renderer viewport.

### Codex

Codex workspaces use a separate SSH exec channel. The remote command starts the
persistent `codex app-server` daemon when needed and connects through
`codex app-server proxy`. Shellow exchanges JSON-RPC messages with that proxy
and models threads, messages, active turns, settings, and approval requests in
the shared core.

The proxy connection follows the mobile SSH session. The daemon and its turns
can continue on the remote host after the phone disconnects, allowing a later
connection to resume a thread.

## Design rules

- Keep platform-specific UI and secure storage in the native applications.
- Keep SSH, terminal semantics, Codex protocol handling, and rendering behavior
  shared in Rust.
- Add cross-platform operations to `shellow-core` before exposing them through
  the bridge.
- Treat JSON snapshots as a compatibility boundary and evolve them deliberately.
- Keep verification history in tests and issue tracking, not in architecture
  documentation.
