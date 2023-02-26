

// a compute shader in wgsl that simulates gravity for all particles


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

const max_force: f32 = -30.0;

fn attraction(dist: f32, f: f32) -> f32 {
    // attraction is a function of distance

    // global repulsion when distance is less than 1 (linear function of distance)
    if dist <= params.global_repulsion_distance {
        return 1.0 * max_force * (params.global_repulsion_distance - dist) / params.global_repulsion_distance;
    }
    return max_force * f * (1.0 - abs(2.0 * (dist - params.global_repulsion_distance - (params.neghborhood_size * 0.5 - 0.5)) / (params.neghborhood_size - params.global_repulsion_distance)));
}

// fn attraction(dist: f32, f: f32, nhs: f32, nhsi: f32, denom: f32) -> f32 {
//     var out: f32;
//     if dist <= params.global_repulsion_distance {
//         out = max_force * (params.global_repulsion_distance - dist) * nhsi;
//     } else{
//         let numer = dist - params.global_repulsion_distance - nhs;
//         out = max_force * f * (1.0 - numer * denom);
//     }
//     return out;
// }

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
    
    let dt: f32 = params.dt;

    // using varlets algorithm
    vVel = vVel + vAcc * dt;
    vVel = vVel * 0.1;
    vVel = vVel * (1.0 - params.friction); 
    vPos = vPos + vVel * dt + vAcc * dt * dt * 0.5;

    var pos: vec2<f32>;
    var vel: vec2<f32>;
    var d: vec2<f32>;
    var dist: f32;
    var dist_sqrt: f32;
    var normal: vec2<f32>;
    var acc = vec2<f32>(0.0, 0.0);
    var f: f32;

    let nhs = params.neghborhood_size * 0.5 - 0.5;
    let denom = 1.0 / (2.0 * (params.neghborhood_size - params.global_repulsion_distance));
    let nhsi = 1.0 / nhs;

    for (var i = 0u; i < array_length; i++) {
        if i == index {
            continue;
        }

        pos = vec2<f32>(particlesA[i].x, particlesA[i].y);
        vel = vec2<f32>(particlesA[i].vel_x, particlesA[i].vel_y);
        
        // distance between particles in a PBC box
        d = pos - vPos;
        let boxSize = vec2(params.box_size);
        d = d - 2.0 * boxSize * clamp(floor((d + boxSize) / (2.0 * boxSize)), vec2<f32>(-1.0), vec2<f32>(1.0));
        
        dist = length(d);
        
        // attraction
        if dist < params.neghborhood_size {
            normal = d / dist;
            f = params.attraction[vType][u32(particlesA[i].type_ + 0.1)];
            // acc = acc + normal * attraction(dist_sqrt, f, nhs, nhsi, denom);
            acc = acc + normal * attraction(dist, f);
        }
    }

    // clamp acc
    let acc_len = sqrt(dot(acc, acc));
    if acc_len > params.max_force {
        acc = acc * params.max_force / acc_len;
    }

    // wrap around
    let bs_2 = params.box_size * 2.0;
    let bs_2_inv = 1.0 / bs_2;
    vPos -= vec2(
        bs_2 * floor((vPos.x + params.box_size) * bs_2_inv),
        bs_2 * floor((vPos.y + params.box_size) * bs_2_inv)
    );
    
    particlesB[index].x = vPos.x;
    particlesB[index].y = vPos.y;
    particlesB[index].vel_x = vVel.x;
    particlesB[index].vel_y = vVel.y;
    particlesB[index].acc_x = acc.x;
    particlesB[index].acc_y = acc.y;
}