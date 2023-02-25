

// a compute shader in wgsl that simulates gravity for all particles

struct SimParams {
    dt: f32,
    neghborhood_size: f32,
    max_velocity: f32,
    friction: f32,
    type_one: vec3<f32>,
    type_two: vec3<f32>,
    type_three: vec3<f32>,
    // _padding: vec3<f32>,
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
    // _padding1: f32,
    // _padding2: f32,
}

@binding(0) @group(0) var<uniform> params : SimParams;
@binding(1) @group(0) var<storage, read> particlesA : array<Particle>;
@binding(2) @group(0) var<storage, read_write> particlesB : array<Particle>;

fn dist(d: vec2<f32>) -> f32 {
    return sqrt(d.x * d.x + d.y * d.y);
}

fn fast_dist(d: vec2<f32>) -> f32 {
    // this is a fast approximation of dist
    var x = abs(d.x);
    var y = abs(d.y);
    var min = min(x, y);
    var max = max(x, y);
    return max + 0.4142135623730950488016887242097 * min;
}

fn more_acurate_dist(d: vec2<f32>) -> f32 {
    // this is a more acurate approximation of dist
    let eps: f32 = 0.0001;
    var a: f32 = d.x*d.x + d.y*d.y;
    var x: f32 = fast_dist(d);
    while abs(x * x - a) > eps {
        let u = a/x;
        x = (x + u) / 2.0;
    }
    return x;
}


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    // particle life

    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particlesA);
    if index >= array_length {
        return;
    }

    var vPos = vec2<f32>(particlesA[index].x, particlesA[index].y);
    var vVel = vec2<f32>(particlesA[index].vel_x, particlesA[index].vel_y);
    var vAcc = vec2<f32>(particlesA[index].acc_x, particlesA[index].acc_y);
    var vType = u32(particlesA[index].type_);
    
    var dt = 0.01;

    // // using varlets algorithm
    vVel = vVel + vAcc * dt;
    vPos = vPos + vVel * dt + vAcc * dt * dt / 2.0;

    var pos: vec2<f32>;
    var vel: vec2<f32>;
    var d: vec2<f32>;
    var dist: f32;
    var dist_sqrt: f32;
    var normal: vec2<f32>;
    var force: f32;
    var acc = vec2<f32>(0.0, 0.0);

    for (var i = 0u; i < array_length; i++) {
        if i == index {
            continue;
        }
        d = pos - vPos;
        dist_sqrt = d.x * d.x + d.y * d.y;
        if dist_sqrt > 0.0 {
            continue;
        }

        pos = vec2<f32>(particlesA[i].x, particlesA[i].y);
        vel = vec2<f32>(particlesA[i].vel_x, particlesA[i].vel_y);

        dist = dist(d);
        normal = d / dist;

    }
    
    particlesB[index].x = vPos.x;
    particlesB[index].y = vPos.y;
    particlesB[index].vel_x = vVel.x;
    particlesB[index].vel_y = vVel.y;
    particlesB[index].acc_x = acc.x;
    particlesB[index].acc_y = acc.y;
}
