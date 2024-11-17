const std = @import("std");

pub const BlockRegistrationTextureCallback = fn (context: *anyopaque, x: usize, y: usize, r: f32, g: f32, b: f32) void;

pub const BlockRegistrationTexture = fn (size: usize, callback: BlockRegistrationTextureCallback, context: *anyopaque) bool;

pub fn ForAllFaces(comptime T: type) type {
    return struct {
        positive_x: T,
        negative_x: T,
        positive_y: T,
        negative_y: T,
        positive_z: T,
        negative_z: T,
    };
}

pub const BLOCK_FLAGS_IS_FULL_BIT: u8 = 1;
pub const BLOCK_FLAGS_IS_TRANSPARENT_BIT: u8 = 2;

pub const BlockType = struct {
    name: [*:0]const u8,
    requiredTextureSize: usize,
    texture: ForAllFaces(BlockRegistrationTexture),
    flags: ForAllFaces(u8),
};

test "hello world" {
    const dirt = BlockType{
        .name = "Dirt",
        .requiredTextureSize = 8,
        .texture = .{ .positive_x = black, .negative_x = black, .positive_y = black, .negative_y = black, .positive_z = black, .negative_z = black },
        .flags = .{ .positive_x = 0, .negative_x = 0, .positive_y = 0, .negative_y = 0, .positive_z = 0, .negative_z = 0 },
    };
    try std.testing.expect(dirt.name == dirt.name);
}

fn black(size: usize, callback: BlockRegistrationTextureCallback, context: *anyopaque) bool {
    for (0..size) |y| {
        for (0..size) |x| {
            callback(context, x, y, 0.0, 0.0, 0.0);
        }
    }
}
