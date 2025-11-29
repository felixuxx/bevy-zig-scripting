Refined, step-by-step **implementation plan** focusing on the architectural and design decisions, *without* providing the actual code.

---

## 🏗️ Implementation Plan: Zig Scripting for Bevy

This plan focuses on the major structural tasks required to build the bridge between the **Bevy ECS (Rust)** and the **Zig script runtime**. 

---

### Phase 1: FFI Foundation & Build System

The goal here is to establish the basic communication channel and compilation workflow.

| Step | Focus Area | Key Architectural Decision |
| :--- | :--- | :--- |
| **1. Build Integration** | **Tooling** | **Define the build process:** Use a Rust `build.rs` script to invoke the Zig compiler, ensuring the dynamic library (`.so`, `.dll`, `.dylib`) is placed where the Bevy executable can find it. |
| **2. FFI Declarations** | **C-ABI Bridge** | Create a `zig_ffi` module in the Rust crate. All functions within this module must use `#[no_mangle] extern "C"` for **ABI compatibility**. Mirror these declarations precisely in a Zig `extern` block. |
| **3. Dynamic Loading** | **Runtime Linkage** | Utilize the `libloading` Rust crate to programmatically load the Zig library at Bevy startup. Store the resulting `Library` handle in a Bevy **Resource** to prevent it from being prematurely unloaded. |
| **4. Basic Ping-Pong Test** | **Verification** | Implement a simple function in Zig (`zig_echo`) and a logging function in Rust (`bevy_log`). Load the Zig function symbol in Rust and call it, which, in turn, calls the Rust logger. This validates the entire FFI chain. |

---

### Phase 2: ECS Access and World Interoperability

This is the most complex phase, involving the safe transfer of the Bevy **World** and ECS operations.

| Step | Focus Area | Key Architectural Decision |
| :--- | :--- | :--- |
| **5. Opaque World Pointer** | **World Transfer** | **Never expose Bevy's internal `World` structure to Zig.** Instead, pass a **raw, opaque pointer** (`*mut World` in Rust $\to$ `*anyopaque` in Zig). This prevents Zig from knowing or depending on Rust's memory layout. |
| **6. System Symbol Retrieval** | **System Registration** | After loading the library, use `libloading` to fetch a specific function pointer (e.g., `zig_update_system`) that adheres to a known C signature (`fn(*anyopaque)`). |
| **7. System Adapter** | **Bevy Integration** | Create a Rust **adapter system** that takes a mutable `&mut World` (required by Bevy) and, inside its body, safely casts and passes the raw world pointer to the loaded Zig function pointer. Then, register this adapter system with Bevy's `Update` schedule. |
| **8. Component Access API** | **C-API Design** | Define a C-API function in Rust for component access, such as `bevy_get_transform(world_ptr, entity_id, out_vec3_ptr)`. The Zig script must call this function, passing an entity ID and a pointer to a struct in Zig's memory where the Rust function will write the component data. |
| **9. Zig Component Structs** | **Data Mapping** | In Zig, define structs that are **memory-layout compatible** with the core Bevy components (e.g., `Vec3` for `Transform.translation`). This ensures safe data passing across the FFI boundary. |

---

### Phase 3: Scripting Workflow and Reloading

The final phase addresses quality-of-life features critical for developer experience.

| Step | Focus Area | Key Architectural Decision |
| :--- | :--- | :--- |
| **10. Script State Component** | **Entity Linking** | Define a new Bevy component (e.g., `ZigScriptState`) that stores **Zig-specific data**, such as a pointer to the script's internal state memory allocated by Zig, or a reference to a specific script name/handle. |
| **11. Hot Reload Logic** | **Development Flow** | **Use Bevy's Asset Watcher** or file system events to monitor changes to the Zig source. When a change is detected: **a) Compile** the new library, **b) Unload** the old library (requires careful handling of the `Library` resource), and **c) Load** the new library. |
| **12. State Transfer (Hot Reload)** | **Memory Management** | Design a mechanism to **transfer essential game state** (like component data, not the whole `World`) from the old script's memory before unloading to the new script's memory after loading. This is the hardest part of hot-reloading. |
| **13. Error Handling** | **Stability** | Implement a Zig function that can be called before a crash (`defer` or a panic handler) to safely call back into Rust's logging system, providing a clean error message and stack trace. This prevents a full Bevy crash from a Zig panic. |

---
