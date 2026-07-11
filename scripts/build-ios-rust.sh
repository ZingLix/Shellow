#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
INCLUDE_DIR="$ROOT_DIR/crates/shellow-ffi/include"
OUTPUT_DIR="$ROOT_DIR/apps/ios/Frameworks"
FRAMEWORK_DIR="$OUTPUT_DIR/ShellowCore.xcframework"

source "$ROOT_DIR/scripts/libghostty-vt-env.sh"

# Keep Rust/C dependencies compatible with the app target instead of inheriting the
# host Xcode SDK version (which can make the static archive require a newer iOS).
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-17.0}"

cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p shellow-ffi --release --features native-integrations --target aarch64-apple-ios
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p shellow-ffi --release --features native-integrations --target aarch64-apple-ios-sim

rm -rf "$FRAMEWORK_DIR"
mkdir -p "$OUTPUT_DIR"

xcodebuild -create-xcframework \
  -library "$ROOT_DIR/target/aarch64-apple-ios/release/libshellow_ffi.a" \
  -headers "$INCLUDE_DIR" \
  -library "$ROOT_DIR/target/aarch64-apple-ios-sim/release/libshellow_ffi.a" \
  -headers "$INCLUDE_DIR" \
  -output "$FRAMEWORK_DIR"

echo "Built $FRAMEWORK_DIR"
