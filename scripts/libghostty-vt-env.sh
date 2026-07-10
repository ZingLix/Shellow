#!/usr/bin/env bash

export ZIG_GLOBAL_CACHE_DIR="${ZIG_GLOBAL_CACHE_DIR:-$ROOT_DIR/work/zig-global-cache}"
export LIBGHOSTTY_VT_SYS_OPTIMIZE="${LIBGHOSTTY_VT_SYS_OPTIMIZE:-ReleaseFast}"

if [ -z "${GHOSTTY_SOURCE_DIR:-}" ]; then
  if [ -f "$ROOT_DIR/work/ghostty-src/build.zig" ]; then
    export GHOSTTY_SOURCE_DIR="$ROOT_DIR/work/ghostty-src"
  elif [ -d "$ROOT_DIR/target" ]; then
    EXISTING_GHOSTTY_BUILD_ZIG="$(
      find "$ROOT_DIR/target" -path '*/ghostty-src/build.zig' -type f -print -quit 2>/dev/null || true
    )"
    if [ -n "$EXISTING_GHOSTTY_BUILD_ZIG" ]; then
      export GHOSTTY_SOURCE_DIR="$(dirname "$EXISTING_GHOSTTY_BUILD_ZIG")"
    fi
  fi
fi
