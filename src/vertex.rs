


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex{
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];//, 1 => Float32x3];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}


pub struct Circle {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    pub num_indices: u32,
}

impl Circle {
    pub fn new(num_points: u16) -> Self {
        let (vertices, indices) = Circle::create(&num_points);
        let num_indices = indices.len() as u32;
        Self {
            vertices,
            indices,
            num_indices,
        }
    }

    fn create(num_points: &u16) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut angle = 0.0;
        let angle_increment = 2.0 * std::f32::consts::PI / *num_points as f32;
        for i in 0..*num_points {
            vertices.push(Vertex { 
                position: [
                    (angle as f32).cos(), 
                    (angle as f32).sin(), 
                    0.0], 
                }
            );
            indices.push(0);
            indices.push(i);
            indices.push(i + 1);
            angle += angle_increment;
        }
        (vertices, indices)
    }

    pub fn get_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        for vertex in &self.vertices {
            vertices.push(Vertex { position: [vertex.position[0], vertex.position[1], 0.0]});
        }
        vertices
    }

    pub fn get_indices(&self) -> Vec<u16> {
        self.indices.clone()
    }
}
