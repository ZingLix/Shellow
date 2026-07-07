This directory contains renderer assets compiled into `shellow-core`.

`JetBrainsMono-Regular.ttf` is copied from Ghostty's vendored font resources and is used as the default embedded monospace font for the Rust `wgpu` terminal glyph atlas. Keeping a bundled font prevents mobile builds from falling back to the procedural debug rasterizer when platform font files are not readable from Rust.
