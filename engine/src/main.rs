use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::path::PathBuf;
use std::{thread, time::Duration};

/// Types for script exports; keep these in sync with the Zig script template.
/// We're using very small forms for clarity — extend these in your real ABI.

type ZigInit = unsafe extern "C" fn();
type ZigUpdate = unsafe extern "C" fn(f32);

fn main() {
    println!("Engine loader prototype — dynamic load Zig script");

    let script_path = PathBuf::from("../scripts/zig-script/libscript.so");

    if !script_path.exists() {
        println!("Script library not found at {:?}. Build the script first:", script_path);
        println!("  cd scripts/zig-script && ./build.sh");
        return;
    }

    // Load library
    unsafe {
        match Library::new(&script_path) {
            Ok(lib) => {
                println!("Loaded library: {:?}", script_path);

                let init: Symbol<ZigInit> = match lib.get(b"zig_init") {
                    Ok(sym) => sym,
                    Err(e) => {
                        eprintln!("Failed to find symbol zig_init: {}", e);
                        return;
                    }
                };

                let update: Symbol<ZigUpdate> = match lib.get(b"zig_update") {
                    Ok(sym) => sym,
                    Err(e) => {
                        eprintln!("Failed to find symbol zig_update: {}", e);
                        return;
                    }
                };

                // Call init once
                println!("Calling zig_init...");
                init();

                // Simulate per-frame update calls for a few frames
                let mut frame: f32 = 0.0;
                for _ in 0..10 {
                    frame += 1.0 / 60.0;
                    println!("Calling zig_update({:.4})", frame);
                    update(frame);
                    thread::sleep(Duration::from_millis(50));
                }

                println!("Finished updates — library will unload when `lib` is dropped");
            }
            Err(e) => {
                eprintln!("Failed to load library {:?}: {}", script_path, e);
            }
        }
    }
}
