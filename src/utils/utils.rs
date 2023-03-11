use std::{f32::consts::PI};

use rand::{distributions::Uniform, prelude::Distribution, thread_rng, Rng};

use crate::system::consts::{self, BOLTZMANN_CONSTANT, BOLTZMANN_CONSTANT_J};
use rand_distr::Normal;

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

unsafe fn u8_slice_as_any<T>(p: &[u8]) -> &T {
    assert_eq!(p.len(), ::core::mem::size_of::<T>());
    &*(p.as_ptr() as *const T)
}

pub fn maxwell_boltzmann_sampler(temperature: f32, mass: f32) -> [f32; 3] {
    let v_rms = ((3.0 * BOLTZMANN_CONSTANT_J * temperature) / (mass * 1.66053906660e-27)).sqrt(); // Root-mean-square velocity
    let normal = Normal::new(0.0, v_rms).unwrap();
    let mut rng = rand::thread_rng();
    let mut v_x: f32 = normal.sample(&mut rng) * 0.5;
    let mut v_y: f32 = normal.sample(&mut rng) * 0.5;
    let mut v_z: f32 = normal.sample(&mut rng) * 0.5;
    // let length = (v_x * v_x + v_y * v_y + v_z * v_z).sqrt();
    
    // println!("v_rms: {v_rms}, v_x: {v_x}, v_y: {v_y}, length: {length}");
    return [v_x / 1000.0, v_y / 1000.0, v_z / 1000.0];
}
