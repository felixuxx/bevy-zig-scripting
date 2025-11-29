# Zig Script Template

This folder contains a minimal Zig script template and a build script to produce a shared library.

Files
- `script.zig` — minimal script that exports `zig_init` and `zig_update`.
- `build.sh` — compiles script with Zig.

Build & run

```bash
cd scripts/zig-script
./build.sh
```

This will output `libscript.so` in the same folder. The engine loader expects the file in `scripts/zig-script/libscript.so`.

Note: Zig build output may vary based on the Zig version; the `build.sh` script attempts a simple `zig build-lib` invocation which should work on modern releases.

Contributing
- Add more exported functions to the Zig template to exercise more ABI features (e.g., logging callbacks, spawn entity requests, event queueing). Ensure Rust side declares matching `libloading` symbol signatures.

