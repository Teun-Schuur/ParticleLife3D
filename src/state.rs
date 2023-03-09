use std::time;
use std::time::Duration;

use egui::FontDefinitions;
use egui_demo_lib::DemoWindows;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use wgpu::util::{DeviceExt, DownloadBuffer};
use wgpu::{BufferAsyncError, Device};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;

use crate::render::gui::GUI;
use crate::system::{compute_set::ComputeSet, consts::*, params::Params, particle::Particle};

use crate::render::{
    camera::{Camera, CameraController, CameraUniform},
    render_set::RenderSet,
    vertex::{Circle, Vertex},
};

use crate::utils::buffers::Buffer;

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
    pub platform: Platform,
    pub egui_rpass: RenderPass,
    pub demo_app: GUI,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub time: time::Instant,
    pub frame_time: time::Instant,
    pub frame_count: u32,
    pub paused: bool,
    pub dt: Duration,
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
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance, // LowPower or HighPerformance
                compatible_surface: Some(&surface), //  tells wgpu to find an adapter that can present to the supplied surface.
                force_fallback_adapter: false, //  tells wgpu to use the software adapter if no hardware adapters are available.
            })
            .await
            .unwrap();

        // The device and queue are the main handles to the GPU.
        // The device is used to create most of the objects we will use in wgpu.
        // The queue is used to submit commands to the GPU.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

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

        // We use the egui_winit_platform crate as the platform.
        let platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });
        let egui_rpass = RenderPass::new(&device, surface_format, 1);
        let demo_app = GUI::default();

        let render = RenderSet::new(&window, &device, &config);
        let compute = ComputeSet::new(&device);

        let time = time::Instant::now();
        let frame_time = time::Instant::now();

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
            platform,
            egui_rpass,
            demo_app,
            size,
            time,
            frame_time,
            frame_count: 0,
            paused: false,
            dt: Duration::from_millis(16),
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

    pub fn handle_event(&mut self, event: &Event<()>) {
        self.platform.handle_event(event);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.render.input(event);

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                if is_pressed {
                    match keycode {
                        VirtualKeyCode::P => {
                            self.paused = !self.paused;
                            true
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.render.update_camera(&self.queue, self.dt);

        self.encoder = Some(
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                }),
        );

        if !self.paused {
            self.compute.update(
                self.encoder.as_mut().unwrap(),
                &self.device,
                &self.queue,
                 self.frame_count as usize
            );
        }

        // time left over from last frame
        let time_left = 1.0 / FPS - self.frame_time.elapsed().as_secs_f32();
        if self.frame_count % 60 == 0 {
            let used_time_fraction = 1.0 - time_left / (1.0 / FPS);
            let used_time = used_time_fraction * 1.0 / FPS;
            println!("========================== 60 frames elapsed ============================");
            print!(
                "used time: {} / {} ms,  fraction: {}%, ",
                used_time * 1000.0,
                1.0 / FPS * 1000.0,
                used_time_fraction * 100.0
            );
        }
        // wait till the end of the frame to reset the timer
        while self.frame_time.elapsed().as_secs_f32() < 1.0 / FPS {}
        if self.frame_count % 60 == 0 {
            println!("FPS: {}", 1.0 / self.frame_time.elapsed().as_secs_f32());
            pollster::block_on(self.compute.debug(&self.device, &self.queue));
        }
        self.dt = self.frame_time.elapsed();
        self.frame_time = time::Instant::now();
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.platform.update_time(self.time.elapsed().as_secs_f64());
        let mut frame = self.render.render(
            self.encoder.as_mut().unwrap(),
            self.compute.get_particle_buffer(self.frame_count),
            &self.surface,
        );

        if !self.paused {
            self.frame_count += 1;
        }

        let mut encoder_ = self.encoder.take().unwrap();
        let tdelta = self.ui_render(&mut encoder_, &frame);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder_.finish()));
        frame.present();

        self.egui_rpass
            .remove_textures(tdelta)
            .expect("remove texture ok");
        Ok(())
    }
    pub fn ui_render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::SurfaceTexture,
    ) -> egui::TexturesDelta {
        let output_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.platform.begin_frame();
        self.demo_app.ui(&self.platform.context(), self.compute.get_history());
        let full_output = self.platform.end_frame(Some(&self.window));
        let paint_jobs = self.platform.context().tessellate(full_output.shapes);

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: self.window.scale_factor() as f32,
        };
        let tdelta: egui::TexturesDelta = full_output.textures_delta;
        self.egui_rpass
            .add_textures(&self.device, &self.queue, &tdelta)
            .expect("add texture ok");
        self.egui_rpass
            .update_buffers(&self.device, &self.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.egui_rpass
            .execute(
                encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                None,
            )
            .unwrap();

        tdelta
    }

    pub fn exit(&self) {
        self.compute.write_stats();
    }
}
