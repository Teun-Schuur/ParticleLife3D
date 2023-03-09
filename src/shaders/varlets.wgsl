

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
    var vAcc = vec2<f32>(particles[index].acc_x, particles[index].acc_y);

    let dt: f32 = params.dt;
    vPos = vPos + vVel * dt + vAcc * dt * dt * 0.5;

    particles[index].x = vPos.x;
    particles[index].y = vPos.y;
}