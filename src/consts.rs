

pub const NUMBER_PARTICLES: u32 = 2000000;
pub const NEIGHBORHOOD_SIZE: f32 = 1.0;

pub const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.04, g: 0.04, b: 0.04, a: 1.0 };
pub const FPS: f32 = 60.0;
pub const ITERATIONS: u32 = 1;
pub const BIN_DEPTH: u32 = 3;
pub const BIN_SIZE: f32 = NEIGHBORHOOD_SIZE;
pub const BIN_COUNT: u32 = 3000; // for each dimension

pub const BOX_SIZE: f32 = BIN_SIZE * BIN_COUNT as f32;