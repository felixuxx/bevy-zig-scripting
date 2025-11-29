Nice — that’s a big, useful bundle. Below I give you everything you asked for **without code**: a full, production-ready **API function spec**, **ABI struct layouts (conceptual only)**, clear **lifecycle diagrams** (init → update → reload → shutdown), guidance for **multiple scripts per entity**, and a complete **hot-reload manager design** with implementation checklist. Use it as the single-source design doc when you implement.

# High-level notes (constraints & guarantees)

* All Zig→Engine calls are *requests* that become events. Scripts *never* mutate ECS directly.
* All ABI types are plain C-compatible types (opaque handles, integers, floats, null-terminated strings, POD structs).
* The engine must version the ABI and reject incompatible script libraries.
* All operations are single-frame-consistent: scripts post events in their update; engine processes events later in the frame.
* Error handling: script calls return short status or enqueue an error event — engine logs errors but keeps running.
* Threading: script callbacks run on the main game thread only. Event processing runs on the main thread (or a controlled worker) to avoid data races.

---

# 1) Full detailed spec for every API function

Each function below is described as: **Name — Purpose — Parameters — Returns — Behavior & guarantees — Error cases**.

### ENTITY MANAGEMENT

**spawn_entity**

* Purpose: Request creation of a new entity with optional prefab/components/initial transform.
* Parameters: prefab_id (optional string) OR component list (optional), initial_transform (optional struct: pos/rot/scale), tag(s) (optional string/enum).
* Returns: `EntityHandle` (opaque integer handle) or error code.
* Behavior: Creates an entity at the earliest safe point after event processing begins (usually same-frame but after scripts finish). If prefab provided, engine instantiates prefab archetype.
* Guarantees: Returned handle is unique and valid once processed. If spawn is deferred to next frame because of ordering, engine returns a provisional handle that becomes valid when processed (engine must communicate this).
* Error cases: Invalid prefab -> error status; too many entities -> error status.

**despawn_entity**

* Purpose: Mark entity for deletion.
* Parameters: EntityHandle.
* Returns: status.
* Behavior: Entity will be removed safely at end-of-frame or at a safe barrier. Any further events targeting that entity in the same frame are no-ops or cause errors per policy.
* Error cases: Unknown entity -> error status (no crash).

### TRANSFORM & MOVEMENT

**set_position**

* Purpose: Request immediate teleport to absolute position.
* Parameters: EntityHandle, Vec3 position.
* Returns: status.
* Behavior: Overwrites transform’s position when event is processed. Not interpolated.
* Error cases: Entity missing transform -> add transform component automatically (configurable).

**translate**

* Purpose: Request a relative translation.
* Parameters: EntityHandle, Vec3 delta.
* Returns: status.
* Behavior: Engine computes new position = current + delta at application time. If physics is present, request may go through physics system (configurable).
* Error cases: As above.

**set_rotation / rotate / look_at**

* Purpose: Set or change rotation; look_at computes rotation to face a point.
* Parameters: EntityHandle, either Quat or Euler/target position.
* Returns: status.
* Behavior: Engine applies rotation respecting local/global space flag. look_at optionally honors up vector.
* Error cases: Invalid args -> status.

**set_scale**

* Purpose: Set absolute scale.
* Parameters: EntityHandle, Vec3 scale.
* Behavior: Engine applies scale.
* Error cases: Negative scale values -> accepted but logged.

### COMPONENTS (High-level)

**add_component**

* Purpose: Attach a high-level gameplay component to entity.
* Parameters: EntityHandle, ComponentTypeID (enum/string), ComponentPayload (POD struct).
* Returns: status.
* Behavior: If entity lacks component, engine adds it at processing time. If component already exists, behavior is replace or merge depending on component policy.
* Error cases: Unknown component type -> error.

**remove_component**

* Purpose: Remove specified component.
* Parameters: EntityHandle, ComponentTypeID.
* Returns: status.
* Behavior: Removal occurs at processing time; subsequent queries may see last-known data if processed earlier.
* Error cases: Component absent -> no-op or warning.

**get_component (read-only)**

* Purpose: Read a snapshot of component data.
* Parameters: EntityHandle, ComponentTypeID, out_ptr to preallocated POD buffer.
* Returns: status.
* Behavior: Returns data copy as of last processed frame. Not live pointer; cannot be used to mutate ECS.
* Error cases: Type mismatch or absent -> error.

### QUERIES & LOOKUPS

**get_position / get_rotation / get_velocity**

* Purpose: Get current snapshot values.
* Parameters: EntityHandle, out_ptr.
* Returns: status.
* Behavior: Snapshot of latest processed state. If entity unknown -> error.

**find_by_tag**

* Purpose: Return list of EntityHandles matching a tag.
* Parameters: tag string.
* Returns: an array handle or status; engine supplies buffer or returns count first (two-step pattern).
* Behavior: Fast lookup using engine maintained tag index.

**query_nearby**

* Purpose: Spatial query for entities within radius.
* Parameters: Vec3 center, float radius, optional filter bitmask (component types or tags).
* Returns: list/array handle (two-step or buffered).
* Behavior: Engine may use spatial index (quadtree/partitioning) to compute.

### TIME & UTILITIES

**get_delta_time / get_time**

* Purpose: Read the current frame delta and global time.
* Parameters: none (or script_instance).
* Returns: float seconds.
* Behavior: Deterministic for this frame.

**log_info / log_warn / log_error**

* Purpose: Logging to engine console and logs.
* Parameters: null-terminated string.
* Returns: status.
* Behavior: Logs tagged with script id and entity id optionally.

### EVENTS / SIGNALS

**emit_signal**

* Purpose: Fire a named event into engine-wide bus.
* Parameters: signal_name string, optional payload pointer (POD blob), optional target entity handle or broadcast flag.
* Returns: status.
* Behavior: Delivered to connectors during event dispatch step. Immediate delivery within same frame as determined by engine dispatch policy.

**connect_signal / disconnect_signal**

* Purpose: Script registers to receive signals.
* Parameters: entity handle (or script instance handle), signal_name.
* Returns: connection id or status.
* Behavior: Engine guarantees that signal handlers are invoked in safe context (e.g., next frame’s script callbacks or dedicated event-callback phase).

### SCRIPT LIFECYCLE

**init(script_handle)**  — called by engine once when script instance is attached or loaded.

* Purpose: Script sets up initial state, requests resources.
* Parameters: script instance handle (opaque), optional args.
* Returns: none/void or status.
* Behavior: Called exactly once on creation. If hot-reload occurs, may be called again depending on policy.

**update(script_handle, dt)** — called every frame.

* Purpose: Main per-frame logic.
* Parameters: script handle, float dt.
* Returns: none.
* Behavior: Scripts should be short and post events rather than heavy computations.

**shutdown(script_handle)** — called before unloading script.

* Purpose: Clean-up, free script-side resources.
* Behavior: Called before library unload. Engine must await completion.

---

# 2) Layout of the ABI structs (conceptual, no code)

Design rules:

* Use flat POD structs.
* Strings are null-terminated pointers (engine owns copies if needed).
* Use explicit sizes: 32-bit/64-bit integers agreed at ABI start.
* Include `abi_version` and `struct_version` fields in all root structs for forward/backward compatibility.
* Keep reserved fields for future expansion.

### Key conceptual ABI objects

**ScriptInstanceDescriptor**

* `abi_version` (u32) — which ABI the script expects.
* `script_id` (u64) — engine-assigned instance id.
* `entity_handle` (u64) — entity this script is attached to (0 if none).
* `user_data_ptr` (void*) — opaque pointer for engine to store per-script state (optional).
* `reserved[4]` — reserved for future.

**EntityHandle (opaque)**

* Conceptually a 64-bit integer containing generation and index so stale handles can be detected.

**Vec3 (POD)**

* `float x, y, z`.

**Quat (POD)**

* `float x, y, z, w`.

**Transform (POD)**

* `Vec3 position; Quat rotation; Vec3 scale`.

**SpawnRequest (POD)**

* `abi_version` (u32)
* `prefab_id_ptr` (char* null-terminated, optional)
* `initial_transform` (Transform, optional)
* `initial_components_ptr` (pointer to component array, optional)
* `component_count` (u32)
* `tag_ptr` (char* optional)
* `request_id` (u64) — correlates response events.

**ComponentDataEntry (POD)**

* `component_type_id` (u32)
* `payload_ptr` (void*) — pointer to POD data the engine will copy.
* `payload_size` (u32).

**EventEnvelope (POD)**

* `event_type` (u32 enum)
* `script_instance_id` (u64)` or 0
* `entity_handle` (u64) optional
* `payload_ptr` (void*) optional
* `payload_size` (u32)
* `timestamp_frame` (u64) — frame id for ordering.

**QueryBufferRequest (POD)**

* `request_id` (u64)
* `query_type` (u32)
* query parameters…
* `out_buffer_ptr` (void*) — caller-allocated buffer; engine writes into it and returns actual_count.

**StatusResult (POD)**

* `status_code` (u32)
* `message_ptr` (char*) optional (error message).
* `reserved`.

Versioning: every top-level struct contains `abi_version` and `struct_version` so engine and script can refuse mismatched versions.

Ownership & memory rules:

* Any pointer passed from script into engine: engine copies immediately if it needs long-term ownership. Script must not free or mutate until call returns.
* For large results (lists), prefer two-step pattern: request -> engine fills user buffer or returns size -> script reallocates and re-requests. This avoids engine allocating script-side memory.

Pointers and alignment:

* Use natural alignment for the target platform; specify 64-bit pointers on 64-bit builds. Document the target platform word size.

---

# 3) Lifecycle diagrams (init → update → reload → shutdown)

Below are ASCII diagrams showing ordered steps and important decision points.

### A — Normal attach & run lifecycle

```
[Attach request] --> Engine creates ScriptInstance record (id) --> Engine loads script .so (link)
      |
      v
Engine resolves exports (init, update, shutdown) -> Engine calls init(script_instance)
      |
      v
Every frame:
  Engine: begin frame
    -> Call all scripts' update(script_id, dt) [on main thread]
    -> Collect events / requests posted by scripts
    -> Process event queue (apply to ECS)
    -> Other engine systems run (physics, rendering)
  Engine: end frame
```

Key guarantees:

* `init` runs before the first `update`.
* Event posting during `update` is buffered and applied after all updates (or in a controlled phase you define).

### B — Hot-reload flow (safe, optimistic migration)

```
[File change detected] --> HotReloadManager: begin build
      |
      v
if build success:
      |
      v
HotReloadManager: compile -> produce new .so -> load new library into temporary slot
      |
      v
Resolve exports -> call new_lib.init(temp_script_instance)  (optional: call a special "migrate" entry on new lib)
      |
      v
Migrate state: if possible, call old_lib.serialize_state -> engine stores blob -> call new_lib.deserialize_state with blob
      |
      v
After migration success:
   -> Swap engine's ScriptInstance to point at new library (atomic swap)
   -> Call old_lib.shutdown(old_instance) (or call shutdown after swap) 
   -> Unload old library
Else if migration fails:
   -> Keep old lib running OR rollback to last known-good .so
   -> Log errors
```

Notes:

* The engine should keep the old lib loaded until migration completes successfully.
* Migrations may be optional; if not implemented, engine will call shutdown on old and init on new (state lost or reinitialized).
* If hot-reload fails catastrophically during load, engine should rollback to previous library version automatically.

### C — Clean shutdown (engine quit or script detach)

```
Engine requests script unload or engine exiting
   |
   v
Call script.shutdown(script_instance)  (synchronous)
   |
   v
Engine waits for shutdown to complete (timeout policy)
   |
   v
After shutdown completes or timeout:
   -> Unregister ScriptInstance
   -> Unload shared library
```

Timeout & forced-unload:

* If script fails to return from shutdown in time, engine should forcibly proceed: detach instance and unload library (but that can leak). Better: require scripts to be cooperative and enforce timeouts in API.

---

# 4) How to support multiple scripts per entity (design & policies)

Design goals:

* Allow multiple script components on a single entity (common for modular behaviors).
* Provide ordering control (deterministic execution).
* Provide isolation so one script’s crash can’t stomp others.
* Allow intra-script messaging & shared state.

### Data model

* `ScriptComponent` is an attachable component that contains: `script_instance_id`, `priority` (small integer), `enabled` flag, `local_state_handle` (opaque).
* Entities may hold an ordered list (vector) of script components. Ordering uses `priority` and stable insertion order. Engine stores the list.

### Execution ordering

* Per-entity script order: scripts execute in ascending priority (or descending depending on policy).
* Global script order: you may have global script phases: `pre-update`, `update`, `post-update`. Each ScriptComponent configures which phase it wants. This helps with deterministic initialization and safe reads/writes. Example: Physics runs between `pre-update` and `post-update` so scripts that read physics should run `post-update`.

### Isolation & fault handling

* Scripts run inside the same process, so true isolation requires OS processes. For practicality: implement runtime guards:

  * If a script crashes (SIGSEGV), engine catches via platform signal handlers and marks script instance as CRASHED; logs stack if possible; deregisters instance. (This is OS-dependent and tricky; treat as advanced.)
  * More realistic: add runtime watchdog: if a script blocks longer than X ms in update, mark it as hung, disable it automatically and log.
* Per-script try/catch: require script authors to handle errors. Engine wrappers capture high-level failures and prevent full-engine crash where possible.

### Shared state between multiple scripts

* Provide two mechanisms:

  1. **Per-entity shared storage**: a `ScriptVars` component (key-value POD) accessible by scripts attached to the same entity via `get_component("ScriptVars")` read/write interface. Engine enforces atomic ops on write.
  2. **Message bus**: scripts emit signals/events targeted at the entity; other scripts subscribe. This avoids direct shared memory and encourages loose coupling.

### Multiple scripts -> event ordering policy

* Within same frame:

  * Phase 1: All `pre-update` scripts run and post events.
  * Phase 2: Engine processes posted events relevant to core systems (optional).
  * Phase 3: All `update` scripts run.
  * Phase 4: Event application & ECS finalize.
* This prevents inter-script race where script A mutates transform and script B reads it in the same update unexpectedly.

### Attach/detach semantics

* Attaching a script to an entity triggers `init` immediately (or next frame depending on policy).
* Detaching triggers `shutdown`.
* Removing entity also triggers script `shutdown` for all attached scripts.

---

# 5) Hot Reload Manager — full design & operational checklist

The HotReloadManager (HRM) is responsible for watching script sources, building them into shared libs, performing safe load/unload, migrating state if possible, and handling failures/rollbacks.

### Core components

1. **File Watcher**

   * Watches script source directory for changes (create/modify/delete).
   * Debounce changes to avoid repeated builds while saving.

2. **Build Pipeline**

   * Invokes Zig compiler to produce a shared library for changed script(s).
   * Produces deterministic output filenames including version/hash.
   * Captures compiler output and error messages.

3. **Loader / Pending Slot**

   * Loads new shared lib into temporary slot (not yet active).
   * Resolves exported symbols.
   * Performs ABI compatibility check (compare `abi_version`, required exports present).
   * Optionally calls `on_hotload_prepare` on new lib to accept migration.

4. **State Migration Engine** (optional but recommended)

   * Old library exposes `serialize_state(script_instance) -> blob` (optional).
   * New library exposes `deserialize_state(script_instance, blob) -> success` (optional).
   * Engine mediates: call old serialize, store blob, call new deserialise.
   * If both not present, migration is considered impossible; state will be lost unless engine provides a PER-INSTANCE property store that survives reload (engine-side persistent script storage).

5. **Atomic Swap & Rollback**

   * On successful migration/initialization, atomically swap ScriptInstance to new library.
   * On migration failure or crash during swap, rollback: keep old library active and unload the temp lib.
   * Keep last-good version as fallback.

6. **Unload & Cleanup**

   * Call `shutdown` on old library after successful swap (or before swap if policy prefers cold-swap).
   * Unload old library binary after shutdown completes.

7. **Error Reporting & UI**

   * Capture compile errors and present them in a developer console or editor panel with file/line info.
   * If runtime errors occur during init/migrate, show stack / failure cause.
   * Provide a "revert" button to roll back to last known good version.

8. **Concurrency & Locking**

   * Ensure load/unload operations are serialized; only one reload per script instance at a time.
   * Use per-script locks to protect instance state during migration.

9. **Versioning & Metadata**

   * Each compiled .so should embed metadata: script name, compile timestamp, git commit hash (if available), ABI version, exported symbols list.
   * Engine can display these metadata in inspector.

10. **Testing & Safety features**

    * Validate built library in sandboxed invocation: call `health_check` export if present.
    * Provide a "dry-run" mode that loads lib, calls init with a temp instance and discards to check for early crashes.

### Migration strategies (options)

* **No-migration:** simplest. On reload, call shutdown -> load new init -> state lost. Use when script state is small or reconstructible.
* **Engine-persisted state:** engine stores `ScriptVars` component for each script; this survives reload automatically. New script reads from `ScriptVars` in init. This is safe and simple.
* **Serialize/deserialize migration:** advanced. Old lib exports `serialize_state` that returns a binary blob; new lib accepts that blob in `deserialize_state`. Allows precise migration but requires author discipline.

### Atomic swap algorithm (safe pattern)

1. Build new .so (hash H_new).
2. Load new .so into memory, resolve symbols.
3. Validate ABI version and required exports.
4. For each running instance of that script:

   * Lock instance.
   * If migration supported: call old.serialize -> get blob.
   * Create new script instance structure referencing new lib.
   * Call new.deserialize with blob or call new.init.
   * If success -> mark instance to be swapped.
   * If any instance fails -> cancel swap for all and rollback.
   * On global success: swap all instance pointers atomically.
5. Call old.shutdown on every old instance (or after swap depending on policy).
6. Unmap old .so from memory.

### Failure modes & handling

* **Compile fails:** present compiler errors; don't change runtime.
* **Load fails (missing export/ABI mismatch):** log error, present to user, keep old version running.
* **Serialize/deserialize mismatch:** abort migration, present error, keep old running.
* **Init of new libs crash:** abort, rollback, optionally isolate problematic script and disable its hot reload.
* **Old lib refuses shutdown/hangs:** use timeout; if timed out, force unload and log (may leak resources).

### Operational controls for developers

* Manual "Reload now" button + auto-reload toggle.
* "Keep last good" toggle (if auto-rollback should occur).
* Safe-mode: only allow reload in development builds.

---

# Implementation checklist (practical next steps)

1. Define ABI version and publish it in your engine docs.
2. Build simple ScriptInstance record and loader that can load a .so and call `init` and `update`.
3. Implement event queue and wire `spawn`, `set_position`, `translate` to queue events.
4. Implement `get_position` as snapshot read.
5. Implement a basic file watcher + build step producing .so.
6. Implement loader temp slot + ABI checks.
7. Implement atomic swap pattern with engine-persisted `ScriptVars` (start with this before serialize/deserialize).
8. Add UI logging for compiler/runtime errors.
9. Add script inspector to attach/detach scripts to entities.
10. Test multiple scripts per entity and execution ordering.
11. Harden with timeouts and basic error recovery.

---

# Final tips & trade-offs

* **Start simple, then add bells:** begin with engine-persisted state (ScriptVars) and cold-swap (shutdown then init) before attempting binary state migration.
* **Prefer deterministic ordering:** give script authors guarantees about when their code sees state.
* **Make errors visible and non-fatal:** dev productivity depends on clear compiler & runtime feedback.
* **Keep ABI stable:** avoid breaking ABI changes; version major changes.

---
