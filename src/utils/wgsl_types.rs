use std::fmt::Debug;



#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct mat4x4 {
    pub m: [[f32; 4]; 4],
}
unsafe impl bytemuck::Pod for mat4x4 {}
unsafe impl bytemuck::Zeroable for mat4x4 {}

impl mat4x4 {
    pub fn new() -> Self {
        Self {
            m: [[0.0; 4]; 4],
        }
    }
}

impl Debug for mat4x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mat4x4 {{ m: [")?;
        for i in 0..4 {
            write!(f, "[")?;
            for j in 0..4 {
                write!(f, "{}, ", self.m[i][j])?;
            }
            write!(f, "], ")?;
        }
        write!(f, "] }}")
    }
}