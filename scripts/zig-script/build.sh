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

if zig build-lib $SRC -dynamic -target native -O Debug --output-file ${OUT}; then
  echo "Built ${OUT} (with --output-file)"
elif zig build-lib $SRC -dynamic -target native -O Debug -femit-bin=${OUT}; then
  echo "Built ${OUT} (with -femit-bin)"
else
  echo "zig build-lib failed; trying default build-lib invocation"
  if zig build-lib $SRC -dynamic -target native -O Debug; then
    echo "Built ${OUT} (default output name)"
  else
    echo "zig build-lib failed completely; check your Zig version and flags"
    exit 1
  fi
fi

# On Linux, Zig outputs `libscript.so` by default with `--output-file` or `build-lib`. The -femit-bin above helps on some versions.
# If the above does not produce libscript.so, try:
# zig build-lib $SRC -dynamic -target x86_64-linux -O Debug

if [ -f "${OUT}" ]; then
  echo "Built ${OUT}"
else
  if [ -f "lib${SRC%.zig}.so" ]; then
    mv "lib${SRC%.zig}.so" "${OUT}"
    echo "Renamed output to ${OUT}"
else
    ls -la
    echo "Build did not produce ${OUT}; check Zig version or output path"
    exit 1
  fi
fi

echo "Zig build successful: ${OUT}"

