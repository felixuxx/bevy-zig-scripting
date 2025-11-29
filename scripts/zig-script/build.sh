#!/usr/bin/env sh
# Build script to compile script.zig into libscript.so using Zig
# Requires: zig compiler in PATH

set -e

SRC=script.zig
OUT=libscript.so

if ! command -v zig >/dev/null 2>&1; then
  echo "zig is not installed or not in PATH. Install zig to build scripts."
  exit 1
fi

zig build-lib $SRC -dynamic -target native -O Debug -lc -femit-bin=${OUT}

# On Linux, Zig outputs `libscript.so` by default with `--output-file` or `build-lib`. The -femit-bin above helps on some versions.
# If the above does not produce libscript.so, try:
# zig build-lib $SRC -dynamic -target x86_64-linux -O Debug

if [ -f "${OUT}" ]; then
  echo "Built ${OUT}"
else
  # Look for the default output; Zig will usually write lib<name>.so
  DEFAULT_OUT=libscript.so
  if [ -f "$DEFAULT_OUT" ]; then
    echo "Built $DEFAULT_OUT"
  else
    ls -la
    echo "Build did not produce ${OUT}; check Zig version or output path"
    exit 1
  fi
fi

# Ensure lib is copied to the project root for the engine loader path
cp -f ${OUT} ../../scripts/zig-script/${OUT}

echo "Copied ${OUT} to ../../scripts/zig-script/${OUT}"

