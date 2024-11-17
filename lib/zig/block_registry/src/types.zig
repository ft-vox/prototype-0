const std = @import("std");

pub const BlockRegistrationTextureCallback = fn (context: *anyopaque, x: usize, y: usize, r: f32, g: f32, b: f32) callconv(.C) void;

pub const BlockRegistrationTexture = fn (size: usize, callback: *const BlockRegistrationTextureCallback, context: *anyopaque) callconv(.C) bool;

pub fn ForAllFaces(comptime T: type) type {
    return extern struct {
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

pub const BlockType = extern struct {
    name: [*:0]const u8,
    requiredTextureSize: usize,
    texture: ForAllFaces(*const BlockRegistrationTexture),
    flags: ForAllFaces(u8),
};
