Here is a **single-page cheat sheet** (compact, reference-style, no code, no long explanations).
You can paste this directly into your engine folder as *“Scripting API Quick Reference.md”* or similar.

---

# **Zig Scripting API — Quick Reference (1-page)**

*(All calls are ABI-safe, event-based; scripts do not directly mutate ECS)*

---

## **ENTITY**

### `spawn_entity(prefab_id?, initial_transform?, components?, tags?) → EntityHandle`

Creates a new entity next safe frame. Returns handle (may be provisional).
**Notes:** Prefab OR component list. Initial transform optional.

### `despawn_entity(entity)`

Marks entity for deletion at end-of-frame.

---

## **TRANSFORM**

### `set_position(entity, Vec3 pos)`

Teleport to absolute position.

### `translate(entity, Vec3 delta)`

Relative movement. Applied after event queue.

### `set_rotation(entity, Quat rot)`

Set absolute rotation.

### `rotate(entity, Vec3 delta_euler)`

Relative rotation.

### `look_at(entity, Vec3 target, Vec3 up?)`

Rotate entity to face direction.

### `set_scale(entity, Vec3 scale)`

Set absolute scale.

---

## **COMPONENTS**

### `add_component(entity, ComponentTypeID, payload_ptr, payload_size)`

Attach component. Replaces or merges depending on type.

### `remove_component(entity, ComponentTypeID)`

Remove component.

### `get_component(entity, ComponentTypeID, out_ptr)`

Snapshot copy of component.

---

## **QUERIES**

### `get_position(entity, out_ptr)`

### `get_rotation(entity, out_ptr)`

### `get_velocity(entity, out_ptr)`

Snapshot reads.

### `find_by_tag(tag) → count`

Two-step: first call returns count; second call with buffer fills list.

### `query_nearby(center, radius, filter?) → count`

Spatial query using filter (tags/components).

---

## **TIME**

### `get_delta_time() → float`

Current frame dt.

### `get_time() → float`

Global engine time.

---

## **LOGGING**

### `log_info(message)`

### `log_warn(message)`

### `log_error(message)`

Logs tagged with script + entity IDs.

---

## **EVENTS / SIGNALS**

### `emit_signal(name, payload_ptr?, payload_size?, target_entity?)`

Broadcast or targeted event.

### `connect_signal(script_or_entity, name)`

Register for event delivery.

### `disconnect_signal(conn_id)`

Remove signal listener.

---

## **SCRIPT LIFECYCLE**

### `init(script_instance)`

Called once when script is attached or after hot-reload.

### `update(script_instance, dt)`

Main per-frame logic. Post events here.

### `shutdown(script_instance)`

Cleanup before unload.

---

## **ABI STRUCT PRIMITIVES** *(names only for recall)*

* `EntityHandle`
* `Vec3`
* `Quat`
* `Transform`
* `SpawnRequest`
* `ComponentDataEntry`
* `EventEnvelope`
* `QueryBufferRequest`
* `StatusResult`
* `ScriptInstanceDescriptor`

*(All structs are POD; contain `abi_version`, `struct_version`, and reserved fields.)*

---

## **MULTIPLE SCRIPTS PER ENTITY — REMINDERS**

* Each script = `ScriptComponent(entity, script_id, priority, enabled)`
* Execution order = by priority (stable).
* Shared state via `ScriptVars` component or signals.
* Attach = `init`; detach = `shutdown`.

---

## **HOT RELOAD — CORE RULES**

* Watch source → build → temp load → ABI check.
* If migrate: old.serialize → new.deserialize.
* If no migrate: engine-persisted `ScriptVars` + new.init.
* Atomic swap only after all instances validate.
* On failure: rollback to last-good.

---
