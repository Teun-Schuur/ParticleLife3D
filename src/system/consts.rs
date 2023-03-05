/*
time: ps
distance: nm
mass: amu
charge: e
temperature: K
force: eV / nm or nm * amu / ps^2 or 1.660539040 pN

 */

pub const MAX_FORCE: f32 = 200e30;

pub const NUMBER_PARTICLES: u32 = 1000;
pub const NEIGHBORHOOD_SIZE: f32 = 10.0; // in nm
pub const INIT_TEMPERATURE: f32 = 300.0; // in Kelvin
// pub const INIT_VELOCITY: f32 = 100e-3; // 1 nm per picosecond

pub const DT : f32 = 1e-3; // in picoseconds
pub const ITERATIONS: u32 = 100;
pub const BIN_DEPTH: u32 = 100;
pub const BIN_SIZE: f32 = NEIGHBORHOOD_SIZE;

const TRUE_BOX_SIZE: f32 = 10.0;    // this times bin_size in nm is the size of the box
pub const BOX_SIZE: f32 = BIN_SIZE * TRUE_BOX_SIZE;
pub const BIN_COUNT: u32 = TRUE_BOX_SIZE as u32; // for each dimension

pub const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.03, g: 0.03, b: 0.03, a: 1.0 };
pub const FPS: f32 = 60.0;

pub const mU_over_eV : f32 = 0.010364269;
pub const eV_over_mU : f32 = 96.48533216;
pub const BOLTZMANN_CONSTANT_J : f32 = 1.38064852e-23; // in J / K
pub const BOLTZMANN_CONSTANT_EV : f32 = 8.617333262145e-5; // in eV / K
pub const BOLTZMANN_CONSTANT : f32 = BOLTZMANN_CONSTANT_EV * mU_over_eV; // in mU / K


/*
0.14417405 nm / ps
= 144.17405 m / s

 */