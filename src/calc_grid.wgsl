
const BIN_SIZE = 1f;
const BIN_DEPTH = 3u;
const BIN_COUNT = 3000u;
// const BOX_SIZE = BIN_SIZE * BIN_COUNT;
const BOX_SIZE = 3000f;

struct Particle {
    x: f32,
    y: f32,
    vel_x: f32,
    vel_y: f32,
    acc_x: f32,
    acc_y: f32,
    color_x: f32,
    color_y: f32,
    color_z: f32,
    type_: f32,
}


@binding(0) @group(0) var<storage, read> particles : array<Particle>;
@binding(1) @group(0) var<storage, read_write> bin_load : array<atomic<u32>>;
@binding(2) @group(0) var<storage, read_write> depth : array<i32>;


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particles);
    if index >= array_length {
        return;
    }

    // get bin index
    let x = (particles[index].x + BOX_SIZE) / 2f;
    let y = (particles[index].y + BOX_SIZE) / 2f;
    let bin_x = u32(floor(x / BIN_SIZE));
    let bin_y = u32(floor(y / BIN_SIZE));
    let bin_index = bin_x + bin_y * BIN_COUNT;

    let depth_index = atomicAdd(&bin_load[bin_index], 1u);

    if (depth_index >= BIN_DEPTH) {
        return;
    }

    let final_index = bin_index * BIN_DEPTH + depth_index;

    depth[final_index] = i32(index);
    // atomicAdd(&depth[final_index], i32(index));
}

