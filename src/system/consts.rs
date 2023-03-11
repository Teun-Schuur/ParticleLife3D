/*
time: ps
distance: nm
mass: amu
charge: e
temperature: K
force: eV / nm or nm * amu / ps^2 or 1.660539040 pN

 */


pub const NUMBER_PARTICLES: u32 = 10000;
pub const NUMBER_PARTICLES_CUBED: f32 = 21.544346900318832;
pub const PARTICLE_SIZE: f32 = 0.2551; // in nm
pub const NEIGHBORHOOD_SIZE: f32 = PARTICLE_SIZE * 2.5; // in nm
pub const INIT_TEMPERATURE: f32 = 10.0; // in Kelvin
pub const INIT_SPACING: f32 = PARTICLE_SIZE * 1.0; // in nm
pub const EXES_SPACING: f32 = PARTICLE_SIZE * 3.0; // in nm

pub const DT: f32 = 1.0e-3; // in picoseconds
pub const ITERATIONS: u32 = 31; 
pub const BIN_DEPTH: u32 = 100;
pub const BIN_SIZE: f32 = NEIGHBORHOOD_SIZE;

const TRUE_BOX_SIZE: u32 = ((NUMBER_PARTICLES_CUBED * INIT_SPACING + EXES_SPACING) / PARTICLE_SIZE) as u32;  // which is 
pub const BOX_SIZE: f32 = BIN_SIZE * (TRUE_BOX_SIZE as f32);
pub const BIN_COUNT: u32 = TRUE_BOX_SIZE as u32; // for each dimension

pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.03,
    g: 0.03,
    b: 0.03,
    a: 1.0,
};
pub const FPS: f32 = 60.0;

// mU = nm^2 * amu / ps^2
// J = m^2 * kg / s^2
pub const mU_over_eV: f32 = 0.010364269;
pub const eV_over_mU: f32 = 96.48533216;
pub const BOLTZMANN_CONSTANT_J: f32 = 1.38064852e-23; // in J / K
pub const BOLTZMANN_CONSTANT_EV: f32 = 8.617333262145e-5; // in eV / K
pub const BOLTZMANN_CONSTANT: f32 = BOLTZMANN_CONSTANT_EV * mU_over_eV; // in mU / K

/*
0.14417405 nm / ps
= 144.17405 m / s

 */
