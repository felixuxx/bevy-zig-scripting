use bevy::prelude::*;
use bevy::app::AppExit;
use std::io::Write;
use libloading::Library;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::path::PathBuf;

/// Types for script exports; keep these in sync with the Zig script template.
type ZigInit = unsafe extern "C" fn();
type ZigUpdate = unsafe extern "C" fn(f32);
type ZigUpdateVoid = unsafe extern "C" fn();

/// Resource that stores the function pointers. We keep the library leaked to ensure the
/// function pointers remain valid while running the program.
#[derive(bevy::prelude::Resource)]
struct ScriptFns {
    init: ZigInit,
    update: ZigUpdate,
}

#[derive(Resource)]
struct FrameCounter(u32);

#[derive(Resource)]
struct MainThreadMarker;


fn main() {
    println!("[engine] main starting");
    use std::io::Write;
    std::io::stdout().flush().ok();
    App::new()
        .add_plugins(MinimalPlugins)
        // Make sure systems requiring `NonSend<MainThreadMarker>` run on the main thread.
        .insert_non_send_resource(MainThreadMarker)
        .add_systems(Startup, load_script_system)
        .add_systems(Update, script_update_system)
        .run();
}

fn get_script_path() -> PathBuf {
    PathBuf::from("../scripts/zig-script/libscript.so")
}

/// Startup system: load the script and insert function pointers as a resource.
fn load_script_system(mut commands: Commands) {
    let script_path = get_script_path();
    if !script_path.exists() {
        error!("Script library not found at {:?}. Build the script first:", script_path);
        error!("  cd scripts/zig-script && sh build.sh");
        return;
    }

    println!("[engine] startup system entered");
    std::io::stdout().flush().ok();
    unsafe {
        println!("[engine] attempting to load library at {:?}", script_path);
        std::io::stdout().flush().ok();
        match Library::new(&script_path) {
            Ok(lib) => {
                println!("[engine] Loaded library: {:?}", script_path);

                // Leak the library so it is not dropped for the lifetime of the program.
                // This keeps function pointers valid. This is a cheap prototype approach.
                let boxed = Box::new(lib);
                // Safety: We leak intentionally to avoid Send/Sync requirements on Library.
                let static_lib: &'static Library = Box::leak(boxed);

                let init_sym: libloading::Symbol<'static, ZigInit> =
                    match static_lib.get(b"zig_init") {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to find symbol zig_init: {}", e);
                            return;
                        }
                    };

                let update_sym: libloading::Symbol<'static, ZigUpdate> =
                    match static_lib.get(b"zig_update") {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to find symbol zig_update: {}", e);
                            return;
                        }
                    };

                // deref to raw fn pointers
                let init_fn: ZigInit = *init_sym;
                let update_fn: ZigUpdate = *update_sym;

                println!("[engine] calling zig_init");
                std::io::stdout().flush().ok();
                init_fn();
                println!("[engine] zig_init returned");
                std::io::stdout().flush().ok();

                commands.insert_resource(ScriptFns {
                    init: init_fn,
                    update: update_fn,
                });
                // Add a counter resource to stop after a few frames for the prototype.
                commands.insert_resource(FrameCounter(0));
                println!("[engine] calling zig_update from startup system");
                std::io::stdout().flush().ok();
                (update_fn)(1.0 / 60.0);
                println!("[engine] returned from zig_update in startup");
                std::io::stdout().flush().ok();
            }
            Err(e) => {
                error!("Failed to load library {:?}: {}", script_path, e);
            }
        }
    }
}

/// Per-frame system: call the Zig update function using Bevy's Time dt.
fn script_update_system(script_fns: Option<Res<ScriptFns>>, _marker: NonSend<MainThreadMarker>, mut counter: Option<ResMut<FrameCounter>>, mut exit: EventWriter<AppExit>) {
    if let Some(fns_res) = script_fns {
        let fns: &ScriptFns = &*fns_res;
        // Be careful, calling into script may throw panics — wrap in `unsafe`.
        unsafe {
            println!("[engine] About to call the Zig update (fn pointer)");
            // Guard with a mutex to prevent possible re-entrancy / concurrent calls into native code.
            static UPDATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
            let _guard = UPDATE_LOCK.lock().unwrap();
            println!("[engine] update fn pointer = {:#x}", fns.update as usize);
            // Try calling the void variant first from a fresh load as a cross-check.
            let lib_path = get_script_path();
            if let Ok(lib2) = libloading::Library::new(&lib_path) {
                if let Ok(sym2) = lib2.get::<ZigUpdateVoid>(b"zig_update_void") {
                    let f2: ZigUpdateVoid = *sym2;
                    println!("[engine] About to call the Zig update (void, fresh load)");
                    f2();
                    println!("[engine] Called the Zig update (void, fresh load)");
                } else {
                    println!("[engine] Failed to locate zig_update_void in fresh load");
                }
            } else {
                println!("[engine] Failed to fresh load the library for void test");
            }
            (fns.update)(1.0 / 60.0);
            println!("[engine] Called the Zig update (fn pointer)");

            // As a test: attempt to load the library on-demand and call its `zig_update` symbol.
            // This helps determine whether storing the function pointer is the issue.
            let lib_path = get_script_path();
            match libloading::Library::new(&lib_path) {
                Ok(lib) => {
                    match lib.get::<ZigUpdate>(b"zig_update") {
                        Ok(sym) => {
                            let f: ZigUpdate = *sym;
                            println!("[engine] About to call the Zig update (fresh load)");
                            f(1.0 / 60.0);
                            println!("[engine] Called the Zig update (fresh load)");
                        }
                        Err(e) => {
                            println!("[engine] Failed to get zig_update symbol from fresh load: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("[engine] Fresh library load failed: {}", e);
                }
            }
            // Attempt a void update call variant as well
            match libloading::Library::new(&lib_path) {
                Ok(lib2) => {
                    match lib2.get::<ZigUpdateVoid>(b"zig_update_void") {
                        Ok(sym2) => {
                            let f2: ZigUpdateVoid = *sym2;
                            println!("[engine] About to call the Zig update (void, fresh load)");
                            f2();
                            println!("[engine] Called the Zig update (void, fresh load)");
                        }
                        Err(e) => println!("[engine] Failed to find zig_update_void: {}", e),
                    }
                }
                Err(e) => println!("[engine] Fresh lib load 2 failed: {}", e),
            }
        }
    }
    if let Some(mut c) = counter {
        c.0 += 1;
        if c.0 > 10 {
            info!("Reached frame limit, quitting.");
            exit.send(AppExit);
        }
    }
}
