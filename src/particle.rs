

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
    const MAX_TYPES: u32 = 3;
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![3 => Float32x2, 4 => Float32x2, 5 => Float32x2, 6 => Float32x3, 7 => Float32];

    pub fn new(container: f32, type_: f32) -> Self {
        let position = [container * (rand::random::<f32>()-0.5) * 2.0, container * (rand::random::<f32>()-0.5) * 2.0];
        let hue = map(type_, 0.0, Self::MAX_TYPES as f32, 0.0, 360.0);
        let color = hsb_to_rgb(hue, 1.0, 1.0);
        let velocity = [0.0, 0.0];
        Self {
            position,
            velocity,
            last_acceleration: [0.0, 0.0],
            color,
            type_ : type_,
        }
    }

    pub fn raw(&self) -> [f32; 10] {
        bytemuck::cast(*self)
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

    pub fn create_particles(num_particles: u64, container: f32) -> Vec<Particle> {
        let mut particles = Vec::with_capacity(num_particles as usize);
        for i in 0..num_particles {
            let _type = i as u32 % Self::MAX_TYPES;
            particles.push(Particle::new(container, _type as f32));
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