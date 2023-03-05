use wgpu::util::{DeviceExt, DownloadBuffer};
use wgpu::{Device, CommandEncoder, Queue, BufferAsyncError};
use crate::system::consts::*;
use crate::system::params::Params;
use crate::system::particle::Particle;


macro_rules! compute_storage_descriptor {
    ($binding:expr, $min_binding_size:expr, $read_only:expr) => {
        wgpu::BindGroupLayoutEntry {
            binding: $binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: $read_only },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new($min_binding_size),
            },
            count: None,
        }
    };
}

macro_rules! bind_group_entry {
    ($binding:expr, $buffer:expr) => {
        wgpu::BindGroupEntry {
            binding: $binding,
            resource: $buffer.as_entire_binding(),
        }
    };
}

pub struct ComputeSet {
    pub particle_buffers: Vec<wgpu::Buffer>,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    pub params: Params,
    pub params_buffer: wgpu::Buffer,
    pub bin_load_buffer: wgpu::Buffer,
    pub depth_buffer: wgpu::Buffer,
    pub empty_bins_bind_group: wgpu::BindGroup,
    pub bining_bind_groups: Vec<wgpu::BindGroup>,
    pub energy_bind_groups: Vec<wgpu::BindGroup>,
    pub energy_buffers: Vec<wgpu::Buffer>,
    pub energy_final_buffer: wgpu::Buffer,
    work_group_count: u32,
    compute_pipeline: wgpu::ComputePipeline,
    empty_bins_pipeline: wgpu::ComputePipeline,
    binning_pipeline: wgpu::ComputePipeline,
    energy_pipeline: wgpu::ComputePipeline,
    total_iterations: u32,
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
        let empty_bins = device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\empty_bins.wgsl"));
        let empty_bins_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // params
                Params::desc(),
                // bin_load_buffer
                compute_storage_descriptor!(1, 4, false),
                // depth_buffer
                compute_storage_descriptor!(2, 4, false),
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

        // ------------------ binning shader setup ------------------ //

        let binning_shader = device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\calc_grid.wgsl"));

        let binning_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // params
                Params::desc(),
                // particle_buffer
                Particle::desc(1, NUMBER_PARTICLES.into(), true),
                // bin_load_buffer
                compute_storage_descriptor!(2, 4, false),
                // depth_buffer
                compute_storage_descriptor!(3, 4, false),
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

        let particle_update_shader = device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\compute.wgsl"));
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                Params::desc(),
                Particle::desc(1, NUMBER_PARTICLES.into(), true),
                Particle::desc(2, NUMBER_PARTICLES.into(), false),
                // bin_load_buffer
                compute_storage_descriptor!(3, 4, true),
                // depth_buffer
                compute_storage_descriptor!(4, 4, true),
                // energy_buffer
                compute_storage_descriptor!(5, 4, false),
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
        let params = Params::new();
        let params_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                // contents: bytemuck::cast_slice(&params.raw()),
                contents: params.serialize(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // ------------------ reduction (energy) pipeline setup ------------------ //

        let reduction_shader = device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\reduce.wgsl"));
        let energy_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // energy_buffer (read only)
                compute_storage_descriptor!(0, 4, true),
                // energy_buffer (read/write)
                compute_storage_descriptor!(1, 4, false),
                // final_energy_buffer (read/write)
                compute_storage_descriptor!(2, 4, false),
            ],
            label: Some("reduction_bind_group_layout"),
        });

        let energy_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("reduction Pipeline Layout"),
            bind_group_layouts: &[&energy_bind_group_layout],
            push_constant_ranges: &[],
        });

        let energy_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("reduction Pipeline"),
            layout: Some(&energy_pipeline_layout),
            module: &reduction_shader,
            entry_point: "main",
        });


        // ------------------ bin load texture setup ------------------ //
        
        let initial_particle_data = Particle::create_particles(NUMBER_PARTICLES.into(), &params);
        let initial_particle_data = Particle::serialize_all(&initial_particle_data);

        // two buffers for ping-ponging
        let mut particle_buffers = Vec::<wgpu::Buffer>::new();
        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Particle Buffer {}", i)),
                        contents: initial_particle_data,
                        usage: wgpu::BufferUsages::VERTEX 
                            |wgpu::BufferUsages::STORAGE,
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


        // ------------------ energy buffer setup ------------------ //
        
        let energy_final_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Energy Final Buffer"),
                contents: bytemuck::cast_slice(&[0f32; 1]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            }
        );

        let mut energy_buffers = Vec::<wgpu::Buffer>::new();
        let mut energy_bind_groups = Vec::<wgpu::BindGroup>::new();
        const num_particles: usize = (1 << (u32::BITS - NUMBER_PARTICLES.leading_zeros())) as usize;
        println!("num particles: {}, NUMBER_PARTICLES: {}", num_particles, NUMBER_PARTICLES);
        for i in 0..2 {
            energy_buffers.push(
                device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("Energy Buffer {}", i)),
                        contents: bytemuck::cast_slice(&[0f32; num_particles]),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                    }
                )
            )
        }
        for i in 0..2 {
            energy_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &energy_bind_group_layout,
                entries: &[
                    bind_group_entry!(0, energy_buffers[i]),
                    bind_group_entry!(1, energy_buffers[(i + 1) % 2]),
                    bind_group_entry!(2, energy_final_buffer),
                ],
                label: Some("reduction_bind_group"),
            }));
        }


        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst
        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    bind_group_entry!(0, params_buffer),
                    bind_group_entry!(1, particle_buffers[i]),
                    bind_group_entry!(2, particle_buffers[(i + 1) % 2]),
                    bind_group_entry!(3, bin_load_buffer),
                    bind_group_entry!(4, depth_buffer),
                    bind_group_entry!(5, energy_buffers[1]),
                ],
                label: None,
            }));
        }

        let empty_bins_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &empty_bins_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
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
            label: Some("empty bins bind group"),
        });

        let mut bining_bind_groups =  Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            bining_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &binning_bind_group_layout,
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
                        resource: bin_load_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
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
            energy_bind_groups,
            energy_buffers,
            energy_final_buffer,
            work_group_count,
            compute_pipeline,
            empty_bins_pipeline,
            binning_pipeline,
            energy_pipeline,
            total_iterations: 0,
        }
    }

    pub fn update(&mut self, encoder: &mut CommandEncoder, frame: usize){
        encoder.push_debug_group("compute gravity and update positions");
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(format!("Compute Pass").as_str()),
            });
            for i in 0..ITERATIONS as usize{
                self.total_iterations += 1;

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

            // energy
            let iterations = (u32::BITS - NUMBER_PARTICLES.leading_zeros()) as usize;
            // println!("iterations to add energies: {}", iterations);
            for (i, index) in (0..iterations).rev().enumerate(){
                compute_pass.set_pipeline(&self.energy_pipeline);
                compute_pass.set_bind_group(0, &self.energy_bind_groups[(frame + i + 1) % 2], &[]);
                let work_group_count = ((1 << (index+1)) as f32 / 64.0).ceil() as u32;
                // println!("work group count: {} for iteration {}.", work_group_count, i);
                compute_pass.dispatch_workgroups(work_group_count, 1, 1);
            }
        }
        encoder.pop_debug_group();
    }

    // pub fn reduce(compute_pass: &mut wgpu::ComputePass, bind_groups: &Vec<&wgpu::BindGroup>, frame: usize){
    //     let iterations = (u32::BITS - NUMBER_PARTICLES.leading_zeros()) as usize;
    //     for (i, index) in (0..iterations).rev().enumerate(){
    //         compute_pass.set_bind_group(0, bind_groups[(frame + i + 1) % 2], &[]);
    //         let work_group_count = ((1 << (index+1)) as f32 / 64.0).ceil() as u32;
    //         compute_pass.dispatch_workgroups(work_group_count, 1, 1);
    //     }
    // }

    pub fn debug(&mut self, device: &Device, queue: &Queue){
        let elapsed = self.total_iterations as f32 * DT;
        print!("Time elapsed: {:.2} ps, ", elapsed);

        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.particle_buffers[0].slice(..), Self::print_data_particles);
        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.particle_buffers[1].slice(..), Self::print_data_particles);

        wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.bin_load_buffer.slice(..), Self::print_data_load_buffer);
        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.depth_buffer.slice(..), Self::print_data_depth_buffer);
        wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.energy_final_buffer.slice(..), Self::print_energy);
        // wgpu::util::DownloadBuffer::read_buffer(device, queue, &self.energy_buffers[1].slice(..), Self::print_energy_buffer);
        // println!("");
    }

    fn print_energy(r: Result<DownloadBuffer, BufferAsyncError>) {
        match r {
            Ok(buffer) => {
                let mut energy = bytemuck::cast_slice::<u8, f32>(&buffer[..])[0] as f32;
                energy /= eV_over_mU;
                println!("energy: {energy} eV");
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        }
    }

    fn print_energy_buffer(r: Result<DownloadBuffer, BufferAsyncError>) {
        match r {
            Ok(buffer) => {
                let mut energy = 0.0;
                let mut tot_energy = 0.0;
                let energies = bytemuck::cast_slice::<u8, f32>(&buffer[..]);
                for i in 0..NUMBER_PARTICLES as usize {
                    let mut energy = energies[i] as f32;
                    energy *= eV_over_mU;
                    tot_energy += energy;
                    println!("energy {}: {energy} eV", i);
                }
                println!("energy True: {tot_energy} eV");
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        }
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
                let mut maxim = 0;
                let data = bytemuck::cast_slice::<u8, u32>(&buffer[..]);
                // println!("load buffer:");
                for y in 0..BIN_COUNT {
                    // print!("y({}): ", y);
                    for x in 0..BIN_COUNT {
                        let i = x + y * BIN_COUNT;
                        let d = data[i as usize];
                        if d > maxim {
                            maxim = d;
                        }
                    }
                }
                if maxim > BIN_DEPTH {
                    println!("max particles per bin exceeded: {}", maxim);
                }
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

