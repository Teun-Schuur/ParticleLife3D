use std::fmt::Debug;

use rand::Rng;

use crate::system::consts::*;
use crate::utils::wgsl_types::*;

// https://openkim.org/files/MO_959249795837_003/LennardJones612_UniversalShifted.params
#[repr(C, align(4))]
#[derive(Copy, Clone)]
pub struct Atom{
    pub size: f32, // in nm
    pub mass: f32, // in Dalton (1.66053906660e-27 kg)
    pub charge: i32, // in elementary charge (1.602176634e-19 C)
    pub sigma: f32, // in nm
    pub epsilon: f32, // nm^2 * u * ps^-2 (1.66053907 J)
}
unsafe impl bytemuck::Pod for Atom {}
unsafe impl bytemuck::Zeroable for Atom {}

#[repr(C, align(4))]
#[derive(Copy, Clone)]
pub struct Params {
    pub dt: f32,  // in ps
    pub neghborhood_size: f32, // in nm
    pub max_force: f32, // in nm * amu / ps^2
    pub friction: f32,  // in amu / ps
    pub box_size: f32, // in nm
    pub bin_size: f32, // in nm
    pub bin_count: u32,
    pub bin_capacity: u32,
    pub helium: Atom,
}
unsafe impl bytemuck::Pod for Params {}
unsafe impl bytemuck::Zeroable for Params {}

impl Params {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            dt: DT,
            neghborhood_size: NEIGHBORHOOD_SIZE,
            max_force: MAX_FORCE,
            friction: 0.0,
            box_size: BOX_SIZE,
            bin_size: BIN_SIZE,
            bin_count: BIN_COUNT as u32,
            bin_capacity: BIN_DEPTH as u32,
            // helium: Atom {
            //     size: 0.04989030,
            //     mass: 4.002602,
            //     charge: 0,
            //     sigma: 0.04989030,
            //     epsilon: 0.0009421 * mU_over_eV,
            // },
            helium: Atom {
                size: 2.2807000,
                mass: 6.0,
                charge: 0,
                sigma: 2.2807000,
                epsilon: 1.0496900 * mU_over_eV,
            }
        }
    }

    pub fn desc() -> wgpu::BindGroupLayoutEntry {
        println!("size of Params: {}", std::mem::size_of::<Params>() as wgpu::BufferAddress);
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(
                    std::mem::size_of::<Params>() as wgpu::BufferAddress,
                ),
            },
            count: None,
        }
    }


    pub fn serialize(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl Debug for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Params")
            .field("dt", &self.dt)
            .field("neghborhood_size", &self.neghborhood_size)
            .field("max_force", &self.max_force)
            .field("friction", &self.friction)
            .field("box_size", &self.box_size)
            .field("bin_size", &self.bin_size)
            .field("bin_count", &self.bin_count)
            .field("bin_capacity", &self.bin_capacity)
            .field("helium", &self.helium)
            .finish()
    }
}

impl Debug for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Atom")
            .field("size (nm)", &self.size)
            .field("mass (u)", &self.mass)
            .field("charge (q)", &self.charge)
            .field("sigma (nm)", &self.sigma)
            .field("epsilon (eV)", &self.epsilon)
            .finish()
    }
}