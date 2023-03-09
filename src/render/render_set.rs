use std::time::Duration;

use wgpu::{CommandEncoder, SurfaceTexture};
use winit::{event::WindowEvent, window::Window};

use crate::render::{
    camera::{Camera, CameraController, CameraUniform},
    vertex::{Circle, Vertex},
};

use crate::system::{consts::*, particle::Particle};

use crate::utils::buffers::Buffer;

use super::{camera::Projection, vertex::UVSphere};

pub struct RenderSet {
    size: winit::dpi::PhysicalSize<u32>,
    camera: Camera,
    camera_controller: CameraController,
    camera_projection: Projection,
    camera_uniform: CameraUniform,
    camera_bind_group: wgpu::BindGroup,
    camera_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderSet {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let size = window.inner_size();

        // buffers:

        let circle = UVSphere::new(16);
        let vertex_buffer = Buffer::new()
            .with_data(bytemuck::cast_slice(&circle.get_vertices()))
            .with_usage(wgpu::BufferUsages::VERTEX)
            .build(&device, Some("Vertex Buffer"));

        let index_buffer = Buffer::new()
            .with_data(bytemuck::cast_slice(&circle.get_indices()))
            .with_usage(wgpu::BufferUsages::INDEX)
            .build(&device, Some("Index Buffer"));

        let num_indices = circle.num_indices;

        let camera_uniform = CameraUniform::new();
        let camera_buffer = Buffer::new()
            .with_data(bytemuck::cast_slice(&[camera_uniform]))
            .with_usage(wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST)
            .build(&device, Some("Camera Buffer"));

        let camera_controller = CameraController::new(20.0, 0.05);
        // let camera = Camera::new(1.0 / BOX_SIZE);
        let camera = Camera::new((0.0, 0.0, 0.0), cgmath::Deg(-45.0), cgmath::Deg(-20.0));
        let camera_projection =
            Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 1000.0);

        // // view from the top
        // let camera = Camera::new((0.0, 0.0, 0.0), cgmath::Deg(0.0), cgmath::Deg(90.0));
        // let camera_projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.01, 100.0);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("..\\shaders\\shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Particle::desc_render()],
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

        Self {
            size,
            camera,
            camera_controller,
            camera_projection,
            camera_uniform,
            camera_bind_group,
            camera_buffer,
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        // self.camera.set_aspect_ratio(self.size.width as f32 / self.size.height as f32);
        self.camera_projection
            .resize(new_size.width, new_size.height);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, dt: Duration) {
        // self.camera_controller.update_camera(&mut self.camera, &self.size);
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.camera_projection);

        queue.write_buffer(
            &self.camera_buffer.get_buffer(),
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        particles_buffer: &wgpu::Buffer,
        surface: &wgpu::Surface,
    ) -> SurfaceTexture {
        let frame = surface
            .get_current_texture()
            .expect("Timeout when acquiring next surface texture");

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

            render_pass.set_vertex_buffer(0, self.vertex_buffer.get_buffer().slice(..));

            render_pass.set_vertex_buffer(1, particles_buffer.slice(..));

            render_pass.set_index_buffer(
                self.index_buffer.get_buffer().slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..self.num_indices, 0, 0..NUMBER_PARTICLES as _);
        }
        encoder.pop_debug_group();

        return frame;
    }
}
