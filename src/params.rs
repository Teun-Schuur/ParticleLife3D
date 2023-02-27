use std::fmt::Debug;

use rand::Rng;



#[repr(C)]
#[derive(Copy, Clone)]
pub struct Params {
    pub dt: f32,
    pub neghborhood_size: f32,
    pub max_force: f32,
    pub friction: f32,
    pub global_repulsion_distance: f32,
    pub box_size: f32,
    pub attraction: [[f32; 4]; 4],
    _padding: [f32; 2],
}
unsafe impl bytemuck::Pod for Params {}
unsafe impl bytemuck::Zeroable for Params {}

impl Params {
    pub fn new(box_size: f32, neghborhood_size: f32) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            dt: 0.04,
            neghborhood_size,
            max_force: 300.0,
            friction: 0.05,
            global_repulsion_distance: 6.0,
            // attraction_one: [-1.0, -0.1, 0.1],
            // attraction_two: [0.1, -1.0, -0.1],
            // attraction_three: [-0.1, -0.1, -1.0],
            box_size,
            attraction: [
                Self::get_random_attraction(),
                Self::get_random_attraction(),
                Self::get_random_attraction(),
                Self::get_random_attraction(),
            ],
            _padding: [0.0; 2],
        }
    }

    fn get_random_attraction() -> [f32; 4] {
        const MAX_ATTRACTION: f32 = 0.4;
        let mut rng = rand::thread_rng();
        [
            rng.gen_range(-MAX_ATTRACTION..MAX_ATTRACTION), 
            rng.gen_range(-MAX_ATTRACTION..MAX_ATTRACTION), 
            rng.gen_range(-MAX_ATTRACTION..MAX_ATTRACTION), 
            rng.gen_range(-MAX_ATTRACTION..MAX_ATTRACTION), 
        ]
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

    pub fn raw(&self) -> [f32; std::mem::size_of::<Params>()/4] {
        bytemuck::cast(*self)
    }

    pub fn reset_repulsion(&mut self) {
        self.attraction = [
            Self::get_random_attraction(),
            Self::get_random_attraction(),
            Self::get_random_attraction(),
            Self::get_random_attraction(),
        ];
    }
}

impl Debug for Params {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Params")
            .field("dt", &self.dt)
            .field("neghborhood_size", &self.neghborhood_size)
            .field("max_force", &self.max_force)
            .field("friction", &self.friction)
            .field("global_repulsion_distance", &self.global_repulsion_distance)
            .field("box_size", &self.box_size)
            .field("attraction", &self.attraction)
            .finish()
    }
}