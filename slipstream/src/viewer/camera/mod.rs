use std::num::NonZeroU64;

mod free;
mod orbit;

pub use free::*;
pub use orbit::*;

const WORLD_UP: glam::Vec3 = glam::Vec3::Y;
const NEAR_PLANE: f32 = 1.0;
const FAR_PLANE: f32 = 100_000.0;

/// The camera data that is sent to the GPU.
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct CameraUniformData {
    /// Current dimensions of the pane that the view is being drawn in.
    /// This does not have to be the entire physical window.
    pub viewport_size: glam::Vec4,
    /// The combined view and projection matrices of the camera.
    ///
    /// This already includes camera movement.
    pub view_proj: glam::Mat4,
    /// The inverse of [`view_proj`].
    ///
    /// [`view_proj`]: CameraUniformData::view_proj
    pub inverse_view_proj: glam::Mat4,
}

impl CameraUniformData {
    /// The size in bytes of this uniform block.
    pub const SIZE: NonZeroU64 = NonZeroU64::new(std::mem::size_of::<Self>() as u64).unwrap();

    /// The size in bytes of this uniform block.
    pub const fn size() -> NonZeroU64 {
        Self::SIZE
    }

    /// Returns the wgpu bind group layout for this uniform.
    pub const fn layout() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(Self::SIZE),
                },
                count: None,
            }],
        }
    }
}

/// Represents some kind of camera.
#[derive(Debug, Clone, PartialEq)]
pub enum Camera {
    /// An orbital camera that looks at a specific point and rotates around it.
    Orbit(OrbitCamera),
    /// A camera that can freely fly around.
    Free(FreeCamera),
}

impl CameraController for Camera {
    fn on_update(&mut self) {
        match self {
            Self::Orbit(x) => x.on_update(),
            Self::Free(x) => x.on_update(),
        }
    }

    fn delta_time(&self) -> f32 {
        match self {
            Self::Orbit(x) => x.delta_time(),
            Self::Free(x) => x.delta_time(),
        }
    }

    fn set_fov(&mut self, fov: f32) {
        match self {
            Self::Orbit(x) => x.set_fov(fov),
            Self::Free(x) => x.set_fov(fov),
        }
    }

    fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        match self {
            Self::Orbit(x) => x.set_aspect_ratio(aspect_ratio),
            Self::Free(x) => x.set_aspect_ratio(aspect_ratio),
        }
    }

    fn on_scroll(&mut self, delta: f32) {
        match self {
            Self::Orbit(x) => x.on_scroll(delta),
            Self::Free(x) => x.on_scroll(delta),
        }
    }

    fn on_drag(&mut self, delta: glam::Vec2) {
        match self {
            Self::Orbit(x) => x.on_drag(delta),
            Self::Free(x) => x.on_drag(delta),
        }
    }

    fn on_move(&mut self, delta: glam::Vec3) {
        match self {
            Self::Orbit(x) => x.on_move(delta),
            Self::Free(x) => x.on_move(delta),
        }
    }

    fn compute_matrix(&self) -> glam::Mat4 {
        match self {
            Self::Orbit(x) => x.compute_matrix(),
            Self::Free(x) => x.compute_matrix(),
        }
    }
}

impl From<OrbitCamera> for Camera {
    fn from(value: OrbitCamera) -> Self {
        Self::Orbit(value)
    }
}

impl From<FreeCamera> for Camera {
    fn from(value: FreeCamera) -> Self {
        Self::Free(value)
    }
}

pub trait CameraController {
    /// Tells the camera it has updated. This is used to keep track of delta time.
    fn on_update(&mut self);
    /// Computes the time since last update.
    fn delta_time(&self) -> f32;
    /// Sets the vertical FOV of the camera to the specified value.
    fn set_fov(&mut self, fov: f32);
    /// Sets the aspect ratio of the camera. This should be set when the viewport is resized
    /// to prevent warping of the output image.
    fn set_aspect_ratio(&mut self, aspect_ratio: f32);
    /// Called when the cursor drags across the viewport.
    fn on_drag(&mut self, delta: glam::Vec2);
    /// Called when the scroll wheel is used or when a pinch gesture is made (on touchscreens and touchpads).
    fn on_scroll(&mut self, delta: f32);
    /// Called when the camera should move. Generally this is called on WASD inputs.
    fn on_move(&mut self, delta: glam::Vec3);
    /// Computes the camera transformation matrix.
    fn compute_matrix(&self) -> glam::Mat4;
}
