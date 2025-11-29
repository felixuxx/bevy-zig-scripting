# Prioritized Implementation Checklist

This checklist is a prioritized, pragmatic rollout plan from minimal prototype to robust Zig script integration for a Bevy-based engine. Use it to guide incremental work with clear deliverables.

## Level 0 — Prototype (High Priority — MVP)
1. Decide ABI basics and document them (`docs/` exists). (✓)
2. Build a minimal loader: load `.so` and call `zig_init` / `zig_update`. (Scaffold: `engine` + `scripts/zig-script`) (✓)
3. Zig script template and build script (Scaffold: `scripts/zig-script`). (✓)
4. Basic logging from Zig script to stdout for initial verification. (✓)
5. Document quick start and build commands (`README.md` in project root). (✓)

## Level 1 — Engine Integration & Safety
1. Integrate a small event queue: Zig → Engine events; engine processes events after calling updates.
2. Add a minimal `EngineAPI` with plain POD types, e.g., `spawn_entity` and `set_position` stubs in Rust and Zig FFI wrappers.
3. Make Rust side call into a generic `zig_update(dt)`, but also provide a simple `log` callback usable by Zig.
4. Create `ScriptComponent` type (Rust) and store script metadata in a Bevy resource or a simple registry.

## Level 2 — Hot Reloading
1. Implement file watcher (using `notify` crate) in the engine prototype to detect `*.zig` changes.
2. Add a build pipeline to compile changed scripts incrementally with Zig.
3. Implement temp load/validate ABI -> swap -> call `init` on new instance -> `shutdown` old instance when safe.
4. Implement an engine-persisted `ScriptVars` per instance to persist simple state across reloads.

## Level 3 — Safety, Versioning & Tooling
1. Implement ABI version checking and fail-safe on mismatch.
2. Implement runtime sandboxing features: watchdog, timeouts for `update` and `shutdown` to avoid blocking engine.
3. Add robust error & compile message reporting in the engine (capture Zig compiler output and highlight in UI).
4. Add a UI Inspector to the editor (optional) to attach/detach scripts and display script metadata.

## Level 4 — Multi-Script Policy & Advanced Features
1. Allow multiple scripts per entity with ordered execution (priority + phases pre/update/post).
2. Shared per-entity `ScriptVars` with atomic access from script runtime.
3. Implement two-step/safer query APIs for large result sets (two call pattern: query -> provide buffer).
4. Add advanced features: health checks, `serialize_state`/`deserialize_state` for advanced migration.

## Level 5 — Polishing and Tests
1. Test Plan execution: create automated tests to simulate reload, ABI mismatch, and stress scenarios (`tests/`).
2. Add Zig stdlib wrappers for convenience and ergonomics for script authors.
3. Add full examples: rotating cube, simple player controller, AI script, hot-reload demo.
4. Add documentation & tutorials for developer onboarding.

---

## Notes / Tips
- Start simple: engine-side `ScriptVars` plus a safe event queue buys most practical benefits without binary state migration complexity.
- Make ABI with clear version numbers for evolution and backward compatibility.
- Make hot-reload optional and safe; include a "dry-run" mode to verify new libs before swapping.
- Add debugging ergonomics: show compile errors and a stack trace if a script panics (Zig `std.debug` output), plus metadata (hash, timestamp).

---

## Suggested Next Steps (immediate):
1. In the prototype loader (`engine`), add a `log` callback to the ABI that Zig can call instead of stdout — this demonstrates engine->script->engine log round-trip.
2. Add a small event struct and a `translate` request; let Zig call `emit_translate` to move a fake-world entity in the Rust loader.
3. Implement a simple file-watcher that triggers `build.sh` and reloads changed library (safe cold-swap).

This checklist is a living document — refer to `docs/` and iterate, adding more tests and robustness as you proceed.
