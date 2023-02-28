const TOTAL_BINS = 9000000u;
const BIN_DEPTH = 3u;

// This is a simple compute shader that clears the bin_load buffer to 0 and 
// clears the depth buffer to 1.0.
@binding(0) @group(0) var<storage, read_write> bin_load : array<u32>;
@binding(1) @group(0) var<storage, read_write> depth : array<i32>;


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    if (index >= TOTAL_BINS) {
        return;
    }
    bin_load[index] = 0u;
    for (var i = 0u; i < BIN_DEPTH; i = i + 1u) {
        depth[index * BIN_DEPTH + i] = -1;
    }
}