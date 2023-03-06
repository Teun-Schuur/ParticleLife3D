
use crate::system::consts::*;
use crate::utils::utils::maxwell_boltzmann_sampler;

use super::params::Params;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub last_acceleration: [f32; 2],
    pub color: [f32; 3],
    pub type_: f32,
}
unsafe impl bytemuck::Pod for Particle {}
unsafe impl bytemuck::Zeroable for Particle {}

impl Particle {
    const MAX_TYPES: u32 = 4;
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![3 => Float32x2, 4 => Float32x2, 5 => Float32x2, 6 => Float32x3, 7 => Float32];

    pub fn new(type_: f32, position: [f32; 2], velocity: [f32; 2]) -> Self {
        // let mut rng = rand::thread_rng();
        // let position = [container * (rand::random::<f32>()-0.5) * 2.0, container * (rand::random::<f32>()-0.5) * 2.0];
        let hue = map(type_, 0.0, Self::MAX_TYPES as f32, 0.0, 360.0);
        let color = hsb_to_rgb(hue, 1.0, 1.0);
        Self {
            position,
            velocity,
            last_acceleration: [0.0, 0.0],
            color,
            type_ : type_,
        }
    }

    // pub fn raw(&self) -> [f32; std::mem::size_of::<Particle>()/4] {
    //     bytemuck::cast(*self)
    // }

    pub fn serialize(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn serialize_all(particles: &[Particle]) -> &[u8] {
        bytemuck::cast_slice(particles)
    }

    pub fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Particle>() as wgpu::BufferAddress
    }

    pub fn desc(binding: u32, num_particles: u64, read_only: bool) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(Particle::size() * num_particles),
            },
            count: None,
        }
    }

    pub fn desc_render<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: Particle::size(),
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn create_particles(num_particles: u64, params: &Params) -> Vec<Particle> {
        // space particles evenly in a grid
        let mut particles = Vec::with_capacity(num_particles as usize);
        
        let side = (num_particles as f32).sqrt().ceil() as u32;
        // let spacing = params.box_size / side as f32;
        let spacing = NEIGHBORHOOD_SIZE / 5.0 * 0.55;
        let ofset = BOX_SIZE / 2.0 - spacing * side as f32 / 2.0;
        for i in 0..side {
            for j in 0..side {
                if i*side + j >= num_particles as u32 {
                    break;
                }
                let x = (i as f32) * spacing * 2.0 - params.box_size + ofset;
                let y = (j as f32) * spacing * 2.0- params.box_size + ofset;
                let _type = (i+j*side) as u32 % Self::MAX_TYPES;
                particles.push(Particle::new(
                    _type as f32, 
                    [x, y],
                    maxwell_boltzmann_sampler(INIT_TEMPERATURE, params.helium.mass)
                ));
            }
        }
        particles
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            last_acceleration: [0.0, 0.0],
            color: [0.0, 0.0, 0.0],
            type_: 0.0,
        }
    }
}

fn hsb_to_rgb(hue: f32, saturation: f32, brightness: f32) -> [f32; 3] {
    let hue = hue / 60.0;
    let i = hue.floor();
    let f = hue - i;
    let p = brightness * (1.0 - saturation);
    let q = brightness * (1.0 - saturation * f);
    let t = brightness * (1.0 - saturation * (1.0 - f));
    let (r, g, b) = match i as i32 {
        0 => (brightness, t, p),
        1 => (q, brightness, p),
        2 => (p, brightness, t),
        3 => (p, q, brightness),
        4 => (t, p, brightness),
        5 => (brightness, p, q),
        _ => (0.0, 0.0, 0.0),
    };
    [r, g, b]
}

fn map(value: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    start2 + (stop2 - start2) * ((value - start1) / (stop1 - start1))
}