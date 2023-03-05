

struct Atom{
    size: f32, // in nm
    mass: f32, // in Dalton (1.66053906660e-27 kg)
    charge: i32, // in elementary charge (1.602176634e-19 C)
    sigma: f32, // in nm
    epsilon: f32, // eV (1.602176634e-19 J)
}

struct Params {
    dt: f32,  // in ps
    neghborhood_size: f32, // in nm
    max_force: f32, // in nm * amu / ps^2
    friction: f32,  // in amu / ps
    box_size: f32, // in nm
    bin_size: f32, // in nm
    bin_count: u32,
    bin_capacity: u32,
    helium: Atom,
}

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

@binding(0) @group(0) var<uniform> params : Params;
@binding(1) @group(0) var<storage, read> particles : array<Particle>;
@binding(2) @group(0) var<storage, read_write> bin_load : array<atomic<u32>>;
@binding(3) @group(0) var<storage, read_write> depth : array<i32>;


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particles);
    if index >= array_length {
        return;
    }

    // get bin index
    let x = (particles[index].x + params.box_size) / 2f;
    let y = (particles[index].y + params.box_size) / 2f;
    let bin_x = u32(floor(x / params.bin_size));
    let bin_y = u32(floor(y / params.bin_size));
    let bin_index = bin_x + bin_y * params.bin_count;

    let depth_index = atomicAdd(&bin_load[bin_index], 1u);

    if (depth_index >= params.bin_capacity) {
        return;
    }

    let final_index = bin_index * params.bin_capacity + depth_index;

    depth[final_index] = i32(index);
    // atomicAdd(&depth[final_index], i32(index));
}

