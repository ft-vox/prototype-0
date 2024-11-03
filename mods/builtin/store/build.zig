const std = @import("std");
const common = @import("../../common/build/build.zig");

pub fn build(b: *std.Build) !void {
    common.build(b, "store", "src/root.zig");
}
