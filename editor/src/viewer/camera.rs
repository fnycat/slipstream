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
pub enum CameraKind {
    Orbit(OrbitCamera),
}

impl CameraKind {
    pub fn get_uniform(&self) -> CameraUniformData {
        match self {
            Self::Orbit(x) => x.get_uniform(),
        }
    }

    pub fn as_orbit(&self) -> &OrbitCamera {
        match self {
            Self::Orbit(x) => x,
            _ => panic!("camera is not an orbit camera"),
        }
    }

    pub fn as_orbit_mut(&mut self) -> &mut OrbitCamera {
        match self {
            Self::Orbit(x) => x,
            _ => panic!("camera is not an orbit camera"),
        }
    }

    pub fn set_viewport(&mut self, viewport: glam::Vec2) {
        match self {
            Self::Orbit(x) => x.viewport = viewport,
        }
    }
}

impl From<OrbitCamera> for CameraKind {
    fn from(value: OrbitCamera) -> Self {
        Self::Orbit(value)
    }
}

/// A camera orbiting around a given point.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitCamera {
    pub orientation: glam::Quat,
    pub sensitivity: f32,
    pub vertical_fov: f32,
    pub viewport: glam::Vec2,
    pub lookat: glam::Vec3,
    pub radius: f32,
}

impl OrbitCamera {
    /// Generates the uniform buffer data for the current camera state.
    pub fn get_uniform(&self) -> CameraUniformData {
        let mat = self.proj_matrix() * self.camera_matrix();

        CameraUniformData {
            viewport_size: glam::vec4(self.viewport.x, self.viewport.y, 0.0, 0.0),
            view_proj: mat,
        }
    }

    /// Rotates the camera using the given drag delta.
    pub fn drag_delta(&mut self, delta: glam::Vec2) {
        let yaw_rot = glam::Quat::from_axis_angle(UP_AXIS, delta.x * self.sensitivity);

        let right = self.orientation * glam::Vec3::X;
        let pitch_rot = glam::Quat::from_axis_angle(right, -delta.y * self.sensitivity);

        self.orientation = (yaw_rot * pitch_rot * self.orientation).normalize();
    }

    /// Computes the matrix to produce the given camera orientation.
    pub fn camera_matrix(&self) -> glam::Mat4 {
        let eye = self.lookat + self.orientation * (glam::Vec3::Z * self.radius);
        let up = self.orientation * glam::Vec3::Y;
        glam::camera::lh::view::look_at_mat4(eye, self.lookat, up)
    }

    /// Computes the matrix for the projection onto the screen.
    pub fn proj_matrix(&self) -> glam::Mat4 {
        glam::camera::lh::proj::directx::perspective(
            self.vertical_fov,
            self.viewport.x / self.viewport.y,
            NEAR_PLANE,
            FAR_PLANE,
        )
    }
}
