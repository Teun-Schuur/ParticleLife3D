

// a compute shader in wgsl that simulates gravity for all particles

const BIN_SIZE = 1f;
const BIN_DEPTH = 3u;
const BIN_COUNT = 3000u;
// const BOX_SIZE = BIN_SIZE * BIN_COUNT;
const BOX_SIZE = 3000f;

struct Params {                        // align(4) size(64)
    dt: f32,                            // align(4) size(4)
    neghborhood_size: f32,              // align(4) size(4)
    max_force: f32,                  // align(4) size(4)
    friction: f32,                      // align(4) size(4)
    global_repulsion_distance: f32,     // align(4) size(4)
    box_size: f32,                      // align(4) size(4)
    attraction: mat4x4<f32>,        // align(4) size(48)
    // one_one: f32,                      // align(4) size(4)
    // one_one: f32,
    // one_two: f32,
    // one_three: f32,
    // two_one: f32,
    // two_two: f32,
    // two_three: f32,
    // three_one: f32,
    // three_two: f32,
    // three_three: f32,
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
@binding(1) @group(0) var<storage, read> particlesA : array<Particle>;
@binding(2) @group(0) var<storage, read_write> particlesB : array<Particle>;
@binding(3) @group(0) var<storage, read> bin_load : array<u32>;
@binding(4) @group(0) var<storage, read> depth : array<i32>;

const max_force: f32 = -30.0;

fn attraction(dist: f32, f: f32) -> f32 {
    // attraction is a function of distance

    // global repulsion when distance is less than 1 (linear function of distance)
    if dist <= params.global_repulsion_distance {
        return 1.0 * max_force * (params.global_repulsion_distance - dist) / params.global_repulsion_distance;
    }
    return max_force * f * (1.0 - abs(2.0 * (dist - params.global_repulsion_distance - (params.neghborhood_size * 0.5 - 0.5)) / (params.neghborhood_size - params.global_repulsion_distance)));
}

fn calculate_cell(x: f32, y: f32) -> u32 {
    // hash the particle position to a cell
    let x_n = (x + BOX_SIZE) / 2f;
    let y_n = (y + BOX_SIZE) / 2f;
    let bin_x = u32(floor(x_n / BIN_SIZE));
    let bin_y = u32(floor(y_n / BIN_SIZE));
    let bin_index = bin_x + bin_y * BIN_COUNT;
}

fn wrap_bin(x: i32, y: i32) -> u32 {
    // wrap the bin index to the box size (without i32 % i32)
    // let bin_x = x % i32(BIN_COUNT);
    // let bin_y = y % i32(BIN_COUNT);
    // let bin_index = bin_x + bin_y * i32(BIN_COUNT);
    // if bin_index < 0 {
    //     return u32(i32(BIN_COUNT) + bin_index);
    // }
    // return u32(bin_index);
    var bin_x = x;
    var bin_y = y;
    if bin_x < 0 {
        bin_x = i32(BIN_COUNT) + bin_x;
    }
    if bin_x >= i32(BIN_COUNT) {
        bin_x = bin_x - i32(BIN_COUNT);
    }
    if bin_y < 0 {
        bin_y = i32(BIN_COUNT) + bin_y;
    }
    if bin_y >= i32(BIN_COUNT) {
        bin_y = bin_y - i32(BIN_COUNT);
    }
    return u32(bin_x + bin_y * i32(BIN_COUNT));
    
}

fn calc_forces(pos: vec2<f32>, d: vec2<f32>, dist: f32, normal: vec2<f32>, f: f32) -> vec2<f32> {
    // calculate the forces on the particle
    return vec2<f32>(0.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    // particle life
    // HELUP!!! I think something is wrong with the shader, because when I run it now it runs, but when I unncomment "var dt: f32 = params.dt;"
    // anwser: 

    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particlesA);
    if index >= array_length {
        return;
    }
    
    var vPos = vec2<f32>(particlesA[index].x, particlesA[index].y);
    var vVel = vec2<f32>(particlesA[index].vel_x, particlesA[index].vel_y);
    var vAcc = vec2<f32>(particlesA[index].acc_x, particlesA[index].acc_y);
    let vType = u32(particlesA[index].type_ + 0.1);
    


    var pos: vec2<f32>;
    var vel: vec2<f32>;
    var d: vec2<f32>;
    var dist: f32;
    var normal: vec2<f32>;
    var acc = vec2<f32>(0.0, 0.0);
    var f: f32;

    // let nhs = params.neghborhood_size * 0.5 - 0.5;
    // let denom = 1.0 / (2.0 * (params.neghborhood_size - params.global_repulsion_distance));
    // let nhsi = 1.0 / nhs;

    let x_temp = (vPos.x + BOX_SIZE) / 2f;
    let y_temp = (vPos.y + BOX_SIZE) / 2f;
    let bin_x = i32(floor(x_temp / BIN_SIZE));
    let bin_y = i32(floor(y_temp / BIN_SIZE));

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let bin_index = wrap_bin(bin_x + x, bin_y + y);
            let depth_index = min(bin_load[bin_index], BIN_DEPTH);

            var overlap = vec2<f32>(0.0);
            var count = 0;
            var new_vel = vec2<f32>(0.0);
            for (var z = 0u; z < depth_index; z++) {
                let p_index_index = bin_index * BIN_DEPTH + z;
                let p_index: i32 = depth[p_index_index];

                if u32(p_index) == index {
                    continue;
                }
                pos = vec2<f32>(particlesA[u32(p_index)].x, particlesA[u32(p_index)].y);
                vel = vec2<f32>(particlesA[u32(p_index)].vel_x, particlesA[u32(p_index)].vel_y);
                
                // distance between particles in a PBC box
                d = vPos - pos;
                // let boxSize = vec2(params.box_size);
                let boxSize = vec2(BOX_SIZE);
                d = d - 2.0 * boxSize * clamp(floor((d + boxSize) / (2.0 * boxSize)), vec2<f32>(-1.0), vec2<f32>(1.0));
                dist = length(d);
                normal = d / dist;
                // f = params.attraction[vType][u32(particlesA[u32(p_index)].type_ + 0.1)];

                // acc = acc + calc_forces(pos, d, dist, normal, f);

                // elastic collision (radius = 1.0, mass = 1.0)

                if dist < 2.0 {
                    // Calculate the velocities of each particle in the normal and tangent directions
                    let tangent = vec2<f32>(-normal.y, normal.x);
                    let v1n = dot(normal, vVel);
                    let v1t = dot(tangent, vVel);
                    let v2n = dot(normal, vel);
                    let v2t = dot(tangent, vel);

                    // Calculate the new velocities of each particle in the normal direction
                    let v1n_new = v2n;
                    let v1t_new = v1n;

                    new_vel = new_vel + v1n_new * normal + v1t_new * tangent;

                    // correct position
                    overlap = overlap + (2.0 - dist) * normal * 0.5;
                    count = count + 1;
                }
            }
            if count > 0 {
                vPos = vPos + overlap / f32(count);
                // vVel = new_vel / f32(count);
            }
        }
    }


    if vPos.x < 0.0 {
        vPos.x += BOX_SIZE*2.0;
    }
    if vPos.x > BOX_SIZE {
        vPos.x -= BOX_SIZE*2.0;
    }
    if vPos.y <-BOX_SIZE {
        vPos.y =-BOX_SIZE;
        // vPos.y += BOX_SIZE*2.0;
    }
    if vPos.y > BOX_SIZE {
        vPos.y -= BOX_SIZE*2.0;
    }


    particlesB[index].x = vPos.x;
    particlesB[index].y = vPos.y;
    particlesB[index].vel_x = vVel.x;
    particlesB[index].vel_y = vVel.y;
    particlesB[index].acc_x = acc.x;
    particlesB[index].acc_y = acc.y;
}