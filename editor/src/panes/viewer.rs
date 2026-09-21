use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, mpsc},
};

use eframe::egui_wgpu;
use egui::mutex::RwLock;
use wgpu::util::DeviceExt;

use crate::{
    node::refs::{VirtualNodeId, VirtualNodeMap},
    panes::{ContentSignature, Pane, PaneAction},
    viewer::{
        GraphicsState,
        camera::{Camera, CameraController, CameraUniformData, OrbitCamera},
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

pub const MSAA_SAMPLE_COUNT: u32 = 4;
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color::BLACK;
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const TEXTURE_FILTER_MODE: wgpu::FilterMode = wgpu::FilterMode::Linear;

struct ScreenTextureState {
    output: wgpu::Texture,
    output_view: wgpu::TextureView,
    egui_texture_id: egui::TextureId,

    msaa_output: wgpu::Texture,
    msaa_output_view: wgpu::TextureView,

    depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl ScreenTextureState {
    pub fn new(state: &GraphicsState, size: glam::UVec2) -> Self {
        let output = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen render texture"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TARGET_USAGES,
            view_formats: &[],
        });

        let output_view = output.create_view(&wgpu::TextureViewDescriptor {
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

        let egui_texture_id = state.renderer.write().register_native_texture(
            &state.device,
            &output_view,
            TEXTURE_FILTER_MODE,
        );

        let msaa_output = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa texture"),
            size: wgpu::Extent3d {
                width: DEFAULT_VIEWPORT.width() as u32,
                height: DEFAULT_VIEWPORT.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TARGET_USAGES,
            view_formats: &[],
        });

        let msaa_output_view = msaa_output.create_view(&wgpu::TextureViewDescriptor {
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

        let depth = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: DEFAULT_VIEWPORT.width() as u32,
                height: DEFAULT_VIEWPORT.height() as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: DEPTH_USAGES,
            view_formats: &[],
        });

        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor {
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

        Self {
            output,
            output_view,
            egui_texture_id,

            msaa_output,
            msaa_output_view,

            depth,
            depth_view,
        }
    }

    pub fn on_resize(&mut self, state: &GraphicsState, new_size: glam::UVec2) {
        self.output = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen render texture"),
            size: wgpu::Extent3d {
                width: new_size.x,
                height: new_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TARGET_USAGES,
            view_formats: &[],
        });

        state
            .renderer
            .write()
            .update_egui_texture_from_wgpu_texture(
                &state.device,
                &self.output_view,
                TEXTURE_FILTER_MODE,
                self.egui_texture_id,
            );

        self.output_view = self.output.create_view(&wgpu::TextureViewDescriptor {
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

        self.msaa_output = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa texture"),
            size: wgpu::Extent3d {
                width: new_size.x,
                height: new_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: TARGET_USAGES,
            view_formats: &[],
        });

        self.msaa_output_view = self.msaa_output.create_view(&wgpu::TextureViewDescriptor {
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

        self.depth = state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: new_size.x,
                height: new_size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: MSAA_SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: DEPTH_USAGES,
            view_formats: &[],
        });

        self.depth_view = self.depth.create_view(&wgpu::TextureViewDescriptor {
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
    }
}

struct CameraState {
    camera: Camera,

    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,

    uniform_buffer: wgpu::Buffer,
}

impl CameraState {
    pub fn new(camera: Camera, state: &GraphicsState, viewport: glam::UVec2) -> Self {
        let bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            viewport_size: glam::vec4(viewport.x as f32, viewport.y as f32, 0.0, 0.0),
            view_proj: camera.compute_matrix(),
        };

        let uniform_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera uniform data buffer"),
                contents: bytemuck::bytes_of(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera uniform data bind group"),
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

        Self {
            camera,
            bind_group,
            bind_group_layout,
            uniform_buffer,
        }
    }

    pub fn on_resize(&mut self, new_viewport: glam::UVec2) {
        todo!()
    }
}

pub struct ViewerPipeline {
    graphics_state: GraphicsState,

    camera_state: CameraState,
    screen_texture_state: ScreenTextureState,

    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl ViewerPipeline {
    pub fn new(graphics_state: GraphicsState) -> Self {
        let viewport = glam::uvec2(800, 600);

        let camera = Camera::Orbit(OrbitCamera {
            radius: 2.0,
            sensitivity: 0.01,
            vertical_fov: 90.0,
            aspect_ratio: viewport.x as f32 / viewport.y as f32,
            zoom_sensitivity: 0.01,
            orientation: glam::Quat::default(),
            lookat: glam::Vec3::ZERO,
        });

        let camera_state = CameraState::new(camera, &graphics_state, viewport);
        let screen_texture_state = ScreenTextureState::new(&graphics_state, viewport);

        todo!("pipeline state");

        Self {
            camera_state,
            screen_texture_state,

            graphics_state,
        }
    }

    pub fn on_resize(&mut self, new_size: glam::UVec2) {
        self.screen_texture_state
            .on_resize(&self.graphics_state, new_size);

        self.camera_state.on_resize(new_size);
    }
}

pub struct ViewerPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    content_sig: ContentSignature,
    node: Option<VirtualNodeId>,
    node_map: VirtualNodeMap,

    pipeline: ViewerPipeline,
}

impl ViewerPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        content_sig: ContentSignature,
        node: Option<VirtualNodeId>,
        node_map: VirtualNodeMap,
        render_state: GraphicsState,
    ) -> Box<dyn Pane> {
        let pipeline = ViewerPipeline::new(render_state);

        Box::new(Self {
            cmd_sender,
            content_sig,
            node,
            node_map,
            pipeline,
        })
    }
}

impl Pane for ViewerPane {
    fn content_signature(&self) -> ContentSignature {
        self.content_sig
    }

    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("3D Viewer"))
    }

    fn draw(&mut self, ui: &mut egui::Ui, tile_id: egui_tiles::TileId) -> egui_tiles::UiResponse {
        // ui.label("Viewing `{}`")

        let drag_started = ui.heading("Viewer").drag_started();

        if drag_started {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}
