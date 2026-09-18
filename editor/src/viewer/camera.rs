use std::num::NonZeroU64;

const UP_AXIS: glam::Vec3 = glam::vec3(0.0, 1.0, 0.0);
const NEAR_PLANE: f32 = 0.1;
const FAR_PLANE: f32 = 100000.0;
const MIN_ORBITAL_PITCH: f32 = -89.0f32.to_radians();
const MAX_ORBITAL_PITCH: f32 = 89.0f32.to_radians();

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniformData {
    pub viewport_size: glam::Vec4, // vec4 is required here due to padding
    pub view_proj: glam::Mat4,
}

impl CameraUniformData {
    pub const fn size() -> NonZeroU64 {
        NonZeroU64::new(std::mem::size_of::<Self>() as u64).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Camera {
    Orbit(OrbitCamera),
}

impl Camera {
    pub fn as_orbit(&self) -> &OrbitCamera {
        match self {
            Self::Orbit(x) => x,
        }
    }

    pub fn as_orbit_mut(&mut self) -> &mut OrbitCamera {
        match self {
            Self::Orbit(x) => x,
        }
    }
}

impl CameraController for Camera {
    fn set_fov(&mut self, fov: f32) {
        match self {
            Self::Orbit(x) => x.set_fov(fov),
        }
    }

    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        match self {
            Self::Orbit(x) => x.set_aspect_ratio(aspect_ratio),
        }
    }

    fn scroll_delta(&mut self, delta: glam::Vec2) {
        match self {
            Self::Orbit(x) => x.scroll_delta(delta),
        }
    }

    fn drag_delta(&mut self, delta: glam::Vec2) {
        match self {
            Self::Orbit(x) => x.drag_delta(delta),
        }
    }

    fn compute_matrix(&self) -> glam::Mat4 {
        match self {
            Self::Orbit(x) => x.compute_matrix(),
        }
    }
}

impl From<OrbitCamera> for Camera {
    fn from(value: OrbitCamera) -> Self {
        Self::Orbit(value)
    }
}

pub trait CameraController {
    fn set_fov(&mut self, fov: f32);
    fn set_aspect_ratio(&mut self, aspect_ratio: f32);
    fn drag_delta(&mut self, delta: glam::Vec2);
    fn scroll_delta(&mut self, delta: glam::Vec2);
    fn compute_matrix(&self) -> glam::Mat4;
}

/// A camera orbiting around a given point.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitCamera {
    pub orientation: glam::Quat,
    pub zoom_sensitivity: f32,
    pub sensitivity: f32,
    pub vertical_fov: f32,
    pub aspect_ratio: f32,
    pub lookat: glam::Vec3,
    pub radius: f32,
}

impl CameraController for OrbitCamera {
    fn set_fov(&mut self, fov: f32) {
        self.vertical_fov = fov;
    }

    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    fn scroll_delta(&mut self, delta: glam::Vec2) {
        self.radius -= delta.y * self.zoom_sensitivity * self.radius;
    }

    fn drag_delta(&mut self, delta: glam::Vec2) {
        let yaw_rot = glam::Quat::from_axis_angle(UP_AXIS, delta.x * self.sensitivity);

        let right = self.orientation * glam::Vec3::X;
        let pitch_rot = glam::Quat::from_axis_angle(right, -delta.y * self.sensitivity);

        self.orientation = (yaw_rot * pitch_rot * self.orientation).normalize();
    }

    fn compute_matrix(&self) -> glam::Mat4 {
        let eye = self.lookat + self.orientation * (glam::Vec3::Z * self.radius);
        let up = self.orientation * glam::Vec3::Y;
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
