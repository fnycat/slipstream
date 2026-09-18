pub mod camera;

use eframe::epaint::mutex::RwLockWriteGuard;
use std::{num::NonZeroU64, sync::Arc};

use eframe::egui_wgpu;
use wgpu::util::DeviceExt;

use crate::viewer::camera::{CameraKind, CameraUniformData, OrbitCamera};

// const VERTICES: [[f32; 2]; 6] = [
//     [-1.0, -1.0],
//     [1.0, 1.0],
//     [-1.0, 1.0],
//     [1.0, -1.0],
//     [1.0, 1.0],
//     [-1.0, -1.0],
// ];

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    coordinates: [f32; 3],
}

impl Vertex {
    pub const fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }
    }
}

const VERTICES: [Vertex; 8] = [
    Vertex {
        coordinates: [-0.5, -0.5, 0.5],
    }, // 0: Bottom-left-front
    Vertex {
        coordinates: [0.5, -0.5, 0.5],
    }, // 1: Bottom-right-front
    Vertex {
        coordinates: [0.5, 0.5, 0.5],
    }, // 2: Top-right-front
    Vertex {
        coordinates: [-0.5, 0.5, 0.5],
    }, // 3: Top-left-front
    Vertex {
        coordinates: [-0.5, -0.5, -0.5],
    }, // 4: Bottom-left-back
    Vertex {
        coordinates: [0.5, -0.5, -0.5],
    }, // 5: Bottom-right-back
    Vertex {
        coordinates: [0.5, 0.5, -0.5],
    }, // 6: Top-right-back
    Vertex {
        coordinates: [-0.5, 0.5, -0.5],
    }, // 7: Top-left-back
];

const INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // front
    1, 5, 6, 6, 2, 1, // right
    5, 4, 7, 7, 6, 5, // back
    4, 0, 3, 3, 7, 4, // left
    3, 2, 6, 6, 7, 3, // top
    4, 5, 1, 1, 0, 4, // bottom
];

const DEFAULT_PANEL_SIZE: egui::Rect =
    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));

/// The usage flags for the offscreen render texture
///
// This uses the `union` method instead of standard bit or because traits are not const right now.
pub const OFFSCREEN_USAGE: wgpu::TextureUsages =
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING);

pub const OFFSCREEN_CLEAR_COLOR: wgpu::Color = wgpu::Color::BLACK;
pub const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const OFFSCREEN_FILTER_MODE: wgpu::FilterMode = wgpu::FilterMode::Linear;

pub struct ViewerCallback;

impl ViewerCallback {
    pub fn init(state: &egui_wgpu::RenderState) -> Self {
        let viewer = Viewer::new(state);
        state.renderer.write().callback_resources.insert(viewer);

        Self
    }
}

impl egui_wgpu::CallbackTrait for ViewerCallback {
    // Render the view to the texture before the UI render pass
    // so that the UI can immediately use an up to date texture.
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let viewer = resources.get::<Viewer>().unwrap();

        let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("offscreen render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &viewer.texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(OFFSCREEN_CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&viewer.render_pipeline);
        render_pass.set_vertex_buffer(0, viewer.vertex_buffer.slice(..));
        render_pass.set_index_buffer(viewer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.set_bind_group(0, &viewer.uniform_bind_group, &[]);
        render_pass.draw_indexed(0..36, 0, 0..1);

        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
    }
}

#[derive(Clone)]
pub struct Viewer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub texture_id: egui::TextureId,

    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,

    pub data_buffer: wgpu::Buffer,
    pub uniform_bind_layout: wgpu::BindGroupLayout,
    pub uniform_bind_group: wgpu::BindGroup,

    // Camera rotation on a flat plane, i.e. this comes directly from dragging the cursor
    // and is converted to actual position using trigonometry.
    pub camera: CameraKind,
}

impl Viewer {
    /// Updates the camera rotation/position on the GPU side of things.
    pub fn update_camera(&mut self) {
        let camera_uniform_data = self.camera.get_uniform();
        if let Some(mut buffer_view) =
            self.queue
                .write_buffer_with(&self.data_buffer, 0, CameraUniformData::size())
        {
            buffer_view.copy_from_slice(bytemuck::bytes_of(&camera_uniform_data));
        }
    }

    /// Attempts to resize the texture and returns whether the texture has actually been resized.
    ///
    /// If this function returns true, the texture view should be reregistered with egui.
    pub fn resize_render_texture(&mut self, bounds: egui::Rect) -> bool {
        let new_width = bounds.width() as u32;
        let new_height = bounds.height() as u32;

        // Sometimes egui sends a texture width of zero. This happens for a single frame
        // when opening the inspector panel for example. The proper texture size seems
        // to always be sent afterwards so we just ignore it.
        if new_width == 0 || new_height == 0 {
            tracing::warn!(
                "egui requested viewer with incorrect dimensions ({new_width}x{new_height}), skipping this resize"
            );
            return false;
        }

        if self.texture.width() != new_width || self.texture.height() != new_height {
            tracing::trace!(
                "Resizing texture from {}x{} to {new_width}x{new_height}",
                self.texture.width(),
                self.texture.height()
            );

            self.texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("offscreen render texture"),
                size: wgpu::Extent3d {
                    width: bounds.width() as u32,
                    height: bounds.height() as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: OFFSCREEN_FORMAT,
                usage: OFFSCREEN_USAGE,
                view_formats: &[],
            });

            self.texture_view = self.texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("offscreen render view"),
                format: Some(OFFSCREEN_FORMAT),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(OFFSCREEN_USAGE),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            self.camera
                .set_viewport(glam::vec2(bounds.width(), bounds.height()));

            self.update_camera();

            return true;
        }

        false
    }

    pub fn new(state: &egui_wgpu::RenderState) -> Self {
        let device = state.device.clone();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen render texture"),
            size: wgpu::Extent3d {
                width: DEFAULT_PANEL_SIZE.width() as u32,
                height: DEFAULT_PANEL_SIZE.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: OFFSCREEN_FORMAT,
            usage: OFFSCREEN_USAGE,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("offscreen render view"),
            format: Some(OFFSCREEN_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(OFFSCREEN_USAGE),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let texture_id = state.renderer.write().register_native_texture(
            &device,
            &texture_view,
            OFFSCREEN_FILTER_MODE,
        );

        let shader =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/viewer.wgsl").into());

        let uniform_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform data bind group"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(CameraUniformData::size()),
                    },
                    count: None,
                }],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("offscreen render pipeline layout"),
            bind_group_layouts: &[Some(&uniform_bind_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Vertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: OFFSCREEN_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera: CameraKind = OrbitCamera {
            sensitivity: 0.01,
            viewport: glam::vec2(DEFAULT_PANEL_SIZE.width(), DEFAULT_PANEL_SIZE.height()),
            lookat: glam::Vec3::ZERO,
            vertical_fov: 90.0f32.to_radians(),
            orientation: glam::Quat::default(),
            radius: 2.0,
        }
        .into();

        let uniform_data = camera.get_uniform();
        let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform data buffer"),
            contents: bytemuck::bytes_of(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform data bind group"),
            layout: &uniform_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &data_buffer,
                    offset: 0,
                    size: Some(CameraUniformData::size()),
                }),
            }],
        });

        Self {
            device,
            queue: state.queue.clone(),

            pipeline_layout,
            render_pipeline,
            texture,
            texture_view,
            texture_id,
            data_buffer,
            vertex_buffer,
            index_buffer,
            uniform_bind_group,
            uniform_bind_layout,

            camera,
        }
    }
}
