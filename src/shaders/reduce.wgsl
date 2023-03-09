
struct Stats {
    KE: f32,
    PE: f32,
};

@binding(0) @group(0) var<storage, read> stats_in : array<Stats>;
@binding(1) @group(0) var<storage, read_write> stats_out : array<Stats>;
@binding(2) @group(0) var<storage, read_write> final_: Stats;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    if (index >= arrayLength(&stats_out)) {
        return;
    }
    let KE = stats_in[index * 2u].KE + stats_in[index * 2u + 1u].KE;
    let PE = stats_in[index * 2u].PE + stats_in[index * 2u + 1u].PE;
    stats_out[index].KE = KE;
    stats_out[index].PE = PE;
    if (index == 0u) {
        final_.KE = KE;
        final_.PE = PE;
    }
}