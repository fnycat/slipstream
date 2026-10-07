use std::time::Instant;

use crate::viewer::camera::{CameraController, FAR_PLANE, NEAR_PLANE, WORLD_UP};

/// A camera orbiting around a given point.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitCamera {
    pub zoom_sensitivity: f32,
    pub sensitivity: f32,
    pub move_speed: f32,
    pub vertical_fov: f32,
    pub aspect_ratio: f32,

    pub last_update: Instant,
    pub orientation: glam::Quat,
    pub lookat: glam::Vec3,
    pub radius: f32,
}

impl CameraController for OrbitCamera {
    #[inline]
    fn on_update(&mut self) {
        self.last_update = Instant::now();
    }

    #[inline]
    fn delta_time(&self) -> f32 {
        let now = Instant::now();
        now.duration_since(self.last_update).as_secs_f32()
    }

    #[inline]
    fn set_fov(&mut self, fov: f32) {
        self.vertical_fov = fov;
    }

    #[inline]
    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    fn on_scroll(&mut self, delta: f32) {
        self.radius -= delta * self.zoom_sensitivity * self.delta_time();
    }

    fn on_drag(&mut self, delta: glam::Vec2) {
        let yaw_rot = glam::Quat::from_axis_angle(WORLD_UP, delta.x * self.sensitivity);

        let right = self.orientation * glam::Vec3::X;
        let pitch_rot = glam::Quat::from_axis_angle(right, -delta.y * self.sensitivity);

        self.orientation = (yaw_rot * pitch_rot * self.orientation * self.delta_time()).normalize();
    }

    fn on_move(&mut self, delta: glam::Vec3) {
        self.lookat += self.move_speed * delta * self.delta_time();
    }

    fn compute_matrix(&self) -> glam::Mat4 {
        let eye = self.lookat + self.orientation * (glam::Vec3::Z * self.radius);
        let up = self.orientation * WORLD_UP;
        let view_matrix = glam::camera::lh::view::look_at_mat4(eye, self.lookat, up);

        let proj_matrix = glam::camera::lh::proj::directx::perspective(
            self.vertical_fov,
            self.aspect_ratio,
            NEAR_PLANE,
            FAR_PLANE,
        );

        proj_matrix * view_matrix
    }
}
