Below is a **clean, realistic, production-ready SCRIPT API design** for how Zig scripts should interact with your Bevy-based engine **without touching ECS directly**.

This is **not code** — this is an **architectural + conceptual API design** you will later implement through ABI calls.

The design focuses on:

* **spawn**
* **move / transform**
* **query ECS**
* **logging**
* **timers/events**
* **script-local state**

Everything is structured so that Zig scripts stay *safe*, *simple*, and *powerful*, while your engine controls all actual ECS operations.

---

# ⭐ HIGH-LEVEL GOALS OF THE SCRIPT API

1. Zig scripts **request operations**, not execute them directly.
2. The engine maintains **ownership of ECS**.
3. Scripts communicate through:

   * a **safe API layer**
   * a **Zig→Engine event queue**
4. API must be simple and stable (C ABI).

---

# ⭐ SCRIPT API OVERVIEW

Zig scripts can call the following conceptual API:

### **Entity Management**

* `spawn_entity(prefab_id or component set)`
* `despawn_entity(entity_id)`

### **Transform**

* `set_position(entity_id, Vec3)`
* `set_rotation(entity_id, Quat)`
* `set_scale(entity_id, Vec3)`
* `translate(entity_id, Vec3)`
* `rotate(entity_id, Quat)`

### **Queries**

* `get_position(entity_id)`
* `get_rotation(entity_id)`
* `get_velocity(entity_id)`
* `has_component(entity_id, component_type)`
* `find_by_tag(tag)`
* `query_nearby(position, radius)`

### **Components**

* `add_component(entity_id, component_type, data)`
* `remove_component(entity_id, component_type)`
* `get_component(entity_id, component_type)` *(read-only)*

### **Math Helpers**

* `forward(entity_id)`
* `right(entity_id)`
* `look_at(entity_id, target_position)`

### **Engine Utilities**

* `log_info(str)`
* `log_warn(str)`
* `log_error(str)`

### **Time**

* `get_delta_time()`
* `get_time()`

### **Events**

* `emit_signal("event_name", payload)`
* `connect_signal(entity_id, "event_name")`

---

# ⭐ SCRIPT-SIDE API (AS DESIGNED, NOT CODE)

Below is a conceptual specification of how the API is structured.

These are **the operations a Zig script can request**, not how to implement them.

---

# 🔷 **1. Entity Management API**

### **spawn_entity()**

**Purpose:** Request the engine to create a new entity.

**Parameters:**

* optional prefab ID
* optional initial transform
* optional initial component list

**Return:**

* `EntityID` (opaque integer or 64-bit handle)

**Usage Pattern:**

```
entity = spawn_entity({
    tag = "Enemy",
    transform = { position = Vec3(0,0,0) }
})
```

---

### **despawn_entity(entity_id)**

Marks entity for safe deletion at end of frame.

---

# 🔷 **2. Transform & Movement API**

### **set_position(entity_id, Vec3)**

Request teleport-like movement.

### **translate(entity_id, Vec3)**

Move relative to current transform.

### **set_rotation(entity_id, Quat)**

### **look_at(entity_id, target_pos)**

Engine performs the math → script only issues a request.

### **set_scale(entity_id, Vec3)**

---

# 🔷 **3. Component API (Safe Limited ECS)**

Scripts are allowed to manipulate high-level gameplay components only (not low-level engine components).

### **add_component(entity_id, ComponentType, ValueStruct)**

Example components:

* Health
* AIState
* MovementSpeed
* ScriptVariables

### **remove_component(entity_id, ComponentType)**

### **get_component(entity_id, ComponentType)**

**Read-only** to prevent ECS corruption.

---

# 🔷 **4. Query API**

### **get_position(entity_id)**

### **get_rotation(entity_id)**

Queryable types:

* transforms
* simple movement data
* physics state (if allowed)

### **find_by_tag(tag)**

Engine returns a list of matching entity IDs.

### **query_nearby(center, radius)**

Engine returns nearest entities efficiently (your implementation may use Bevy’s spatial queries).

---

# 🔷 **5. Time API**

### **get_delta_time()**

Returns `dt`.

### **get_time()**

Game time in seconds.

Used for timers, animations, AI behavior, etc.

---

# 🔷 **6. Logging API**

### **log_info(message)**

### **log_warn(message)**

### **log_error(message)**

Scripting errors appear in your engine console.

---

# 🔷 **7. Event / Signal API**

### **emit_signal(signal_name, payload?)**

Example:

```
emit_signal("OnHit", { damage = 20 })
```

### **connect_signal(entity, eventName)**

Scripts can listen to signals coming from engine or other scripts.

Events are delivered next frame as safe callbacks.

---

# 🔷 **8. Script Lifecycle API**

Scripts export:

* `init(script_instance)`
* `update(script_instance, dt)`
* `shutdown(script_instance)`

Engine injects the `script_instance` which includes:

* `entity_id`
* user-defined script state (if any)

---

# ⭐ INTERNAL ENGINE-SIDE DESIGN (IMPORTANT)

To implement the API above cleanly, you need three subsystems:

---

# 🔶 A. ENGINE API LAYER

This is a C ABI–safe interface containing functions like:

* `EngineAPI.spawn_entity(request_ptr)`
* `EngineAPI.get_position(entity_id, out_ptr)`

This layer:

* has **no generics**
* uses only **plain structs**
* never exposes ECS directly

---

# 🔶 B. EVENT QUEUE

All Zig→Engine calls become **events**:

Example event:

```
{
  type: SET_POSITION,
  entity_id: 51,
  position: (2.0, 0.0, 5.0)
}
```

Engine processes events after script updates.

---

# 🔶 C. EVENT PROCESSOR

Translates events into ECS operations.

This ensures:

* safety
* multithreading compatibility
* predictable update order

---

# ⭐ FINAL: EXAMPLE OF A SCRIPT FLOW (ARCHITECTURAL)

```
Zig script calls translate(entity, Vec3(1,0,0))
             │
             ▼
 Event Request:
  { type: TRANSLATE, entity, vector }
             │
             ▼
 Added to Zig→Engine Event Queue
             │
             ▼
Engine Script Manager processes queue
             │
             ▼
ECS System applies translation to actual Transform component
```

The script never touches the Transform component directly.

---

