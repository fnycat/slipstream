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

pub const MSAA_SAMPLE_COUNT: u32 = 4;
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color::BLACK;
pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const TEXTURE_FILTER_MODE: wgpu::FilterMode = wgpu::FilterMode::Linear;

struct ViewerCallback;

impl egui_wgpu::CallbackTrait for ViewerCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _screen_desc: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let viewer = callback_resources.get::<ViewerPipeline>().unwrap();
        viewer.draw(egui_encoder)
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        callback_resources: &egui_wgpu::CallbackResources,
    ) {
    }
}

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

    /// Due to the render lock being required, this function cannot update the egui texture
    /// by itself.
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
    viewport_size: glam::Vec4,

    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,

    uniform_buffer: wgpu::Buffer,
}

impl CameraState {
    pub fn new(camera: Camera, state: &GraphicsState, viewport_size: glam::UVec2) -> Self {
        let bind_group_layout = state
            .device
            .create_bind_group_layout(&CameraUniformData::layout());

        let uniform_data = CameraUniformData {
            viewport_size: glam::vec4(viewport_size.x as f32, viewport_size.y as f32, 0.0, 0.0),
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
            viewport_size: uniform_data.viewport_size,
            bind_group,
            bind_group_layout,
            uniform_buffer,
        }
    }

    pub fn update(&self, graphics_state: &GraphicsState) {
        if let Some(mut buffer_view) = graphics_state.queue.write_buffer_with(
            &self.uniform_buffer,
            0,
            CameraUniformData::size(),
        ) {
            let view_proj = self.camera.compute_matrix();
            buffer_view.copy_from_slice(bytemuck::bytes_of(&CameraUniformData {
                view_proj,
                viewport_size: self.viewport_size,
            }))
        }
    }
}

struct PipelineState {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
}

impl PipelineState {
    pub fn new(state: &GraphicsState, camera_state: &CameraState) -> Self {
        let shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("../../shaders/viewer.wgsl").into());

        let pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("viewer pipeline layout"),
                    bind_group_layouts: &[Some(&camera_state.bind_group_layout)],
                    immediate_size: 0,
                });

        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("viewer render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(Vertex3::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
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
                        clamp: 0.0,
                        constant: 0,
                        slope_scale: 0.0,
                    },
                }),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TARGET_FORMAT,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multisample: wgpu::MultisampleState {
                    count: MSAA_SAMPLE_COUNT,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        Self {
            pipeline_layout,
            pipeline,
        }
    }
}

struct ModelState {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl ModelState {
    pub fn new(state: &GraphicsState) -> Self {
        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewer vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX,
                contents: bytemuck::cast_slice(&CUBE_VERTICES),
            });

        let index_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("viewer index buffer"),
                usage: wgpu::BufferUsages::INDEX,
                contents: bytemuck::cast_slice(&CUBE_INDICES),
            });

        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

pub struct ViewerPipeline {
    graphics_state: GraphicsState,

    camera_state: CameraState,
    screen_texture_state: ScreenTextureState,
    pipeline_state: PipelineState,
    model_state: ModelState,

    viewport_size: glam::UVec2,
}

impl ViewerPipeline {
    pub fn new(graphics_state: GraphicsState) -> Self {
        let viewport_size = glam::uvec2(800, 600);

        let camera = Camera::Orbit(OrbitCamera {
            radius: 2.0,
            sensitivity: 0.01,
            vertical_fov: 90.0,
            aspect_ratio: viewport_size.x as f32 / viewport_size.y as f32,
            zoom_sensitivity: 0.01,
            orientation: glam::Quat::default(),
            lookat: glam::Vec3::ZERO,
        });

        let camera_state = CameraState::new(camera, &graphics_state, viewport_size);
        let screen_texture_state = ScreenTextureState::new(&graphics_state, viewport_size);
        let pipeline_state = PipelineState::new(&graphics_state, &camera_state);

        let model_state = ModelState::new(&graphics_state);

        Self {
            camera_state,
            screen_texture_state,
            pipeline_state,
            model_state,

            viewport_size,
            graphics_state,
        }
    }

    pub fn update_size(&mut self, new_size: glam::UVec2) -> bool {
        if new_size.x == 0 || new_size.y == 0 {
            return false;
        }

        if new_size != self.viewport_size {
            self.screen_texture_state
                .on_resize(&self.graphics_state, new_size);

            self.camera_state.viewport_size =
                glam::vec4(new_size.x as f32, new_size.y as f32, 0.0, 0.0);

            self.camera_state.update(&self.graphics_state);

            return true;
        }

        false
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder) -> Vec<wgpu::CommandBuffer> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("viewer render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.screen_texture_state.msaa_output_view,
                depth_slice: None,
                resolve_target: Some(&self.screen_texture_state.output_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.screen_texture_state.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        render_pass.set_pipeline(&self.pipeline_state.pipeline);
        render_pass.set_bind_group(0, &self.camera_state.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.model_state.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.model_state.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..36, 0, 0..1);

        Vec::new()
    }
}

pub struct ViewerPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    content_sig: ContentSignature,
    node: Option<VirtualNodeId>,
    node_map: VirtualNodeMap,

    render_state: GraphicsState,
}

impl ViewerPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        content_sig: ContentSignature,
        node: Option<VirtualNodeId>,
        node_map: VirtualNodeMap,
        render_state: GraphicsState,
    ) -> Box<dyn Pane> {
        let pipeline = ViewerPipeline::new(render_state.clone());
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(pipeline);

        tracing::trace!("Viewer pipeline initialized");

        Box::new(Self {
            cmd_sender,
            content_sig,
            node,
            node_map,
            render_state,
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

    fn draw(&mut self, ui: &mut egui::Ui, _tile_id: egui_tiles::TileId) -> egui_tiles::UiResponse {
        let drag_started = ui.heading("3D Viewer").drag_started();

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let target_size = ui.available_size();
            let panel_bounds = egui::Rect::from_min_size(ui.cursor().min, target_size);

            let mut renderer = self.render_state.renderer.write();
            let mut pipeline = renderer
                .callback_resources
                .get_mut::<ViewerPipeline>()
                .unwrap();

            let has_resized =
                pipeline.update_size(glam::uvec2(target_size.x as u32, target_size.y as u32));

            if has_resized {
                // Create copies to temporarily drop the pipeline borrow.
                //
                // This allows us to mutably borrow renderer, which would otherwise be borrowed
                // by the viewer pipeline.
                let output_view = pipeline.screen_texture_state.output_view.clone();
                let egui_tex_id = pipeline.screen_texture_state.egui_texture_id;

                renderer.update_egui_texture_from_wgpu_texture(
                    &self.render_state.device,
                    &output_view,
                    TEXTURE_FILTER_MODE,
                    egui_tex_id,
                );

                // Then reborrow the pipeline.
                pipeline = renderer
                    .callback_resources
                    .get_mut::<ViewerPipeline>()
                    .unwrap();
            }

            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                panel_bounds,
                ViewerCallback,
            ));

            let image_widget = egui::Image::new(egui::load::SizedTexture {
                id: pipeline.screen_texture_state.egui_texture_id,
                size: panel_bounds.size(),
            })
            .sense(egui::Sense::click_and_drag());

            // Camera
            // =============================================================================

            pipeline
                .camera_state
                .camera
                .set_aspect_ratio(target_size.x / target_size.y);

            // If the screen has resized, we need to update to adjust the aspect ratio.
            let mut camera_updated = has_resized;

            let response = ui.add(image_widget);
            if response.dragged() {
                let drag_delta = response.drag_delta();

                pipeline
                    .camera_state
                    .camera
                    .drag_delta(glam::vec2(drag_delta.x, drag_delta.y));

                camera_updated = true;
            }

            ui.input(|i| {
                if i.is_scrolling() && response.contains_pointer() {
                    let scroll_delta = i.smooth_scroll_delta();

                    pipeline
                        .camera_state
                        .camera
                        .scroll_delta(glam::vec2(scroll_delta.x, scroll_delta.y));

                    camera_updated = true;
                }
            });

            if camera_updated {
                pipeline.camera_state.update(&self.render_state);
            }
        });

        if drag_started {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}
