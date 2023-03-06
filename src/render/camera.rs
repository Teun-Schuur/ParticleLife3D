use cgmath::{SquareMatrix, InnerSpace};
use winit::event::{WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use cgmath::*;
use winit::event::*;
use winit::dpi::PhysicalPosition;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;


#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<
        V: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            Vector3::unit_y(),
        )
    }
}


pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}
 



#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_position: [f32; 4],
}
unsafe impl bytemuck::Pod for CameraUniform {}
unsafe impl bytemuck::Zeroable for CameraUniform {}

impl CameraUniform {
    pub fn new() -> Self {
        // use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_position: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into();
    }
}



#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
    mouse_pressed: bool,
    just_mouse_pressed: bool,
    mouse_last_position: PhysicalPosition<f64>,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
            mouse_pressed: false,
            just_mouse_pressed: false,
            mouse_last_position: PhysicalPosition::new(0.0, 0.0),
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.process_keyboard(*key, *state),
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                if self.mouse_pressed && !self.just_mouse_pressed {
                    self.just_mouse_pressed = true;
                }
                true
            },
            WindowEvent::MouseWheel { delta, .. } => {
                self.process_scroll(delta);
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.process_mouse(position.x, position.y);
                true
            }
            
            _ => false,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        if !self.mouse_pressed {
            return;
        }
        if self.just_mouse_pressed {
            self.mouse_last_position = PhysicalPosition::new(mouse_dx, mouse_dy);
            self.just_mouse_pressed = false;
            return;
        }
        let dx = mouse_dx - self.mouse_last_position.x;
        let dy = mouse_dy - self.mouse_last_position.y;
        self.mouse_last_position = PhysicalPosition::new(mouse_dx, mouse_dy);
        self.rotate_horizontal = dx as f32;
        self.rotate_vertical = dy as f32;
        // println!("mouse_dx: {}, mouse_dy: {}", dx, dy)
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 300.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}





// pub struct CameraController {
//     move_speed: f32,
//     zoom_speed: f32,
//     is_forward_pressed: bool,
//     is_backward_pressed: bool,
//     is_left_pressed: bool,
//     is_right_pressed: bool,
//     is_shift_pressed: bool,
//     is_control_pressed: bool,
//     mouse_position: [f32; 2],
//     mouse_last_position: [f32; 2],
//     mouse_wheel_delta: f32,
//     mouse_left_pressed: bool,
// }


// impl CameraController {
//     pub fn new(move_speed: f32, zoom_speed: f32) -> Self {
//         Self {
//             move_speed,
//             zoom_speed,
//             is_forward_pressed: false,
//             is_backward_pressed: false,
//             is_left_pressed: false,
//             is_right_pressed: false,
//             is_shift_pressed: false,
//             is_control_pressed: false,
//             mouse_position: [0.0, 0.0],
//             mouse_last_position: [0.0, 0.0],
//             mouse_wheel_delta: 0.0,
//             mouse_left_pressed: false,
//         }
//     }

//     pub fn process_events(&mut self, event: &WindowEvent) -> bool {
//         match event {
//             WindowEvent::KeyboardInput {
//                 input: KeyboardInput {
//                     state,
//                     virtual_keycode: Some(keycode),
//                     ..
//                 },
//                 ..
//             } => {
//                 let is_pressed = *state == ElementState::Pressed;
//                 match keycode {
//                     VirtualKeyCode::W | VirtualKeyCode::Up => {
//                         self.is_forward_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::A | VirtualKeyCode::Left => {
//                         self.is_left_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::S | VirtualKeyCode::Down => {
//                         self.is_backward_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::D | VirtualKeyCode::Right => {
//                         self.is_right_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::LShift => {
//                         self.is_shift_pressed = is_pressed;
//                         true
//                     }
//                     VirtualKeyCode::LControl => {
//                         self.is_control_pressed = is_pressed;
//                         true
//                     }
//                     _ => false,
//                 }
//             }
//             // mouse left button pressed
//             WindowEvent::MouseInput { state, button, .. } => {
//                 if *button == winit::event::MouseButton::Left {
//                     self.mouse_left_pressed = *state == ElementState::Pressed;
//                     true
//                 } else {
//                     false
//                 }
//             }
//             WindowEvent::MouseWheel { delta, .. } => {
//                 match delta {
//                     winit::event::MouseScrollDelta::LineDelta(_, y) => {
//                         self.mouse_wheel_delta = *y;
//                         true
//                     }
//                     winit::event::MouseScrollDelta::PixelDelta(pos) => {
//                         self.mouse_wheel_delta = pos.y as f32;
//                         true
//                     }
//                 }
//             }
//             // mouse clicked and moved
//             WindowEvent::CursorMoved { position, .. } => {
//                 self.mouse_position = [position.x as f32, position.y as f32];
//                 true
//             }
            
//             _ => false,
//         }
//     }

//     pub fn update_camera(&mut self, camera: &mut Camera, win_size: &winit::dpi::PhysicalSize<u32>) {
        
//         let mut direction = [0.0, 0.0];
//         if self.is_forward_pressed {
//             direction[1] += 1.0;
//         }
//         if self.is_backward_pressed {
//             direction[1] -= 1.0;
//         }
//         if self.is_left_pressed {
//             direction[0] -= 1.0;
//         }
//         if self.is_right_pressed {
//             direction[0] += 1.0;
//         }
//         if self.is_shift_pressed {
//             camera.zoom *= 1.0 + self.zoom_speed;
//         }
//         if self.is_control_pressed {
//             camera.zoom *= 1.0 - self.zoom_speed;
//         }
//         if self.mouse_left_pressed {
//             let delta_x = (self.mouse_position[0] - self.mouse_last_position[0]) / win_size.width as f32 * 2.0;
//             let delta_y = (self.mouse_position[1] - self.mouse_last_position[1]) / win_size.height as f32 * 2.0;
//             // println!("delta_x: {}, delta_y: {}", delta_x, delta_y);
//             camera.position[0] -= delta_x / camera.zoom;
//             camera.position[1] += delta_y / camera.zoom;
//         }
//         if self.mouse_wheel_delta != 0.0 {
//             camera.zoom *= 1.0 + self.mouse_wheel_delta * self.zoom_speed * 3.0;
//         }

//         if direction != [0.0, 0.0] {
//             let direction = cgmath::Vector2::from(direction).normalize();
//             camera.position[0] += direction.x * self.move_speed / camera.zoom;
//             camera.position[1] += direction.y * self.move_speed / camera.zoom;
//         }

//         self.mouse_last_position = self.mouse_position;
//         self.mouse_wheel_delta = 0.0;
//     }
// }
 