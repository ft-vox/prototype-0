const std = @import("std");

const Target = struct { name: []const u8, query: std.Target.Query };

const TARGETS = [_]Target{
    .{ .name = "aarch64_linux", .query = .{ .cpu_arch = .aarch64, .os_tag = .linux } },
    .{ .name = "aarch64_macos", .query = .{ .cpu_arch = .aarch64, .os_tag = .macos } },
    .{ .name = "x86_64_linux", .query = .{ .cpu_arch = .x86_64, .os_tag = .linux } },
    .{ .name = "x86_64_macos", .query = .{ .cpu_arch = .x86_64, .os_tag = .macos } },
    .{ .name = "x86_64_windows", .query = .{ .cpu_arch = .x86_64, .os_tag = .windows } },
};

pub fn common_build(b: *std.Build, lib_name: []const u8, source: []const u8) !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseFast });
    const root_source_file: std.Build.LazyPath = .{ .src_path = .{ .owner = b, .sub_path = source } };

    const lib_step = b.step("lib", "Install executable for all targets");

    for (TARGETS) |TARGET| {
        const name = try std.fmt.allocPrint(allocator, "{s}_{s}", .{ lib_name, TARGET.name });
        const lib = b.addSharedLibrary(.{
            .name = name,
            .target = b.resolveTargetQuery(TARGET.query),
            .optimize = optimize,
            .root_source_file = root_source_file,
        });

        const lib_install = b.addInstallArtifact(lib, .{});
        lib_step.dependOn(&lib_install.step);
    }

    b.default_step.dependOn(lib_step);
}

pub fn build(b: *std.Build) !void {
    try common_build(b, "store", "src/root.zig");
}
