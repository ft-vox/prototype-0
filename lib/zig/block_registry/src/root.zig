const std = @import("std");
const types = @import("./types.zig");

test "hello world" {
    const dirt = types.BlockType{
        .name = "Dirt",
        .requiredTextureSize = 8,
        .texture = .{ .positive_x = black, .negative_x = black, .positive_y = black, .negative_y = black, .positive_z = black, .negative_z = black },
        .flags = .{ .positive_x = 0, .negative_x = 0, .positive_y = 0, .negative_y = 0, .positive_z = 0, .negative_z = 0 },
    };
    try std.testing.expect(dirt.name == dirt.name);
}

fn black(size: usize, callback: *const types.BlockRegistrationTextureCallback, context: *anyopaque) callconv(.C) bool {
    for (0..size) |y| {
        for (0..size) |x| {
            callback(context, x, y, 0.0, 0.0, 0.0);
        }
    }
    return false;
}
