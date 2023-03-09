use std::ops::Deref;
use std::sync::{Mutex, Arc};

use crate::system::consts::*;
use crate::system::params::Params;
use crate::system::particle::Particle;
use crate::system::stats::Stat;
use std::sync::mpsc::channel;
use wgpu::util::{DeviceExt, DownloadBuffer};
use wgpu::{BufferAsyncError, CommandEncoder, Device, Queue, Features};

use super::stats::{Stats, StatHistory};

macro_rules! compute_storage_descriptor {
    ($binding:expr, $min_binding_size:expr, $read_only:expr) => {
        wgpu::BindGroupLayoutEntry {
            binding: $binding,
            visibility: wgpu::ShaderStages::COMPUTE,
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

macro_rules! bind_group_entry {
    ($binding:expr, $buffer:expr) => {
        wgpu::BindGroupEntry {
            binding: $binding,
            resource: $buffer.as_entire_binding(),
        }
    };
}

macro_rules! storage_buffer_empty {
    ($device:expr, $label:expr, $null_data:expr, $size:expr) => {
        $device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some($label),
            contents: bytemuck::cast_slice(&[$null_data; $size as usize]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        })
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Timings {
    pub empty_bins: f32,
    pub binning: f32,
    pub energy: f32,
    pub collision: f32,
    pub total: f32,
}

pub struct ComputeSet {
    particle_buffers: Vec<wgpu::Buffer>,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    params: Params,
    params_buffer: wgpu::Buffer,
    bin_load_buffer: wgpu::Buffer,
    depth_buffer: wgpu::Buffer,
    verlet_bind_groups: Vec<wgpu::BindGroup>,
    empty_bins_bind_group: wgpu::BindGroup,
    bining_bind_groups: Vec<wgpu::BindGroup>,
    stats_bind_groups: Vec<wgpu::BindGroup>,
    stats_buffers: Vec<wgpu::Buffer>,
    stats_final_buffer: wgpu::Buffer,
    stats: Arc<Mutex<Stats>>,
    stats_history: Arc<Mutex<StatHistory>>,
    work_group_count: u32,
    verlet_pipeline: wgpu::ComputePipeline,
    compute_pipeline: wgpu::ComputePipeline,
    empty_bins_pipeline: wgpu::ComputePipeline,
    binning_pipeline: wgpu::ComputePipeline,
    stats_pipeline: wgpu::ComputePipeline,
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
        let empty_bins =
            device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\empty_bins.wgsl"));
        let empty_bins_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let empty_bins_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Empty Bins Pipeline Layout"),
                bind_group_layouts: &[&empty_bins_bind_group_layout],
                push_constant_ranges: &[],
            });
            

        let empty_bins_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Empty Bins Pipeline"),
                layout: Some(&empty_bins_pipeline_layout),
                module: &empty_bins,
                entry_point: "main",
            });

        // ------------------ binning shader setup ------------------ //

        let binning_shader =
            device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\calc_grid.wgsl"));

        let binning_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let binning_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

        let particle_update_shader =
            device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\compute.wgsl"));
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    Params::desc(),
                    Particle::desc(1, NUMBER_PARTICLES.into(), true),
                    Particle::desc(2, NUMBER_PARTICLES.into(), false),
                    // bin_load_buffer
                    compute_storage_descriptor!(3, 4, true),
                    // depth_buffer
                    compute_storage_descriptor!(4, 4, true),
                    // stats_buffer
                    Stat::desc(5, NUMBER_PARTICLES.into(), false)
                ],
                label: Some("compute_bind_group_layout"),
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            // contents: bytemuck::cast_slice(&params.raw()),
            contents: params.serialize(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // ------------------ reduction (energy) pipeline setup ------------------ //

        let reduction_shader =
            device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\reduce.wgsl"));
        let stats_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // energy_buffer (read only)
                    Stat::desc(0, NUMBER_PARTICLES.into(), true),
                    // energy_buffer (read/write)
                    Stat::desc(1, NUMBER_PARTICLES.into(), false),
                    // final_energy_buffer (read/write)
                    Stat::desc(2, 1, false),
                ],
                label: Some("reduction_bind_group_layout"),
            });

        let stats_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("reduction Pipeline Layout"),
                bind_group_layouts: &[&stats_bind_group_layout],
                push_constant_ranges: &[],
            });

        let stats_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("reduction Pipeline"),
            layout: Some(&stats_pipeline_layout),
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
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {}", i)),
                    contents: initial_particle_data,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                }),
            )
        }

        let bin_load_buffer = storage_buffer_empty!(device, "Bin Load Texture", 0u32, BIN_COUNT * BIN_COUNT);
        let depth_buffer = storage_buffer_empty!(device, "Depth Texture", 0i32, BIN_COUNT * BIN_COUNT * BIN_DEPTH);
        let stats_final_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stats Final Buffer"),
            contents: bytemuck::cast_slice(Stat::create_stats(1).as_slice()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        });

        let mut stats_buffers = Vec::<wgpu::Buffer>::new();
        let mut stats_bind_groups = Vec::<wgpu::BindGroup>::new();
        const NUM_PARTICLES_: usize =
            (1 << (u32::BITS - NUMBER_PARTICLES.leading_zeros())) as usize;
        println!(
            "num particles: {}, NUMBER_PARTICLES: {}",
            NUM_PARTICLES_, NUMBER_PARTICLES
        );
        for i in 0..2 {
            stats_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Stats Buffer {}", i)),
                    contents: bytemuck::cast_slice(Stat::create_stats(NUM_PARTICLES_).as_slice()),
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                })
            )
        }
        for i in 0..2 {
            stats_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &stats_bind_group_layout,
                entries: &[
                    bind_group_entry!(0, stats_buffers[i]),
                    bind_group_entry!(1, stats_buffers[(i + 1) % 2]),
                    bind_group_entry!(2, stats_final_buffer),
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
                    bind_group_entry!(5, stats_buffers[1]),
                ],
                label: None,
            }));
        }

        let verlet_shader = device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\varlets.wgsl"));

        let verlet_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    Params::desc(),
                    Particle::desc(1, NUMBER_PARTICLES.into(), false),
                ],
                label: Some("verlet pipeline bind group layout"),
            });

        let mut verlet_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            verlet_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &verlet_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                ],
                label: Some("verlet pipeline bind group"),
            }));
        }

        let verlet_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("verlet pipeline layout"),
                bind_group_layouts: &[&verlet_bind_group_layout],
                push_constant_ranges: &[],
            });

        let verlet_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("verlet pipeline"),
            layout: Some(&verlet_pipeline_layout),
            module: &verlet_shader,
            entry_point: "main",
        });

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



        let mut bining_bind_groups = Vec::<wgpu::BindGroup>::new();
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
        let stats = Arc::new(Mutex::new(Stats::default()));
        let st_hist = StatHistory::new(params.clone());
        let stats_history = Arc::new(Mutex::new(st_hist));

        Self {
            particle_buffers,
            particle_bind_groups,
            params,
            params_buffer,
            bin_load_buffer,
            depth_buffer,
            verlet_bind_groups,
            empty_bins_bind_group,
            bining_bind_groups,
            stats_bind_groups,
            stats_buffers,
            stats_final_buffer,
            stats,
            stats_history,
            work_group_count,
            verlet_pipeline,
            compute_pipeline,
            empty_bins_pipeline,
            binning_pipeline,
            stats_pipeline,
            total_iterations: 0,
        }
    }

    pub fn update(&mut self, encoder: &mut CommandEncoder, device: &Device, queue: &Queue, frame: usize) {
        encoder.push_debug_group("compute gravity and update positions");
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(format!("Compute Pass").as_str()),
            });
            for i in 0..ITERATIONS as usize {
                self.total_iterations += 1;

                // empty bins
                compute_pass.set_pipeline(&self.empty_bins_pipeline);
                compute_pass.set_bind_group(0, &self.empty_bins_bind_group, &[]);
                compute_pass.dispatch_workgroups(
                    ((BIN_COUNT * BIN_COUNT) as f32 / 256.0).ceil() as u32,
                    1,
                    1,
                );

                // bin particles
                compute_pass.set_pipeline(&self.binning_pipeline);
                compute_pass.set_bind_group(0, &self.bining_bind_groups[(frame + i) % 2], &[]);
                compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);

                // verlet 1
                compute_pass.set_pipeline(&self.verlet_pipeline);
                compute_pass.set_bind_group(0, &self.verlet_bind_groups[(frame + i) % 2], &[]);
                compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);

                // collisions
                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(0, &self.particle_bind_groups[(frame + i) % 2], &[]);
                compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);
            }

            // stats
            let iterations = (u32::BITS - NUMBER_PARTICLES.leading_zeros()) as usize;
            for (i, index) in (0..iterations).rev().enumerate() {
                compute_pass.set_pipeline(&self.stats_pipeline);
                compute_pass.set_bind_group(0, &self.stats_bind_groups[(i + 1) % 2], &[]);
                let work_group_count = ((1 << (index + 1)) as f32 / 64.0).ceil() as u32;
                compute_pass.dispatch_workgroups(work_group_count, 1, 1);
            }
        }
        encoder.pop_debug_group();
        if frame != 0 {
            self.download_stats(device, queue);
        }
    }

    pub fn get_particle_buffer(&self, frame: u32) -> &wgpu::Buffer {
        &self.particle_buffers[frame as usize % 2]
    }

    pub fn print_stats(&self) {
        println!("Total iterations: {}", self.total_iterations);
    }

    pub fn download_stats(&self, device: &Device, queue: &Queue) {
        wgpu::util::DownloadBuffer::read_buffer(
            device,
            queue,
            &self.stats_final_buffer.slice(..),
            {
                let stats = self.stats.clone();
                let stats_history = self.stats_history.clone();
                let itters = self.total_iterations as usize;
                move |r| {
                    let data = r.unwrap();
                    let mut stats_ = stats.lock().unwrap();
                    let mut stats_history_ = stats_history.lock().unwrap();
                    let stat = bytemuck::cast_slice::<u8, Stat>(&data[..])[0] as Stat;
                    stats_.KE = stat.KE / eV_over_mU;
                    stats_.PE = stat.PE / eV_over_mU;
                    stats_.iteration = itters;
                    stats_history_.add(*stats_);
                }
            }
        );
    }

    pub fn get_history(&self) -> StatHistory {
        // self.stats_history.clone().lock().unwrap() -> std::sync::MutexGuard<'_, StatHistory>
        self.stats_history.clone().lock().unwrap().deref().clone()
    }

    pub async fn debug(&mut self, device: &Device, queue: &Queue) {
        let elapsed = self.total_iterations as f32 * DT;
        print!("Time elapsed: {:.2} ps - iterations: {}k - ", elapsed, self.total_iterations / 1000);


        wgpu::util::DownloadBuffer::read_buffer(
            device,
            queue,
            &self.bin_load_buffer.slice(..),
            Self::print_data_load_buffer,
        );
        // wgpu::util::DownloadBuffer::read_buffer(
        //     device,
        //     queue,
        //     &self.energy_final_buffer.slice(..),
        //     Self::print_energy,
        // );
        println!("Stats: {:?}", self.stats.lock().unwrap());
    }

    pub fn write_stats(&self) {
        // write stats to csv file
        let date_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let file_name = format!("stats_{}.csv", date_time);

        match self.stats_history.lock().unwrap().save(file_name.as_str()) {
            Ok(_) => {
                println!("Stats saved to file: {}", file_name);
            }
            Err(e) => {
                println!("Error saving stats: {}", e);
            }
        }
    }
    

    fn set_energy(&mut self, r: Result<DownloadBuffer, BufferAsyncError>) {
        match r {
            Ok(buffer) => {
                let mut energy = bytemuck::cast_slice::<u8, f32>(&buffer[..])[0] as f32;
                energy /= eV_over_mU;
            }
            Err(e) => {
                println!("error: {:?}", e);
            }
        }
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
                println!("max particles per bin: {}", maxim);
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
                            let i = z + x * BIN_DEPTH + y * BIN_DEPTH * BIN_COUNT;
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

