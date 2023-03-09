// wgpu buffers

use wgpu::util::DeviceExt;

pub struct Buffer {
    buffer: Option<wgpu::Buffer>,
    size: Option<wgpu::BufferAddress>,
    data_: Option<Vec<u8>>,
    usage_: Option<wgpu::BufferUsages>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            buffer: None,
            size: None,
            data_: None,
            usage_: None,
        }
    }

    pub fn with_data(self, data: &[u8]) -> Self {
        Self {
            data_: Some(data.to_vec()),
            ..self
        }
    }

    pub fn with_usage(self, usage: wgpu::BufferUsages) -> Self {
        Self {
            usage_: Some(usage),
            ..self
        }
    }

    pub fn build(mut self, device: &wgpu::Device, label: Option<&str>) -> Self {
        let data = self.data_.as_ref().unwrap();
        let usage = self.usage_.unwrap();
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&data),
                usage,
            }),
        );
        self
    }

    pub fn create_buffer(
        &mut self,
        device: &wgpu::Device,
        size: wgpu::BufferAddress,
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) {
        self.buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[0u8; 0]),
                usage,
            }),
        );
        self.size = Some(size);
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        self.buffer.as_ref().unwrap()
    }

    pub fn get_size(&self) -> wgpu::BufferAddress {
        self.size.unwrap()
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.buffer.as_ref().unwrap().as_entire_binding()
    }
}

struct BufferPair {
    buffers: [wgpu::Buffer; 2],
    bind_groups: [wgpu::BindGroup; 2],
}

// impl BufferPair {
//     pub fn new(device: &wgpu::Device, size: wgpu::BufferAddress, usage: wgpu::BufferUsages, label: Option<&str>) -> Self {
//         let mut buffers = [device.create_buffer(&wgpu::BufferDescriptor {
//             label,
//             size,
//             usage,
//             mapped_at_creation: false,
//         }); 2];
//         let bind_groups = [
//             device.create_bind_group(&wgpu::BindGroupDescriptor {
//                 layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//                     entries: &[wgpu::BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: wgpu::ShaderStages::VERTEX,
//                         ty: wgpu::BindingType::Buffer {
//                             ty: wgpu::BufferBindingType::Storage { read_only: false },
//                             has_dynamic_offset: false,
//                             min_binding_size: None,
//                         },
//                         count: None,
//                     }],
//                     label: None,
//                 }),
//                 entries: &[wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::Buffer(buffers[0].slice(..)),
//                 }],
//                 label: None,
//             }),
//             device.create_bind_group(&wgpu::BindGroupDescriptor {
//                 layout: &device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//                     entries: &[wgpu::BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: wgpu::ShaderStages::VERTEX,
//                         ty: wgpu::BindingType::Buffer {
//                             ty: wgpu::BufferBindingType::Storage { read_only: false },
//                             has_dynamic_offset: false,
//                             min_binding_size: None,
//                         },
//                         count: None,
//                     }],
//                     label: None,
//                 }),
//                 entries: &[wgpu::BindGroupEntry {
//                     binding: 0,
//                     resource: wgpu::BindingResource::Buffer(buffers[1].slice(..)),
//                 }],
//                 label: None,
//             }),
//         ];
//         Self { buffers, bind_groups }
//     }

//     pub fn swap(&mut self) {
//         self.buffers.swap(0, 1);
//         self.bind_groups.swap(0, 1);
//     }
// }
