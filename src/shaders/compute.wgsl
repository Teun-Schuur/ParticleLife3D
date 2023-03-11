

// a compute shader in wgsl that simulates gravity for all particles

struct Atom{
    size: f32, // in nm
    mass: f32, // in Dalton (1.66053906660e-27 kg)
    charge: i32, // in elementary charge (1.602176634e-19 C)
    sigma: f32, // in nm
    epsilon: f32, // eV (1.602176634e-19 J)
}

struct Params {
    N: u32, // number of particles
    dt: f32,  // in ps
    neghborhood_size: f32, // in nm
    max_force: f32, // in nm * amu / ps^2
    friction: f32,  // in amu / ps
    box_size: f32, // in nm
    bin_size: f32, // in nm
    bin_count: u32,
    bin_capacity: u32,
    align1: u32,
    align2: u32,
    align3: u32,
    helium: Atom,
}

struct Particle {
    x: f32,
    y: f32,
    z: f32,
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,
    acc_x: f32,
    acc_y: f32,
    acc_z: f32,
    color_x: f32,
    color_y: f32,
    color_z: f32,
    type_: f32,
}

struct Stats {
    KE: f32,
    PE: f32,
}

@binding(0) @group(0) var<uniform> params : Params;
@binding(1) @group(0) var<storage, read> particlesA : array<Particle>;
@binding(2) @group(0) var<storage, read_write> particlesB : array<Particle>;
@binding(3) @group(0) var<storage, read> bin_load : array<u32>;
@binding(4) @group(0) var<storage, read> depth : array<i32>;
@binding(5) @group(0) var<storage, read_write> stats : array<Stats>;

// let PARTICLES_TO_CHECK_SIZE: u32 = params.bin_capacity * 9u;



fn wrap_bin(x: i32, y: i32, z: i32) -> u32 {
    var bin_x = x;
    var bin_y = y;
    var bin_z = z;
    let bin_count = i32(params.bin_count);
    if bin_x < 0 {
        bin_x = bin_count + bin_x;
    }
    if bin_x >= bin_count {
        bin_x = bin_x - bin_count;
    }
    if bin_y < 0 {
        bin_y = bin_count + bin_y;
    }
    if bin_y >= bin_count {
        bin_y = bin_y - bin_count;
    }
    if bin_z < 0 {
        bin_z = bin_count + bin_z;
    }
    if bin_z >= bin_count {
        bin_z = bin_z - bin_count;
    }
    return u32(bin_x + bin_y * bin_count + bin_z * bin_count * bin_count);    
}

// fn cap_bin(x: i32, y: i32, z: i32) -> i32 {
//     let bin_count = i32(params.bin_count);
//     if x < 0 || x >= bin_count || y < 0 || y >= bin_count || z < 0 || z >= bin_count {
//         return -1;
//     }
//     return x + y * bin_count + z * bin_count * bin_count;
// }


fn lennard_jones(dist: f32, sigma: f32, epsilon: f32) -> f32 {
    // lennard jones potential (in nm^2 * u * ps^-2)
    let r6 = pow(sigma / dist, 6.0);
    let r12 = r6 * r6;
    return 4.0 * epsilon * (r12 - r6);
}

fn lennard_jones_force(dist: f32, sigma: f32, epsilon: f32) -> f32 {
    let r6 = pow(sigma / dist, 6.0);
    let r12 = r6 * r6;
    // let force = -24.0 * epsilon / sigma * r6inv * (2.0 * r6inv - 1.0);
    let force = 24.0 * epsilon * (2.0 * r12 - r6) / dist;
    return force;
}

fn calc_forces(dist: f32, normal: vec3<f32>) -> vec3<f32> {
    let force = lennard_jones_force(dist, params.helium.sigma, params.helium.epsilon);
    return force * normal;
}


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let index = GlobalInvocationID.x;
    let array_length = arrayLength(&particlesA);
    if index >= array_length {
        return;
    }
    
    var vPos = vec3<f32>(particlesA[index].x, particlesA[index].y, particlesA[index].z);
    var vVel = vec3<f32>(particlesA[index].vel_x, particlesA[index].vel_y, particlesA[index].vel_z);
    var vAcc = vec3<f32>(particlesA[index].acc_x, particlesA[index].acc_y, particlesA[index].acc_z);

    var pe = 0.0;
    var pos: vec3<f32>;
    var vel: vec3<f32>;
    var d: vec3<f32>;
    var dist: f32;
    var normal: vec3<f32>;
    var acc = vec3<f32>(0.0);
    var f: f32;

    let x_temp = (vPos.x + params.box_size) / 2f;
    let y_temp = (vPos.y + params.box_size) / 2f;
    let z_temp = (vPos.z + params.box_size) / 2f;
    let bin_x = i32(floor(x_temp / params.bin_size));
    let bin_y = i32(floor(y_temp / params.bin_size));
    let bin_z = i32(floor(z_temp / params.bin_size));

    for (var x = -1; x <= 1; x += 1) {
        for (var y = -1; y <= 1; y += 1) {
            for (var z = -1; z <= 1; z += 1) {
                let new_x = bin_x + x;
                let new_y = bin_y + y;
                let new_z = bin_z + z;

                // if (new_x < 0 || new_x >= i32(params.bin_count) || new_y < 0 || new_y >= i32(params.bin_count)) {
                    // continue;
                // }
                let bin_index = wrap_bin(new_x, new_y, new_z);
                // let bin_index = u32(new_x) + u32(new_y) * params.bin_count;
                let bin_size = bin_load[bin_index];
                for (var j = 0u; j < bin_size; j += 1u) {
                    let p_index = u32(depth[bin_index*params.bin_capacity + j]);
                    if p_index == index {
                        continue;
                    }

                    pos = vec3<f32>(particlesA[p_index].x, particlesA[p_index].y, particlesA[p_index].z);
                    vel = vec3<f32>(particlesA[p_index].vel_x, particlesA[p_index].vel_y, particlesA[p_index].vel_z);
                    d = vPos - pos;
                    // wrap around
                    if d.x > params.box_size {
                        d.x -= params.box_size*2.0;
                    }
                    if d.x < -params.box_size {
                        d.x += params.box_size*2.0;
                    }
                    if d.y > params.box_size {
                        d.y -= params.box_size*2.0;
                    }
                    if d.y < -params.box_size {
                        d.y += params.box_size*2.0;
                    }
                    if d.z > params.box_size {
                        d.z -= params.box_size*2.0;
                    }
                    if d.z < -params.box_size {
                        d.z += params.box_size*2.0;
                    }

                    dist = length(d);
                    // if dist > params.neghborhood_size {
                    //     continue;
                    // }
                    if dist < params.helium.sigma * 0.35 {
                        dist = params.helium.sigma * 0.35;
                    }
                    normal = d / dist;
                    pe = pe + lennard_jones(dist, params.helium.sigma, params.helium.epsilon) * 0.5;
                    acc = acc + calc_forces(dist, normal) / params.helium.mass;
                }
            }
        }
    }


    // leap frog integration
    // vPos = vPos + vVel * params.dt + vAcc * params.dt * params.dt * 0.5;
    // vVel = vVel + acc * params.dt;
    

    vVel = vVel + (acc + vAcc) * params.dt * 0.5;
    // vPos = vPos + vVel * params.dt + vAcc * params.dt * params.dt * 0.5;

    // vPos = vPos + vVel * params.dt + vAcc * params.dt * params.dt * 0.5;
    // vVel = vVel + acc * params.dt;

    // let vVelHalf = vVel + acc * params.dt * 0.5 + acc * params.dt * params.dt * 0.25 + vAcc * params.dt * params.dt * 0.125;
    // vPos = vPos + vVelHalf * params.dt + vAcc * params.dt * params.dt * 0.5;
    // vVel = vVelHalf + acc * params.dt * 0.5 + acc * params.dt * params.dt * 0.25 + vAcc * params.dt * params.dt * 0.125;

    // // Runge-Kutta
    // let dxdt = vVel + acc * params.dt * 0.5;
    // let dvdt = acc;
    // let dt = params.dt;

    // // Compute intermediate values
    // let k1x = dxdt*dt;
    // let k1v = dvdt*dt;
    // let k2x = (dxdt + 0.5*k1v)*dt;
    // let k2v = (dvdt + 0.5*k1x)*dt;
    // let k3x = (dxdt + 0.5*k2v)*dt;
    // let k3v = (dvdt + 0.5*k2x)*dt;
    // let k4x = (dxdt + k3v)*dt;
    // let k4v = (dvdt + k3x)*dt;
    
    // // Compute final values
    // vPos = vPos + (k1x + 2.0*k2x + 2.0*k3x + k4x)/6.0;
    // vVel = vVel + (k1v + 2.0*k2v + 2.0*k3v + k4v)/6.0;
    

    let ke = 0.5 * dot(vVel, vVel) * params.helium.mass;
    
    if vPos.x < 0.0 {
        vPos.x += params.box_size*2.0;
    }
    if vPos.x > params.box_size {
        vPos.x -= params.box_size*2.0;
    }
    if vPos.y < 0.0 {
        // vPos.y =-params.box_size;
        vPos.y += params.box_size*2.0;
    }
    if vPos.y > params.box_size {
        vPos.y -= params.box_size*2.0;
    }
    if vPos.z < 0.0 {
        vPos.z += params.box_size*2.0;
    }
    if vPos.z > params.box_size {
        vPos.z -= params.box_size*2.0;
    }

    // if vPos.x < -params.box_size {
    //     vPos.x = -params.box_size;
    //     vVel.x = vVel.x * -1.0;
    // }
    // if vPos.x > params.box_size {
    //     vPos.x = params.box_size;
    //     vVel.x = vVel.x * -1.0;
    // }
    // if vPos.y < -params.box_size {
    //     vPos.y = -params.box_size;
    //     vVel.y = vVel.y * -1.0;
    // }
    // if vPos.y > params.box_size {
    //     vPos.y = params.box_size;
    //     vVel.y = vVel.y * -1.0;
    // }

    
    stats[index].KE = ke;
    stats[index].PE = pe;
    particlesB[index].x = vPos.x;
    particlesB[index].y = vPos.y;
    particlesB[index].z = vPos.z;
    particlesB[index].vel_x = vVel.x;
    particlesB[index].vel_y = vVel.y;
    particlesB[index].vel_z = vVel.z;
    particlesB[index].acc_x = acc.x;
    particlesB[index].acc_y = acc.y;
    particlesB[index].acc_z = acc.z;
}