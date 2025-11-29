# Bevy Zig Scripting — Prototype Scaffolding

This workspace contains documentation and an initial scaffold for using Zig as a scripting language with a Bevy-based engine.

The scaffold includes:

- `engine/` — a minimal Rust example loader that demonstrates dynamically loading a Zig library and calling exported functions.
- `scripts/zig-script/` — a Zig script template that exports `zig_init` and `zig_update` and a build script to produce a shared library.

Requirements
- Rust toolchain (cargo + rustc)
- Zig compiler
- Fish or bash for build scripts (build scripts are simple `sh` compatible)

Quick start
1. Build the Zig script (one-shot build):

```bash
cd scripts/zig-script
./build.sh
```

2. Build and run the Rust engine loader:

```bash
cd engine
cargo run
```

You should see the Rust loader load `libscript.so` from the Zig script build and call `zig_init` and `zig_update` for a few frames.

Next steps
- Replace the minimal Rust loader with a Bevy system wrapped around `libloading`.
- Add more ABI calls and a small `EngineAPI` in Rust and Zig-backed stdlib for convenience.
- Implement file watcher + hot reload (notify crate) and for incremental builds.

See `docs/` for detailed design and implementation notes.
