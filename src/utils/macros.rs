// macros

mod macros {
    #[macro_export]
    macro_rules! compute_storage_descriptor {
        ($binding:expr, $min_binding_size:expr, $read_only:expr) => {
            wgpu::BindGroupLayoutEntry {
                binding: $binding,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: $read_only,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new($min_binding_size),
                },
                count: None,
            }
        };
    }
}
