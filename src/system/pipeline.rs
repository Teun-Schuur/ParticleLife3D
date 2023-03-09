

// struct Pipeline{
//     pub label: String,
//     pub shader: wgpu::ComputePipeline,
//     pub bind_group_layouts: Vec<wgpu::BindGroupLayoutEntry>,
//     pub bind_groups: Vec<wgpu::BindGroup>,
// }

// impl Pipeline {
//     pub fn new(
//         device: &wgpu::Device,
//         file_name: &str,
//         label: &str,
//         bind_group_layouts: Vec<wgpu::BindGroupLayoutEntry>,
//         bind_groups: Vec<wgpu::BindGroup>,
//     ) -> Self {
//         // let file_name_ = format!("..\\shaders\\{file_name}").as_str();
//         let shader = wgpu::ShaderModuleDescriptor {
//             label: Some(file_name),
//             source: wgpu::ShaderSource::Wgsl(file_name.into()),
//         }

//         let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some(format!("{} layout", label).as_str()),
//             bind_group_layouts: &[&bind_group_layouts],
//             push_constant_ranges: &[],
//         });

//         let shader = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
//             label: Some(label),
//             layout: Some(&layout),
//             module: &shader,
//             entry_point: "main",
//         });
//         Self {
//             label: label.to_string(),
//             shader,
//             bind_group_layouts,
//             bind_groups,
//         }
//     }
// }