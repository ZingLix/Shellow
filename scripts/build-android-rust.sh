#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ANDROID_DIR="$ROOT_DIR/apps/android/app/src/main"
JNI_LIBS_DIR="$ANDROID_DIR/jniLibs"
API_LEVEL="${ANDROID_API_LEVEL:-26}"
SDK_DIR="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
NDK_DIR="${ANDROID_NDK_HOME:-$SDK_DIR/ndk/27.1.12297006}"

if [ -n "${ANDROID_NDK_PREBUILT_HOST_TAG:-}" ]; then
  NDK_HOST_TAG="$ANDROID_NDK_PREBUILT_HOST_TAG"
else
  case "$(uname -s):$(uname -m)" in
    Linux:x86_64 | Linux:amd64)
      NDK_HOST_TAG="linux-x86_64"
      ;;
    Linux:aarch64 | Linux:arm64)
      NDK_HOST_TAG="linux-aarch64"
      ;;
    Darwin:*)
      NDK_HOST_TAG="darwin-x86_64"
      ;;
    *)
      echo "Unsupported Android NDK host platform: $(uname -s) $(uname -m)" >&2
      exit 1
      ;;
  esac
fi

TOOLCHAIN_DIR="$NDK_DIR/toolchains/llvm/prebuilt/$NDK_HOST_TAG/bin"

if [ ! -x "$TOOLCHAIN_DIR/aarch64-linux-android${API_LEVEL}-clang" ]; then
  echo "Android NDK clang was not found at $TOOLCHAIN_DIR." >&2
  echo "Set ANDROID_NDK_HOME or ANDROID_NDK_PREBUILT_HOST_TAG if your NDK is installed elsewhere." >&2
  exit 1
fi

source "$ROOT_DIR/scripts/libghostty-vt-env.sh"

export AR_aarch64_linux_android="$TOOLCHAIN_DIR/llvm-ar"
export CC_aarch64_linux_android="$TOOLCHAIN_DIR/aarch64-linux-android${API_LEVEL}-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN_DIR/aarch64-linux-android${API_LEVEL}-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-soname,libshellow_ffi.so"

export AR_x86_64_linux_android="$TOOLCHAIN_DIR/llvm-ar"
export CC_x86_64_linux_android="$TOOLCHAIN_DIR/x86_64-linux-android${API_LEVEL}-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$TOOLCHAIN_DIR/x86_64-linux-android${API_LEVEL}-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_RUSTFLAGS="-C link-arg=-Wl,-soname,libshellow_ffi.so"

cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p shellow-ffi --release --features native-integrations --target aarch64-linux-android
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" -p shellow-ffi --release --features native-integrations --target x86_64-linux-android

mkdir -p "$JNI_LIBS_DIR/arm64-v8a" "$JNI_LIBS_DIR/x86_64"
install -m 0644 "$ROOT_DIR/target/aarch64-linux-android/release/libshellow_ffi.so" "$JNI_LIBS_DIR/arm64-v8a/libshellow_ffi.so"
install -m 0644 "$ROOT_DIR/target/x86_64-linux-android/release/libshellow_ffi.so" "$JNI_LIBS_DIR/x86_64/libshellow_ffi.so"

echo "Built Android Rust libraries in $JNI_LIBS_DIR"
