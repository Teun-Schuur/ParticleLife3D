
// wgpu buffers

use wgpu::util::DeviceExt;

struct Buffer {
    buffer: Option<wgpu::Buffer>,
    size: Option<wgpu::BufferAddress>,
    binding: Option<wgpu::BindGroup>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            buffer: None,
            size: None,
            binding: None,
        }
    }

    pub fn create_buffer(&mut self, device: &wgpu::Device, size: wgpu::BufferAddress, usage: wgpu::BufferUsages, label: Option<&str>) {
        self.buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[0u8; 0]),
                usage,
            }
        ));
        self.size = Some(size);
    }

    pub fn set_data(&mut self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(self.buffer.as_ref().unwrap(), 0, data);
    }

    pub fn create_binding(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffers_to_bind: &[&Buffer], label: Option<&str>) {
        self.binding = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &buffers_to_bind.iter().enumerate().map(|(i, buffer)| {
                wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: buffer.buffer.as_ref().unwrap().as_entire_binding(),
                }
            }).collect::<Vec<wgpu::BindGroupEntry>>(),
            label,
        }));
    }

    pub fn create_binding_single(&mut self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout, label: Option<&str>) {
        self.binding = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_ref().unwrap().as_entire_binding(),
            }],
            label,
        }));
    }

    pub fn get_binding(&self) -> &wgpu::BindGroup {
        self.binding.as_ref().unwrap()
    }

    pub fn get_buffer(&self) -> &wgpu::Buffer {
        self.buffer.as_ref().unwrap()
    }

    pub fn get_size(&self) -> wgpu::BufferAddress {
        self.size.unwrap()
    }
}