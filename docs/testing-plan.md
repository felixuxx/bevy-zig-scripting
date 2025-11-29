Below is a **complete test plan** for your Zig-as-scripting-language runtime, covering **Hot Reload Manager** + **Multi-Script per Entity** scenarios.
This is structured like a real engine QA document: clear objectives, test cases, expected results, failure conditions, and required tools.

---

# **Zig Scripting Runtime — Test Plan**

## **1. Scope**

Validate the correctness, stability, and determinism of:

### **A. Hot Reload Manager**

* Detecting file changes
* Rebuilding Zig scripts
* Swapping dynamic libraries
* State preservation & migration
* Handling errors (build failure, ABI mismatch)

### **B. Multi-Script Support**

* Attaching multiple scripts to one entity
* Ensuring deterministic execution order
* Handling conflicting commands
* Managing per-script state and lifecycles
* Interaction between scripts

---

# **2. Test Categories**

1. Functional tests
2. Stress and performance tests
3. Error and failure injection
4. Concurrency / edge-case behavior
5. Regression-style tests for complex interactions

---

# **3. Functional Test Cases**

## **3.1 Hot Reload Manager — Basic Flow**

### **Test HR-01: Detect change & rebuild**

**Steps:**

1. Modify a Zig script file.
2. Save file.
3. Observe file watcher → build trigger.

**Expected:**

* Change is detected within N ms.
* Compile starts immediately.
* No unnecessary rebuilds.

---

### **Test HR-02: Swap libraries**

**Steps:**

1. Trigger hot reload.
2. Capture timestamp / pointer of current dynamic library.
3. Allow manager to swap in new library.

**Expected:**

* New library handle differs from old.
* Old library is fully unloaded.
* Function pointers updated correctly.

---

### **Test HR-03: State preservation**

**Steps:**

1. Script writes state to ABI memory block (position, counters, timers, etc.).
2. Trigger reload.
3. Inspect new script instance.

**Expected:**

* State is identical (byte-accurate) in new script.
* No missing fields.
* No default resets unless coded.

---

### **Test HR-04: zig_reload callback**

**Steps:**

1. Add debug log inside zig_reload.
2. Trigger reload.

**Expected:**

* zig_reload is called exactly once.
* Receives correct Context & state pointer.

---

### **Test HR-05: Update loop continuity**

**Steps:**

1. Create script that increments a counter each update.
2. Record counter value at frame N.
3. Hot reload on frame N+1.

**Expected:**

* Counter continues incrementing normally.
* No double-calling update.
* No skipped frames.

---

## **3.2 Hot Reload — Error Scenarios**

### **Test HR-E01: Build failure handling**

**Steps:**

1. Introduce Zig syntax error.
2. Save file.

**Expected:**

* Build fails → engine logs error.
* Old script continues running without interruption.
* No crash or unload.

---

### **Test HR-E02: ABI mismatch**

**Steps:**

1. Change struct layout (remove field, change type).
2. Trigger reload.

**Expected:**

* Hot reload manager refuses reload.
* Clear ABI mismatch error.
* Old script continues safely.

---

### **Test HR-E03: Runtime panic in zig_init**

**Steps:**

1. Insert panic in zig_init.
2. Attach script or reload.

**Expected:**

* Script load aborts safely.
* No partial state left in ECS.
* Error surfaced to user.

---

## **3.3 Multi-Script per Entity — Core**

### **Test MS-01: Attach multiple scripts**

**Steps:**

1. Attach ScriptA and ScriptB to same entity.
2. Run update loop.

**Expected:**

* Both scripts receive zig_update.
* Execution order follows config (e.g., alphabetical or user-defined).
* Output logs reflect correct ordering.

---

### **Test MS-02: Independent state**

**Steps:**

1. ScriptA increments counter A.
2. ScriptB increments counter B.
3. Run for 100 frames.

**Expected:**

* Counters never interfere with each other.
* No cross-contamination of memory.

---

### **Test MS-03: Multiple scripts modify same component**

**Steps:**

1. ScriptA moves entity +X.
2. ScriptB moves entity +Y.
3. Run update loop.

**Expected:**

* Final position reflects combined result.
* Determinism: same order → same result.

**Failure:**

* Race conditions
* Non-deterministic ordering
* Lost writes

---

### **Test MS-04: Add/remove script at runtime**

**Steps:**

1. Entity starts with ScriptA only.
2. During frame, ScriptA attaches ScriptB.
3. After 10 frames, remove ScriptA.

**Expected:**

* ScriptB starts next frame (init once).
* ScriptA shutdown called.
* No duplicate updates or missing callbacks.

---

## **3.4 Multi-Script — Hot Reload**

### **Test MS-HR-01: Reload with multiple scripts**

**Steps:**

1. Apply hot reload when entity has ScriptA + ScriptB.
2. Inspect call logs.

**Expected:**

* zig_reload called for each script.
* State blocks preserved separately.

---

### **Test MS-HR-02: Reload one script but not the other**

**Steps:**

1. Modify only ScriptA.
2. Trigger hot reload.

**Expected:**

* Only ScriptA rebuilds & reloads.
* ScriptB remains stable with no re-init.
* Entity remains fully functional.

---

### **Test MS-HR-03: One script fails reload**

**Steps:**

1. Break ScriptA (build error).
2. Trigger reload.

**Expected:**

* ScriptA stays on old version.
* ScriptB reloads successfully (if changed).
* No crashing; partial reload is supported.

---

# **4. Stress Tests**

### **Stress 01: Rapid reload spam**

**Steps:**

1. Edit & save script 20 times quickly.

**Expected:**

* Only latest reload applied.
* Old builds canceled or discarded.
* No memory leaks from abandoned libraries.

---

### **Stress 02: Many scripts per entity**

**Steps:**

1. Attach 20 scripts to one entity.
2. Reload all.

**Expected:**

* All scripts reload without performance spikes.
* Execution ordering preserved.
* Memory stable.

---

### **Stress 03: Hundreds of entities**

**Steps:**

1. 1000 entities each with 1–3 scripts.
2. Perform reload.
3. Measure duration.

**Expected:**

* No more than target threshold (e.g. 50–100 ms).
* No frame hitching > allowed.

---

# **5. Failure / Edge Case Tests**

### **Edge 01: Remove script file while engine running**

Expected:

* Graceful error, script remains loaded from memory.

### **Edge 02: Script modifies itself during update**

Expected:

* No crash, reload occurs only after update cycle.

### **Edge 03: Entity destroyed during reload**

Expected:

* Script shutdown is called safely.
* No dangling pointers.

---

# **6. Tooling Required**

* Log capture system
* Instrumentation timestamps
* Memory leak detector
* Build pipeline log listener
* Toggle to simulate ABI mismatch
* ECS debugging inspector

---
