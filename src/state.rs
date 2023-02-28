use std::{time};

use wgpu::{BufferAsyncError, Device};
use wgpu::util::{DeviceExt, DownloadBuffer};
use winit::window::Window;
use winit::event::{WindowEvent, VirtualKeyCode, ElementState, KeyboardInput};

use crate::buffers::Buffer;
use crate::compute_set::ComputeSet;
use crate::params::Params;
use crate::particle::Particle;
use crate::render_set::RenderSet;
use crate::vertex::{Vertex, Circle};
use crate::camera::{Camera, CameraUniform, CameraController};
use crate::consts::*;

// using version 0.15.0 of wgpu


pub struct State {
    pub device: Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub config: wgpu::SurfaceConfiguration,
    pub encoder: Option<wgpu::CommandEncoder>,
    pub window: Window,
    pub render: RenderSet,
    pub compute: ComputeSet,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub time: time::Instant,
    pub frame_count: u32,
    pub paused: bool,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        // instance is a handle to our GPU

        // bentchmarking
        // Vulkan: 900
        // DX12: 860
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
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
                power_preference: wgpu::PowerPreference::HighPerformance, // LowPower or HighPerformance
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

        let render = RenderSet::new(&window, &device, &config);
        let compute = ComputeSet::new(&device);

        let time = time::Instant::now();
        
        Self {
            device,
            queue,
            surface,
            adapter,
            config,
            encoder: None,
            window,
            render,
            compute,
            size,
            time,
            frame_count: 0,
            paused: false,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width <= 0 || new_size.height <= 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.size = new_size;
        self.surface.configure(&self.device, &self.config);

        self.render.resize(new_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.render.input(event);

        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                if is_pressed {
                    match keycode {
                        VirtualKeyCode::R => {
                            self.compute.params.reset_repulsion();
                            self.queue.write_buffer(&self.compute.params_buffer, 0, bytemuck::cast_slice(&self.compute.params.raw()));
                            println!("params: {:?}", self.compute.params);
                            true
                        }
                        VirtualKeyCode::Equals => {
                            self.compute.params.dt *= 1.1;
                            self.queue.write_buffer(&self.compute.params_buffer, 0, bytemuck::cast_slice(&self.compute.params.raw()));
                            println!("updated dt: {:?}", self.compute.params.dt);
                            true
                        }
                        VirtualKeyCode::Minus => {
                            self.compute.params.dt /= 1.1;
                            self.queue.write_buffer(&self.compute.params_buffer, 0, bytemuck::cast_slice(&self.compute.params.raw()));
                            println!("updated dt: {:?}", self.compute.params.dt);
                            true
                        }
                        VirtualKeyCode::P => {
                            self.paused = !self.paused;
                            true
                        }
                        VirtualKeyCode::Space => {
                            self.queue.write_buffer(&self.compute.particle_buffers[0], 0, bytemuck::cast_slice(&Particle::create_particles(NUMBER_PARTICLES.into(), (NUMBER_PARTICLES as f32).sqrt() * BOX_SIZE)));
                            self.queue.write_buffer(&self.compute.particle_buffers[1], 0, bytemuck::cast_slice(&Particle::create_particles(NUMBER_PARTICLES.into(), (NUMBER_PARTICLES as f32).sqrt() * BOX_SIZE)));
                            true
                        }
                        _ => false
                    }
                }
                else {
                    false
                }
            }
            _ => false,
        }

    }

    pub fn update(&mut self) {
        self.render.update_camera(&self.queue);

        self.encoder = Some(self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        }));


        if !self.paused{
            self.compute.update(
                self.encoder.as_mut().unwrap(),
                self.frame_count as usize,
            );
        }
        
        
        // time left over from last frame
        let time_left = 1.0 / FPS - self.time.elapsed().as_secs_f32();
        if self.frame_count % 60 == 0 {
            self.compute.debug(&self.device, &self.queue);
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

        
        // let frame = self.render.unwrap().render(self.encoder.take().unwrap(), &self.compute.unwrap().particle_buffers[(self.frame_count as usize + ITERATIONS as usize) % 2]);
        let frame = self.render.render(
            self.encoder.as_mut().unwrap(),
            &self.compute.particle_buffers[self.frame_count as usize % 2],
            &self.surface,
        );
        if !self.paused{
            self.frame_count += 1;
        }
        // println!("encoder: {:?}", self.encoder);
        let encoder_ = self.encoder.take().unwrap();
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder_.finish()));
        frame.present();
    
        Ok(())
    }
}
 