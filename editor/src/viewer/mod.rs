pub mod camera;
pub mod render_state;
pub use render_state::*;

mod vertex;

use std::sync::Arc;

use eframe::egui_wgpu;
use egui::mutex::RwLock;
use wgpu::util::DeviceExt;

use crate::{
    format::mdl0::{normals::NormalData, vertices::VertexData},
    viewer::{
        self,
        camera::{Camera, CameraController, CameraUniformData, OrbitCamera},
        vertex::{CUBE_INDICES, CUBE_VERTICES, Vertex3},
    },
};

const DEFAULT_VIEWPORT: egui::Rect =
    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));

/// The usage flags for the offscreen render texture
///
// This uses the `union` method instead of standard bit or because traits are not const right now.
pub const TARGET_USAGES: wgpu::TextureUsages =
    wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING);

pub const DEPTH_USAGES: wgpu::TextureUsages = TARGET_USAGES;

pub const SAMPLE_COUNT: u32 = 4;
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color::BLACK;
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const TEXTURE_FILTER_MODE: wgpu::FilterMode = wgpu::FilterMode::Linear;

pub struct ViewerCallback;

impl ViewerCallback {
    pub fn init(state: &viewer::RenderState) -> Self {
        let viewer = ViewerState::new(state);
        state.renderer.write().callback_resources.insert(viewer);

        tracing::trace!("Initialized model viewer");

        Self
    }

    pub fn deinit(renderer: &Arc<RwLock<egui_wgpu::Renderer>>) {
        let mut renderer = renderer.write();
        if let Some(viewer) = renderer.callback_resources.remove::<ViewerState>() {
            renderer.free_texture(&viewer.texture_data.texture_id);
        }

        tracing::trace!("Deinitialized model viewer");
    }
}

impl egui_wgpu::CallbackTrait for ViewerCallback {
    // Render the view to the texture before the UI render pass
    // so that the UI can immediately use an up to date texture.
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let viewer = resources.get::<ViewerState>().unwrap();

        let mut render_pass = egui_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("offscreen render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &viewer.texture_data.msaa_texture_view,
                depth_slice: None,
                resolve_target: Some(&viewer.texture_data.texture_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &viewer.texture_data.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&viewer.pipeline_data.render_pipeline);
        render_pass.set_vertex_buffer(0, viewer.pipeline_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            viewer.pipeline_data.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.set_bind_group(0, &viewer.camera_data.bind_group, &[]);
        render_pass.draw_indexed(0..36, 0, 0..1);

        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        _render_pass: &mut wgpu::RenderPass<'static>,
        _resources: &egui_wgpu::CallbackResources,
    ) {
    }
}

#[derive(Clone)]
pub struct TextureData {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub texture_id: egui::TextureId,

    pub msaa_texture: wgpu::Texture,
    pub msaa_texture_view: wgpu::TextureView,

    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub depth_sampler: wgpu::Sampler,
}

#[derive(Clone)]
pub struct CameraBindGroupData {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buffer: wgpu::Buffer,
}

#[derive(Clone)]
pub struct RenderPipelineData {
    pub module: wgpu::ShaderModule,

    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,
}

#[derive(Clone)]
pub struct ModelData {
    pub vertices: VertexData,
    // normals: NormalData,
}

#[derive(Clone)]
pub struct ViewerState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub model_data: Option<ModelData>,
    pub texture_data: TextureData,
    pub pipeline_data: RenderPipelineData,
    pub camera_data: CameraBindGroupData,

    // Camera rotation on a flat plane, i.e. this comes directly from dragging the cursor
    // and is converted to actual position using trigonometry.
    pub camera: Camera,
}

unsafe impl Send for ViewerState {}
unsafe impl Sync for ViewerState {}

impl ViewerState {
    pub fn viewport_size(&self) -> glam::Vec2 {
        let texture = &self.texture_data.texture;
        glam::vec2(texture.width() as f32, texture.height() as f32)
    }

    /// Updates the camera rotation/position on the GPU side of things.
    pub fn on_camera_update(&mut self) {
        if let Some(mut buffer_view) = self.queue.write_buffer_with(
            &self.camera_data.uniform_buffer,
            0,
            CameraUniformData::size(),
        ) {
            let view_proj = self.camera.compute_matrix();

            let viewport_size2 = self.viewport_size();
            let viewport_size = glam::vec4(viewport_size2.x, viewport_size2.y, 0.0, 0.0);

            buffer_view.copy_from_slice(bytemuck::bytes_of(&CameraUniformData {
                view_proj,
                viewport_size,
            }));
        }
    }

    pub fn on_model_update(&mut self) {
        let Some(model) = &self.model_data else {
            return;
        };

        tracing::trace!("Updating model view");

        self.pipeline_data.vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vertex buffer"),
                    contents: model.vertices.as_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.pipeline_data.render_pipeline =
            self.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("render pipeline"),
                    layout: Some(&self.pipeline_data.pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &self.pipeline_data.module,
                        entry_point: Some("vs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[Some(wgpu::VertexBufferLayout {
                            array_stride: (model.vertices.components() * size_of::<f32>()) as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: model.vertices.format(),
                                offset: 0,
                                shader_location: 0,
                            }],
                        })],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: DEPTH_FORMAT,
                        depth_write_enabled: Some(true),
                        depth_compare: Some(wgpu::CompareFunction::Less),
                        stencil: wgpu::StencilState {
                            front: wgpu::StencilFaceState::IGNORE,
                            back: wgpu::StencilFaceState::IGNORE,
                            read_mask: 0,
                            write_mask: 0,
                        },
                        bias: wgpu::DepthBiasState {
                            constant: 0,
                            slope_scale: 0.0,
                            clamp: 0.0,
                        },
                    }),
                    multisample: wgpu::MultisampleState {
                        count: SAMPLE_COUNT,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &self.pipeline_data.module,
                        entry_point: Some("fs_main"),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: TARGET_FORMAT,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    multiview_mask: None,
                    cache: None,
                });
    }

    fn create_textures(state: &viewer::RenderState) -> TextureData {
        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen render texture"),
            size: wgpu::Extent3d {
                width: DEFAULT_VIEWPORT.width() as u32,
                height: DEFAULT_VIEWPORT.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TARGET_USAGES,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("offscreen render view"),
            format: Some(TARGET_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(TARGET_USAGES),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let texture_id = state.renderer.write().register_native_texture(
            &state.device,
            &texture_view,
            TEXTURE_FILTER_MODE,
        );

        let msaa_texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa texture"),
            size: wgpu::Extent3d {
                width: DEFAULT_VIEWPORT.width() as u32,
                height: DEFAULT_VIEWPORT.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TARGET_USAGES,
            view_formats: &[],
        });

        let msaa_texture_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("msaa texture view"),
            format: Some(TARGET_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(TARGET_USAGES),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let depth_texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: DEFAULT_VIEWPORT.width() as u32,
                height: DEFAULT_VIEWPORT.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: DEPTH_USAGES,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("depth texture view"),
            format: Some(DEPTH_FORMAT),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: Some(DEPTH_USAGES),
            aspect: wgpu::TextureAspect::DepthOnly,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let depth_sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("depth texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            anisotropy_clamp: 1,
            border_color: None,
        });

        TextureData {
            texture,
            texture_view,
            texture_id,

            msaa_texture,
            msaa_texture_view,

            depth_texture,
            depth_view,
            depth_sampler,
        }
    }

    fn create_camera_bind_group(device: &wgpu::Device, camera: &Camera) -> CameraBindGroupData {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let uniform_data = CameraUniformData {
            viewport_size: glam::vec4(
                DEFAULT_VIEWPORT.width(),
                DEFAULT_VIEWPORT.height(),
                0.0,
                0.0,
            ),
            view_proj: camera.compute_matrix(),
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform data buffer"),
            contents: bytemuck::bytes_of(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform data bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buffer,
                    offset: 0,
                    size: Some(CameraUniformData::size()),
                }),
            }],
        });

        CameraBindGroupData {
            bind_group,
            bind_group_layout,
            uniform_buffer,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
    ) -> RenderPipelineData {
        let module =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/viewer.wgsl").into());

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("offscreen render pipeline layout"),
            bind_group_layouts,
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("texture render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Vertex3::layout())],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState::IGNORE,
                    back: wgpu::StencilFaceState::IGNORE,
                    read_mask: !0,
                    write_mask: !0,
                },
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TARGET_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(&CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        RenderPipelineData {
            module,

            pipeline_layout,
            render_pipeline,
            vertex_buffer,
            index_buffer,
        }
    }

    /// Attempts to resize the texture and returns whether the texture has actually been resized.
    ///
    /// If this function returns true, the texture view should be reregistered with egui.
    pub fn resize_viewport(&mut self, bounds: egui::Rect) -> bool {
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

        let texture = &mut self.texture_data.texture;
        if texture.width() != new_width || texture.height() != new_height {
            self.texture_data.texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("offscreen render texture"),
                size: wgpu::Extent3d {
                    width: bounds.width() as u32,
                    height: bounds.height() as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TARGET_FORMAT,
                usage: TARGET_USAGES,
                view_formats: &[],
            });

            self.texture_data.texture_view =
                self.texture_data
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some("offscreen render view"),
                        format: Some(TARGET_FORMAT),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        usage: Some(TARGET_USAGES),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    });

            self.texture_data.msaa_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("msaa texture"),
                size: wgpu::Extent3d {
                    width: bounds.width() as u32,
                    height: bounds.height() as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: TARGET_FORMAT,
                usage: TARGET_USAGES,
                view_formats: &[],
            });

            self.texture_data.msaa_texture_view =
                self.texture_data
                    .msaa_texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some("msaa texture view"),
                        format: Some(TARGET_FORMAT),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        usage: Some(TARGET_USAGES),
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    });

            self.texture_data.depth_texture =
                self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("depth texture"),
                    size: wgpu::Extent3d {
                        width: bounds.width() as u32,
                        height: bounds.height() as u32,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: SAMPLE_COUNT,
                    dimension: wgpu::TextureDimension::D2,
                    format: DEPTH_FORMAT,
                    usage: DEPTH_USAGES,
                    view_formats: &[],
                });

            self.texture_data.depth_view =
                self.texture_data
                    .depth_texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        label: Some("depth texture view"),
                        format: Some(DEPTH_FORMAT),
                        dimension: Some(wgpu::TextureViewDimension::D2),
                        usage: Some(DEPTH_USAGES),
                        aspect: wgpu::TextureAspect::DepthOnly,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: None,
                    });

            self.camera
                .set_aspect_ratio(bounds.width() / bounds.height());

            self.on_camera_update();

            return true;
        }

        false
    }

    pub fn new(state: &viewer::RenderState) -> Self {
        let device = state.device.clone();

        let camera: Camera = OrbitCamera {
            zoom_sensitivity: 0.005,
            sensitivity: 0.01,
            aspect_ratio: DEFAULT_VIEWPORT.width() / DEFAULT_VIEWPORT.height(),
            lookat: glam::Vec3::ZERO,
            vertical_fov: 90.0f32.to_radians(),
            orientation: glam::Quat::default(),
            radius: 2.0,
        }
        .into();

        let texture_data = Self::create_textures(&state);
        let camera_data = Self::create_camera_bind_group(&device, &camera);
        let pipeline_data = Self::create_pipeline(&device, &[Some(&camera_data.bind_group_layout)]);

        Self {
            device,
            queue: state.queue.clone(),

            model_data: None,

            texture_data,
            pipeline_data,
            camera_data,
            camera,
        }
    }
}
