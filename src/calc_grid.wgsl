
// const GRID_SIZE: u32 = 128;

// struct Particle {
//     x: f32,
//     y: f32,
//     vel_x: f32,
//     vel_y: f32,
//     acc_x: f32,
//     acc_y: f32,
//     color_x: f32,
//     color_y: f32,
//     color_z: f32,
//     type_: f32,
// }

// struct Grid {
//     occupancy: atomic<i32>,
// }


// @binding(1) @group(0) var<storage, read> particlesA : array<Particle>;
// @binding(2) @group(0) var<storage, read_write> particlesB : array<Particle>;
// @binding(3) @group(0) var<storage, read_write> grid: Grid;

// fn calculate_cell(x: f32, y: f32) -> u32 {
//     // hash the particle position to a cell
//     let cell_x = floor(x * GRID_SIZE);
//     let cell_y = floor(y * GRID_SIZE);
//     return cell_x + cell_y * GRID_SIZE;
// }

// @compute @workgroup_size(8, 8)
// fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
//     let index = GlobalInvocationID.xyz;
//     let particle = particlesA[index];
//     let cell = calculate_cell(particle.x, particle.y);
//     atomic_add(&grid.occupancy[cell], 1);
// }

