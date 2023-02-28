

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
@binding(1) @group(0) var<storage, read_write> particles : array<Particle>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {

    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particles);
    if index >= array_length {
        return;
    }
    
    var vPos = vec2<f32>(particles[index].x, particles[index].y);
    var vVel = vec2<f32>(particles[index].vel_x, particles[index].vel_y);
    // var vAcc = vec2<f32>(particles[index].acc_x, particles[index].acc_y);
    var vAcc = vec2<f32>(0.0, -2.0);


    let dt: f32 = params.dt;
    vVel = vVel + vAcc * dt;
    vVel = vVel * (1.0 - params.friction); 
    vPos = vPos + vVel * dt + vAcc * dt * dt * 0.5;


    particles[index].x = vPos.x;
    particles[index].y = vPos.y;
    particles[index].vel_x = vVel.x;
    particles[index].vel_y = vVel.y;
}