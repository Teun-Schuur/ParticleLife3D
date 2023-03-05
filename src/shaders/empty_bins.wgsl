

// This is a simple compute shader that clears the bin_load buffer to 0 and 
// clears the depth buffer to 1.0.
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

@binding(0) @group(0) var<uniform> params : Params;
@binding(1) @group(0) var<storage, read_write> bin_load : array<u32>;
@binding(2) @group(0) var<storage, read_write> depth : array<i32>;


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    if (index >= arrayLength(&bin_load)) {
        return;
    }
    bin_load[index] = 0u;
    for (var i = 0u; i < params.bin_capacity; i = i + 1u) {
        depth[index * params.bin_capacity + i] = -1;
    }
}