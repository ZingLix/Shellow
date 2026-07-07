# Rust Workspace

Shellow's mobile apps share this Rust workspace.

- `shellow-core`: session actors, profile DTOs, terminal snapshots, `russh` integration, direct `libghostty-vt` terminal handling, and the shared `wgpu` renderer runtime.
- `shellow-ffi`: thin C ABI exports used by iOS directly and by Android through JNI.
- `libghostty-vt-sys`: local `[patch.crates-io]` copy of the upstream `0.2.0` sys crate so Ghostty's vendored Zig build supports Shellow's iOS and Android targets.

Current native targets:

- Apple: static library packaged into `ShellowCore.xcframework`.
- Android: shared libraries packaged as `libshellow_ffi.so` for `arm64-v8a` and `x86_64`.

Build helpers live in `../scripts`.
