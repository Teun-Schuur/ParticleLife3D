
// struct Stats {
//     KE: f32;
//     PE: f32;
// };

@binding(0) @group(0) var<storage, read> energies_in : array<f32>;
@binding(1) @group(0) var<storage, read_write> energies_out : array<f32>;
@binding(2) @group(0) var<storage, read_write> final_: f32;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    if (index >= arrayLength(&energies_out)) {
        return;
    }
    let sum = energies_in[index * 2u] + energies_in[index * 2u + 1u];
    energies_out[index] = sum;
    if (index == 0u) {
        final_ = sum;
    }
}