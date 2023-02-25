use std::{time};

use wgpu::BufferAsyncError;
use wgpu::util::{DeviceExt, DownloadBuffer};
use winit::window::Window;
use winit::event::{WindowEvent};

use crate::params::Params;
use crate::particle::Particle;
use crate::vertex::{Vertex, Circle};
use crate::camera::{Camera, CameraUniform, CameraController};

// using version 0.15.0 of wgpu
const CLEAR_COLOR: wgpu::Color = wgpu::Color { r: 0.04, g: 0.04, b: 0.04, a: 1.0 };
const FPS: f32 = 60.0;
const NUMBER_PARTICLES: u32 = 20;
const BOX_SIZE: f32 = 10.0;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    particle_buffers: Vec<wgpu::Buffer>,
    compute_pipeline: wgpu::ComputePipeline,
    work_group_count: u32,
    time: time::Instant,
    frame_count: u32,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {

        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        // instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // Surface is the abstraction to present things to the screen
        // it works by creating a "swapchain" of images that are presented to the screen
        let surface = unsafe { instance.create_surface(&window).unwrap() };

        // The adapter is a handle to our actual graphics card. 
        // You can use this to get information about the graphics card such as its name and what backend the adapter uses. 
        // We use this to create our Device and Queue later.
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), // LowPower or HighPerformance
                compatible_surface: Some(&surface),  //  tells wgpu to find an adapter that can present to the supplied surface.
                force_fallback_adapter: false,  //  tells wgpu to use the software adapter if no hardware adapters are available.
            })
            .await
            .unwrap();

        // The device and queue are the main handles to the GPU.
        // The device is used to create most of the objects we will use in wgpu.
        // The queue is used to submit commands to the GPU.
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        // The surface capabilities tell us the surface's current size and other details.
        let surface_caps = surface.get_capabilities(&adapter);

        // The surface format is the format of the pixels that will be presented to the screen.
        // The surface format is usually a sRGB format.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|format| format.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        
        // The surface configuration is the configuration of the swapchain.
        // The swapchain is the list of images that are presented to the screen.
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // Now we create the render pipeline
        // The render pipeline is the pipeline that tells the GPU how to render things to the screen.
        // The render pipeline consists of a vertex shader and a fragment shader.
        // The vertex shader is used to transform vertices into clipspace.
        // The fragment shader is used to color the pixels in the triangles.

        
        // buffers:
        
        let camera_uniform = CameraUniform::new();
        
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let circle = Circle::new(6);
        
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&circle.get_vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&circle.get_indices()),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        let num_indices = circle.num_indices;
        
        let camera_controller = CameraController::new(0.03, 0.03);
        
        let camera = Camera::new();

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
                ],
                label: Some("camera_bind_group_layout"),
            });
            
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
                ],
                label: Some("camera_bind_group"),
            });
                
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
            
            
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc(),
                    Particle::desc_render(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let compte_shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                Params::desc(),
                Particle::desc(1, NUMBER_PARTICLES.into(), true),
                Particle::desc(2, NUMBER_PARTICLES.into(), false),
            ],
            label: Some("compute_bind_group_layout"),
        });

        println!("compute_bind_group_layout: {:?}", compute_bind_group_layout);

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        println!("compute_pipeline_layout: {:?}", compute_pipeline_layout);

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compte_shader,
            entry_point: "main",
        });

        println!("compute_pipeline: {:?}", compute_pipeline);
        
        let params = Params::new();
        let params_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::cast_slice(&params.raw()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        
        let initial_particle_data = Particle::create_particles(NUMBER_PARTICLES.into(), (NUMBER_PARTICLES as f32).sqrt() * BOX_SIZE);
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
                            // | wgpu::BufferUsages::COPY_SRC,
                        }
                )
            )
        }

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
                ],
                label: None,
            }));
        }

        let work_group_count = (NUMBER_PARTICLES as f32 / 64.0).ceil() as u32;

        let time = time::Instant::now();
        
        State {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            particle_bind_groups,
            particle_buffers,
            compute_pipeline,
            work_group_count,
            time,
            frame_count: 0,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width <= 0 || new_size.height <= 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event);
        match event {
            // get mouse position
            _ => false,
        }
    }

    pub fn update(&mut self) {        
        self.camera_controller.update_camera(&mut self.camera, &self.size);
        self.camera_uniform.update_view_proj(&mut self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        
        // time left over from last frame
        let time_left = 1.0 / FPS - self.time.elapsed().as_secs_f32();
        if self.frame_count % 60 == 0 {
            // wgpu::util::DownloadBuffer::read_buffer(&self.device, &self.queue, &self.particle_buffers[0].slice(..), print_data);
            let used_time_fraction = 1.0 - time_left / (1.0 / FPS);
            let used_time = used_time_fraction * 1.0 / FPS;
            println!("used time: {} / {} ms,  fraction: {}%", used_time*1000.0, 1.0 / FPS * 1000.0, used_time_fraction*100.0);
        }
        // wait till the end of the frame to reset the timer
        while self.time.elapsed().as_secs_f32() < 1.0 / FPS {
        }

        if self.frame_count % 60 == 0 {
            println!("FPS: {}", 1.0 / self.time.elapsed().as_secs_f32());
        }
        self.time = time::Instant::now();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });
        
        encoder.push_debug_group("compute gravity and update positions");
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.particle_bind_groups[self.frame_count as usize % 2], &[]);
            compute_pass.dispatch_workgroups(self.work_group_count, 1, 1);
        }
        encoder.pop_debug_group();


        encoder.push_debug_group("render particles");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            render_pass.set_vertex_buffer(1, self.particle_buffers[(self.frame_count as usize + 1) % 2].slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..NUMBER_PARTICLES as _);
        }
        encoder.pop_debug_group();

        self.frame_count += 1;
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    
        Ok(())
    }
}
 

/// callback function for reading the particle buffer
fn print_data(r: Result<DownloadBuffer, BufferAsyncError>) {
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