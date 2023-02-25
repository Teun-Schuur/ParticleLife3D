

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub dt: f32,
    pub neghborhood_size: f32,
    pub max_velocity: f32,
    pub friction: f32,
    pub attraction_one: [f32; 3],
    pub attraction_two: [f32; 3],
    pub attraction_three: [f32; 3],
    // pub _padding: [f32; 3],
}
unsafe impl bytemuck::Pod for Params {}
unsafe impl bytemuck::Zeroable for Params {}

impl Params {
    // const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![0 => Float32, 1 => Float32, 2 => Float32, 3 => Float32];
    pub fn new() -> Self {
        Self {
            dt: 0.01,
            neghborhood_size: 0.1,
            max_velocity: 1.0,
            friction: 0.001,
            attraction_one: [0.0, 0.0, 0.0],
            attraction_two: [0.0, 0.0, 0.0],
            attraction_three: [0.0, 0.0, 0.0],
            // _padding: [0.0; 3],
        }
    }

    pub fn desc() -> wgpu::BindGroupLayoutEntry {
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

    pub fn raw(&self) -> [f32; 13] {
        // println!("got here");
        // let cast: [f32; 13] = bytemuck::cast(*self);
        // println!("data: {:?}", cast);
        // return cast;
        bytemuck::cast(*self)
    }
}