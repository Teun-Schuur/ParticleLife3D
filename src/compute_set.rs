use wgpu::util::{DeviceExt, DownloadBuffer};
use wgpu::{Device, CommandEncoder, Queue, BufferAsyncError};
use crate::consts::*;
use crate::params::Params;
use crate::particle::Particle;


pub struct ComputeSet {
    pub particle_buffers: Vec<wgpu::Buffer>,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    pub params: Params,
    pub params_buffer: wgpu::Buffer,
    pub bin_load_buffer: wgpu::Buffer,
    pub depth_buffer: wgpu::Buffer,
    pub empty_bins_bind_group: wgpu::BindGroup,
    pub bining_bind_groups: Vec<wgpu::BindGroup>,
    pub varlets_bind_groups: Vec<wgpu::BindGroup>,
    work_group_count: u32,
    compute_pipeline: wgpu::ComputePipeline,
    empty_bins_pipeline: wgpu::ComputePipeline,
    binning_pipeline: wgpu::ComputePipeline,
    varlets_pipeline: wgpu::ComputePipeline,
}

impl ComputeSet {
    /*
    pipeline layout:
    1. empty bins
    2. calculate bin load and the index for each particle in the bin
    3. sort particles into bins
    4. do the actual collision detection and update particles
    
     */
    pub fn new(device: &Device) -> Self {

        // ------------------ emptying bins shader setup ------------------ //
        let empty_bins = device.create_shader_module(wgpu::include_wgsl!("empty_bins.wgsl"));
        let empty_bins_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // bin_load_buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
            ],
            label: Some("empty_bins_bind_group_layout"),
        });

        let empty_bins_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Empty Bins Pipeline Layout"),
            bind_group_layouts: &[&empty_bins_bind_group_layout],
            push_constant_ranges: &[],
        });

        let empty_bins_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Empty Bins Pipeline"),
            layout: Some(&empty_bins_pipeline_layout),
            module: &empty_bins,
            entry_point: "main",
        });

        // // ------------------ calculate index shader setup ------------------ //
        
        // let calculate_index_shader = device.create_shader_module(wgpu::include_wgsl!("calculate_index.wgsl")); 



        // ------------------ binning shader setup ------------------ //

        let binning_shader = device.create_shader_module(wgpu::include_wgsl!("calc_grid.wgsl"));

        let binning_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // particle_buffer
                Particle::desc(0, NUMBER_PARTICLES.into(), true),
                // bin_load_buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
            ],
            label: Some("binning_bind_group_layout"),
        });

        let binning_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Binning Pipeline Layout"),
            bind_group_layouts: &[&binning_bind_group_layout],
            push_constant_ranges: &[],
        });

        let binning_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Binning Pipeline"),
            layout: Some(&binning_pipeline_layout),
            module: &binning_shader,
            entry_point: "main",
        });

        // ------------------ particle update shader setup ------------------ //

        let particle_update_shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                Params::desc(),
                Particle::desc(1, NUMBER_PARTICLES.into(), true),
                Particle::desc(2, NUMBER_PARTICLES.into(), false),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4),
                    },
                    count: None,
                },
            ],
            label: Some("compute_bind_group_layout"),
        });


        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });


        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &particle_update_shader,
            entry_point: "main",
        });

        
        // let params = Params::new((NUMBER_PARTICLES as f32).sqrt() * BOX_SIZE);
        let params = Params::new(BOX_SIZE, NEIGHBORHOOD_SIZE);
        let params_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::cast_slice(&params.raw()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // ------------------ varlets pipeline setup ------------------ //

        let varlets_shader = device.create_shader_module(wgpu::include_wgsl!("varlets.wgsl"));
        let varlets_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                Params::desc(),
                Particle::desc(1, NUMBER_PARTICLES.into(), false),
            ],
            label: Some("varlets_bind_group_layout"),
        });

        let varlets_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Varlets Pipeline Layout"),
            bind_group_layouts: &[&varlets_bind_group_layout],
            push_constant_ranges: &[],
        });

        let varlets_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Varlets Pipeline"),
            layout: Some(&varlets_pipeline_layout),
            module: &varlets_shader,
            entry_point: "main",
        });


        // ------------------ bin load texture setup ------------------ //
        
        let initial_particle_data = Particle::create_particles(NUMBER_PARTICLES.into(), BOX_SIZE);
        let initial_particle_data = initial_particle_data
            .iter()
            .map(|p| p.raw())
            .collect::<Vec<_>>();
        
        // two buffers for ping-ponging
        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Particle Buffer {}", i)),
                        contents: bytemuck::cast_slice(&initial_particle_data),
                        usage: wgpu::BufferUsages::VERTEX 
                            |wgpu::BufferUsages::STORAGE 
                            | wgpu::BufferUsages::COPY_DST
                            | wgpu::BufferUsages::COPY_SRC,
                        }
                )
            )
        }

        let bin_load_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Bin Load Texture"),
                contents: bytemuck::cast_slice(&[0u32; (BIN_COUNT * BIN_COUNT) as usize]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }
        );

        let depth_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Depth Texture"),
                contents: bytemuck::cast_slice(&[0i32; (BIN_COUNT * BIN_COUNT * BIN_DEPTH) as usize]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }
        );


        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst
        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: bin_load_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: depth_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }));
        }

        let mut varlets_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            varlets_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &varlets_bind_group_layout,
                entries: &[                 
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    }, 
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    }
                ],
                label: None,
            }));
        }

        let empty_bins_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &empty_bins_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bin_load_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: depth_buffer.as_entire_binding(),
                },
            ],
            label: Some("empty bins bind group"),
        });

        let mut bining_bind_groups =  Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            bining_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &binning_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: bin_load_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: depth_buffer.as_entire_binding(),
                    },
                ],
                label: None,
            }));
        }

        let work_group_count = (NUMBER_PARTICLES as f32 / 64.0).ceil() as u32;

        Self {
            particle_buffers,
            particle_bind_groups,
            params,
            params_buffer,
            bin_load_buffer,
            depth_buffer,
            empty_bins_bind_group,
            bining_bind_groups,
            varlets_bind_groups,
            work_group_count,
            compute_pipeline,
            empty_bins_pipeline,
            binning_pipeline,
            varlets_pipeline,
        }
    }

    pub fn update(&mut self, encoder: &mut CommandEncoder, frame: usize){
        encoder.push_debug_group("compute gravity and update positions");
        {
            for i in 0..ITERATIONS as usize{
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(format!("Compute Pass {}", i).as_str()),
                });

                // varlets
                compute_pass.set_pipeline(&self.varlets_pipeline);
                compute_pass.set_bind_group(0, &self.varlets_bind_groups[(frame + i ) % 2], &[]);
                compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);

                // // empty bins
                compute_pass.set_pipeline(&self.empty_bins_pipeline);
                compute_pass.set_bind_group(0, &self.empty_bins_bind_group, &[]);
                compute_pass.dispatch_workgroups(((BIN_COUNT * BIN_COUNT) as f32 / 256.0).ceil() as u32, 1, 1);

                // bin particles
                compute_pass.set_pipeline(&self.binning_pipeline);
                compute_pass.set_bind_group(0, &self.bining_bind_groups[(frame + i ) % 2], &[]);
                compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);
                
                // collisions
                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(0, &self.particle_bind_groups[(frame + i ) % 2], &[]);
                compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);
                
            }
        }
        encoder.pop_debug_group();
    }


    pub fn debug(&mut self, device: &Device, queue: &Queue){
        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.particle_buffers[0].slice(..), Self::print_data_particles);
        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.particle_buffers[1].slice(..), Self::print_data_particles);

        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.bin_load_buffer.slice(..), Self::print_data_load_buffer);
        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.depth_buffer.slice(..), Self::print_data_depth_buffer);
        
    }

        /// callback function for reading the particle buffer
    fn print_data_particles(r: Result<DownloadBuffer, BufferAsyncError>) {
        match r {
            Ok(buffer) => {
                let data = bytemuck::cast_slice::<u8, Particle>(&buffer[..]);
                for i in 0..data.len() {
                    println!("particle {}: {:?}", i, data[i]);
                }
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        }   
    }

    fn print_data_load_buffer(r: Result<DownloadBuffer, BufferAsyncError>) {
        match r {
            Ok(buffer) => {
                let mut tot = 0;
                let mut maxim = 0;
                let data = bytemuck::cast_slice::<u8, u32>(&buffer[..]);
                // println!("load buffer:");
                for y in 0..BIN_COUNT {
                    // print!("y({}): ", y);
                    for x in 0..BIN_COUNT {
                        let i = x + y * BIN_COUNT;
                        let d = data[i as usize];
                        tot += d;
                        if d > maxim {
                            maxim = d;
                        }
                        // print!(" {} ", d);
                    }
                    // println!("");
                }
                println!("total particles: {}", tot);
                println!("max particles: {}", maxim);
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        }
    }

    fn print_data_depth_buffer(r: Result<DownloadBuffer, BufferAsyncError>) {
        match r {
            Ok(buffer) => {
                let mut tot: u64 = 0;
                let data = bytemuck::cast_slice::<u8, i32>(&buffer[..]);
                println!("size: {}", data.len());
                for y in 0..BIN_COUNT {
                    for x in 0..BIN_COUNT {
                        for z in 0..BIN_DEPTH {
                            let i = z + x*BIN_DEPTH + y*BIN_DEPTH*BIN_COUNT;
                            let d = data[i as usize];
                            if d != -1 {
                                tot += 1;
                            }
                            // println!("bin {} {}: {:?}", x, y, data[i as usize]);
                        }
                    }
                }
                println!("total particles indexes: {}", tot);
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        }
    }
}

