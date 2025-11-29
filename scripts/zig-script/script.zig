const std = @import("std");

export fn zig_init() void {
    // Called from the engine after the library is loaded and symbols resolved.
    std.debug.print("[zig script] zig_init called\n", .{});
}

export fn zig_update(dt: f32) void {
    // Called every frame with dt (seconds). Keep this short.
    std.debug.print("[zig script] zig_update: dt = {}\n", .{dt});
}

// For testing: an update function with no args; the safest to call from other contexts.
export fn zig_update_void() void {
    std.debug.print("[zig script] zig_update_void called\n", .{});
}
