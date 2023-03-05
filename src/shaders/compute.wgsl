

// a compute shader in wgsl that simulates gravity for all particles

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
@binding(1) @group(0) var<storage, read> particlesA : array<Particle>;
@binding(2) @group(0) var<storage, read_write> particlesB : array<Particle>;
@binding(3) @group(0) var<storage, read> bin_load : array<u32>;
@binding(4) @group(0) var<storage, read> depth : array<i32>;
@binding(5) @group(0) var<storage, read_write> energies : array<f32>;

// let PARTICLES_TO_CHECK_SIZE: u32 = params.bin_capacity * 9u;


fn calculate_cell(x: f32, y: f32) -> u32 {
    // hash the particle position to a cell
    let x_n = (x + params.box_size) / 2f;
    let y_n = (y + params.box_size) / 2f;
    let bin_x = u32(floor(x_n / params.bin_size));
    let bin_y = u32(floor(y_n / params.bin_size));
    let bin_index = bin_x + bin_y * params.bin_count;
}

fn wrap_bin(x: i32, y: i32) -> u32 {
    var bin_x = x;
    var bin_y = y;
    if bin_x < 0 {
        bin_x = i32(params.bin_count) + bin_x;
    }
    if bin_x >= i32(params.bin_count) {
        bin_x = bin_x - i32(params.bin_count);
    }
    if bin_y < 0 {
        bin_y = i32(params.bin_count) + bin_y;
    }
    if bin_y >= i32(params.bin_count) {
        bin_y = bin_y - i32(params.bin_count);
    }
    return u32(bin_x + bin_y * i32(params.bin_count));    
}

fn cap_bin(x: i32, y: i32) -> i32 {
    if x < 0 || x >= i32(params.bin_count) || y < 0 || y >= i32(params.bin_count) {
        return -1;
    }
    return x + y * i32(params.bin_count);
}


fn lennard_jones(dist: f32, sigma: f32, epsilon: f32) -> f32 {
    // lennard jones potential (in nm^2 * u * ps^-2)
    let r6 = pow(sigma / dist, 6.0);
    let r12 = r6 * r6;
    return 4.0 * epsilon * (r12 - r6);
}

fn lennard_jones_force(dist: f32, sigma: f32, epsilon: f32) -> f32 {
    // lennard jones potential derivative (in nm * u * ps^-2)
    // let r6 = pow(sigma / dist, 6.0);
    // let r12 = r6 * r6;
    // return -24.0 * epsilon * (r6 - 2.0 * r12) / dist;

    let r6 = pow(sigma / dist, 6.0);
    let r12 = r6 * r6;
    // let force = -24.0 * epsilon / sigma * r6inv * (2.0 * r6inv - 1.0);
    let force = 24.0 * epsilon * (2.0 * r12 - r6) / dist;
    return force;
}

fn calc_forces(dist: f32, normal: vec2<f32>) -> vec2<f32> {
    // calculate the forces on the particle
    let force = lennard_jones_force(dist, params.helium.sigma, params.helium.epsilon);
    // force = clamp(force, -params.max_force, params.max_force);
    return force * normal;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particlesA);
    if index >= array_length {
        return;
    }
    
    var vPos = vec2<f32>(particlesA[index].x, particlesA[index].y);
    var vVel = vec2<f32>(particlesA[index].vel_x, particlesA[index].vel_y);
    var vAcc = vec2<f32>(particlesA[index].acc_x, particlesA[index].acc_y);

    var energy = 0.0;
    var pos: vec2<f32>;
    var vel: vec2<f32>;
    var d: vec2<f32>;
    var dist: f32;
    var normal: vec2<f32>;
    var acc = vec2<f32>(0.0);
    var f: f32;

    let x_temp = (vPos.x + params.box_size) / 2f;
    let y_temp = (vPos.y + params.box_size) / 2f;
    let bin_x = i32(floor(x_temp / params.bin_size));
    let bin_y = i32(floor(y_temp / params.bin_size));

    // for (var i = 0u; i < array_length; i++) {
    //     if i == index {
    //         continue;
    //     }
    //     pos = vec2<f32>(particlesA[i].x, particlesA[i].y);
    //     vel = vec2<f32>(particlesA[i].vel_x, particlesA[i].vel_y);
    //     d = pos - vPos;
    //     dist = length(d);
    //     normal = d / dist;
    //     energy = energy + lennard_jones(dist, params.helium.sigma, params.helium.epsilon);
    //     acc = acc + calc_forces(dist, normal) / params.helium.mass;
    // }

    for (var x = -1; x <= 1; x += 1) {
        for (var y = -1; y <= 1; y += 1) {
            let new_x = bin_x + x;
            let new_y = bin_y + y;
            if (new_x < 0 || new_x >= i32(params.bin_count) || new_y < 0 || new_y >= i32(params.bin_count)) {
                continue;
            }
            let bin_index = u32(new_x) + u32(new_y) * params.bin_count;
            let bin_size = bin_load[bin_index];
            for (var j = 0u; j < bin_size; j += 1u) {
                let p_index = u32(depth[bin_index*params.bin_capacity + j]);
                if p_index == index {
                    continue;
                }

                pos = vec2<f32>(particlesA[p_index].x, particlesA[p_index].y);
                vel = vec2<f32>(particlesA[p_index].vel_x, particlesA[p_index].vel_y);
                d = vPos - pos;
                dist = length(d);
                normal = d / dist;
                energy = energy + lennard_jones(dist, params.helium.sigma, params.helium.epsilon) * 0.5;
                acc = acc + calc_forces(dist, normal) / params.helium.mass;

            }
        }
    }


    // if vPos.x < 0.0 {
    //     vPos.x += params.box_size*2.0;
    // }
    // if vPos.x > params.box_size {
    //     vPos.x -= params.box_size*2.0;
    // }
    // if vPos.y < 0.0 {
    //     // vPos.y =-params.box_size;
    //     vPos.y += params.box_size*2.0;
    // }
    // if vPos.y > params.box_size {
    //     vPos.y -= params.box_size*2.0;
    // }
    // acc = acc + vec2<f32>(0.0, -0.02);
    vPos = vPos + vVel * params.dt + vAcc * params.dt * params.dt * 0.5;
    vVel = vVel + (acc + vAcc) * params.dt * 0.5;
    energy = energy + 0.5 * dot(vVel, vVel) * params.helium.mass;

    if vPos.x < -params.box_size {
        vPos.x = -params.box_size;
        vVel.x = vVel.x * -1.0;
    }
    if vPos.x > params.box_size {
        vPos.x = params.box_size;
        vVel.x = vVel.x * -1.0;
    }
    if vPos.y < -params.box_size {
        vPos.y = -params.box_size;
        vVel.y = vVel.y * -1.0;
    }
    if vPos.y > params.box_size {
        vPos.y = params.box_size;
        vVel.y = vVel.y * -1.0;
    }

    
    energies[index] = energy;
    particlesB[index].x = vPos.x;
    particlesB[index].y = vPos.y;
    particlesB[index].vel_x = vVel.x;
    particlesB[index].vel_y = vVel.y;
    particlesB[index].acc_x = acc.x;
    particlesB[index].acc_y = acc.y;
}