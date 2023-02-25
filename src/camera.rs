use cgmath::{SquareMatrix, InnerSpace};
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};

// 2D camera
pub struct Camera {
    pub position: [f32; 2],
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            zoom: 0.03,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        use cgmath::{Matrix4, Vector3};

        let scale = Matrix4::from_nonuniform_scale(self.zoom, self.zoom, 1.0);
        let translate = Matrix4::from_translation(Vector3::new(
            -self.position[0],
            -self.position[1],
            0.0,
        ));
        scale * translate
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}
unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}

impl CameraUniform {
    pub fn new() -> Self {
        // use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = (camera.build_view_projection_matrix()).into();
    }
}


pub struct CameraController {
    move_speed: f32,
    zoom_speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_shift_pressed: bool,
    is_control_pressed: bool,
    mouse_position: [f32; 2],
    mouse_last_position: [f32; 2],
    mouse_wheel_delta: f32,
    mouse_left_pressed: bool,
}

impl CameraController {
    pub fn new(move_speed: f32, zoom_speed: f32) -> Self {
        Self {
            move_speed,
            zoom_speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_shift_pressed: false,
            is_control_pressed: false,
            mouse_position: [0.0, 0.0],
            mouse_last_position: [0.0, 0.0],
            mouse_wheel_delta: 0.0,
            mouse_left_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
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
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_shift_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LControl => {
                        self.is_control_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            // mouse left button pressed
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == winit::event::MouseButton::Left {
                    self.mouse_left_pressed = *state == ElementState::Pressed;
                    true
                } else {
                    false
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        self.mouse_wheel_delta = *y;
                        true
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.mouse_wheel_delta = pos.y as f32;
                        true
                    }
                }
            }
            // mouse clicked and moved
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = [position.x as f32, position.y as f32];
                true
            }
            
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, win_size: &winit::dpi::PhysicalSize<u32>) {
        
        let mut direction = [0.0, 0.0];
        if self.is_forward_pressed {
            direction[1] += 1.0;
        }
        if self.is_backward_pressed {
            direction[1] -= 1.0;
        }
        if self.is_left_pressed {
            direction[0] -= 1.0;
        }
        if self.is_right_pressed {
            direction[0] += 1.0;
        }
        if self.is_shift_pressed {
            camera.zoom *= 1.0 + self.zoom_speed;
        }
        if self.is_control_pressed {
            camera.zoom *= 1.0 - self.zoom_speed;
        }
        if self.mouse_left_pressed {
            let delta_x = (self.mouse_position[0] - self.mouse_last_position[0]) / win_size.width as f32 * 2.0;
            let delta_y = (self.mouse_position[1] - self.mouse_last_position[1]) / win_size.height as f32 * 2.0;
            // println!("delta_x: {}, delta_y: {}", delta_x, delta_y);
            camera.position[0] -= delta_x / camera.zoom;
            camera.position[1] += delta_y / camera.zoom;
        }
        if self.mouse_wheel_delta != 0.0 {
            camera.zoom *= 1.0 + self.mouse_wheel_delta * self.zoom_speed * 3.0;
        }

        if direction != [0.0, 0.0] {
            let direction = cgmath::Vector2::from(direction).normalize();
            camera.position[0] += direction.x * self.move_speed / camera.zoom;
            camera.position[1] += direction.y * self.move_speed / camera.zoom;
        }

        self.mouse_last_position = self.mouse_position;
        self.mouse_wheel_delta = 0.0;
    }
}
 