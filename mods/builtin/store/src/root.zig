const std = @import("std");

const Node = struct {
    key: []const u8,
    value: ?*anyopaque,
    deleteValue: *const fn (?*anyopaque) void,
    left: ?*Node = null,
    right: ?*Node = null,
    height: u8 = 1,
};

const TMap = struct {
    allocator: *const std.mem.Allocator,
    root: ?*Node = null,

    pub fn new(allocator: *const std.mem.Allocator) !*TMap {
        const map = try allocator.create(TMap);
        map.* = TMap{
            .allocator = allocator,
            .root = null,
        };
        return map;
    }

    fn getHeight(node: ?*Node) u8 {
        return if (node) |real_node| real_node.height else 0;
    }

    fn getBalanceFactor(node: ?*Node) i8 {
        if (node) |real_node| {
            const left: i8 = @intCast(getHeight(real_node.left));
            const right: i8 = @intCast(getHeight(real_node.right));
            return left - right;
        } else {
            return 0;
        }
    }

    fn rotateRight(y: *Node) *Node {
        const x = y.left.?;
        const T2 = x.right;

        x.right = y;
        y.left = T2;

        y.height = 1 + @max(getHeight(y.left), getHeight(y.right));
        x.height = 1 + @max(getHeight(x.left), getHeight(x.right));

        return x;
    }

    fn rotateLeft(x: *Node) *Node {
        const y = x.right.?;
        const T2 = y.left;

        y.left = x;
        x.right = T2;

        x.height = 1 + @max(getHeight(x.left), getHeight(x.right));
        y.height = 1 + @max(getHeight(y.left), getHeight(y.right));

        return y;
    }

    fn balanceNode(node: *Node) *Node {
        const balance = getBalanceFactor(node);

        if (balance > 1) {
            if (getBalanceFactor(node.left) < 0) {
                node.left = rotateLeft(node.left.?);
            }
            return rotateRight(node);
        }

        if (balance < -1) {
            if (getBalanceFactor(node.right) > 0) {
                node.right = rotateRight(node.right.?);
            }
            return rotateLeft(node);
        }

        return node;
    }

    fn compareKeys(nodeKey: []const u8, newKey: []const u8, index: usize) struct { order: std.math.Order, idx: usize } {
        var idx = index;
        while (idx < nodeKey.len and idx < newKey.len and nodeKey[idx] == newKey[idx]) {
            idx += 1;
        }
        const order = if (idx == nodeKey.len and idx == newKey.len)
            std.math.Order.eq
        else if (idx == nodeKey.len)
            std.math.Order.lt
        else if (idx == newKey.len)
            std.math.Order.gt
        else
            std.mem.order(u8, nodeKey[idx..], newKey[idx..]);
        return .{ .order = order, .idx = idx };
    }

    fn insertNode(
        self: *TMap,
        node: ?*Node,
        key: []const u8,
        value: ?*anyopaque,
        deleteValue: *const fn (?*anyopaque) void,
        index: usize,
        inserted: *bool,
    ) !?*Node {
        if (node == null) {
            const newNode = try self.allocator.create(Node);
            newNode.* = Node{
                .key = try self.allocator.dupe(u8, key),
                .value = value,
                .deleteValue = deleteValue,
                .left = null,
                .right = null,
                .height = 1,
            };
            inserted.* = true;
            return newNode;
        }

        const cmp_result = compareKeys(node.?.key, key, index);
        const cmp = cmp_result.order;
        const new_index = cmp_result.idx;

        if (cmp == std.math.Order.gt) {
            node.?.left = try insertNode(self, node.?.left, key, value, deleteValue, new_index, inserted);
        } else if (cmp == std.math.Order.lt) {
            node.?.right = try insertNode(self, node.?.right, key, value, deleteValue, new_index, inserted);
        } else {
            inserted.* = false;
            return node;
        }

        node.?.height = 1 + @max(getHeight(node.?.left), getHeight(node.?.right));
        return balanceNode(node.?);
    }

    pub fn insert(
        self: *TMap,
        key: []const u8,
        value: ?*anyopaque,
        deleteValue: *const fn (?*anyopaque) void,
    ) !bool {
        var inserted: bool = false;
        self.root = try insertNode(self, self.root, key, value, deleteValue, 0, &inserted);
        return inserted;
    }

    pub fn search(self: *TMap, key: []const u8) ?*anyopaque {
        var current = self.root;
        var index: usize = 0;
        while (current != null) {
            const cmp_result = compareKeys(current.?.key, key, index);
            const cmp = cmp_result.order;
            const new_index = cmp_result.idx;
            if (cmp == std.math.Order.eq) {
                return current.?.value;
            } else if (cmp == std.math.Order.gt) {
                current = current.?.left;
            } else {
                current = current.?.right;
            }
            index = new_index;
        }
        return null;
    }

    fn deleteNode(self: *TMap, node: ?*Node) void {
        if (node == null) return;
        deleteNode(self, node.?.left);
        deleteNode(self, node.?.right);
        self.allocator.free(node.?.key);
        node.?.deleteValue(node.?.value);
        self.allocator.destroy(node.?);
    }

    pub fn deinit(self: *TMap) void {
        deleteNode(self, self.root);
        self.allocator.destroy(self);
    }

    pub export fn TMap_insert_c(map: *TMap, key: []const u8, value: ?*anyopaque, deleteValue: fn (value: ?*anyopaque) void) bool {
        switch (map.insert(key, value, deleteValue)) {
            error.OutOfMemory => return true,
            error.DuplicateKey => return true,
            else => |inserted| return !inserted,
        }
    }
};

fn createTestValue(allocator: *const std.mem.Allocator, key: []const u8) ![]u8 {
    const value = try allocator.alloc(u8, key.len + 1);
    std.mem.copyForwards(u8, value[0..key.len], key);
    value[key.len] = 0; // Null-terminate the string
    return value;
}

fn doNothing(_: ?*anyopaque) void {}

test "TMap basic operations" {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = &arena.allocator();

    var map = try TMap.new(allocator);

    const value1 = (try createTestValue(allocator, "value1")).ptr;
    const value2 = (try createTestValue(allocator, "value1")).ptr;
    const value3 = (try createTestValue(allocator, "value1")).ptr;

    _ = try map.insert("key1", value1, doNothing);
    _ = try map.insert("key2", value2, doNothing);
    _ = try map.insert("key3", value3, doNothing);

    try std.testing.expect(map.search("key1") != null);
    try std.testing.expect(map.search("key2") != null);
    try std.testing.expect(map.search("key3") != null);

    const insert_duplicate = try map.insert("key1", (try createTestValue(allocator, "duplicate")).ptr, doNothing);
    try std.testing.expect(insert_duplicate == false);

    const val1 = map.search("key1");
    const val2 = map.search("key2");
    const val3 = map.search("key3");

    try std.testing.expect(val1 == @as(?*anyopaque, value1));
    try std.testing.expect(val2 == @as(?*anyopaque, value2));
    try std.testing.expect(val3 == @as(?*anyopaque, value3));

    const val4 = map.search("nonexistent");
    try std.testing.expect(val4 == null);

    map.deinit();
}
