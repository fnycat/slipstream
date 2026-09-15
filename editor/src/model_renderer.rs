use eframe::egui_wgpu;
use wgpu::util::DeviceExt;

const VERTICES: [[f32; 3]; 3] = [[0.0, 0.5, 0.0], [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0]];

struct ModelRendererResources {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelRenderer;

impl ModelRenderer {
    pub fn init(state: &egui_wgpu::RenderState) {
        let shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl").into());

        let pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("model render pipeline layout"),
                    bind_group_layouts: &[],
                    immediate_size: 0,
                });

        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("render pipeline"),
                layout: Some(&pipeline_layout),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    conservative: false,
                    cull_mode: Some(wgpu::Face::Back),
                    front_face: wgpu::FrontFace::Ccw,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    strip_index_format: None,
                    unclipped_depth: false,
                },
                vertex: wgpu::VertexState {
                    entry_point: Some("vs_main"),
                    module: &shader,
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<f32>() as u64 * 3,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                        step_mode: wgpu::VertexStepMode::Vertex,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.target_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    entry_point: Some("fs_main"),
                    module: &shader,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                multisample: wgpu::MultisampleState {
                    alpha_to_coverage_enabled: false,
                    count: 1,
                    mask: !0,
                },

                multiview_mask: None,
                depth_stencil: None,
                cache: None,
            });

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("model vertex buffer"),
                contents: bytemuck::cast_slice(&VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        state
            .renderer
            .write()
            .callback_resources
            .insert(ModelRendererResources {
                pipeline,
                vertex_buffer,
            });
    }
}

impl egui_wgpu::CallbackTrait for ModelRenderer {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &egui_wgpu::ScreenDescriptor,
        egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &ModelRendererResources = resources.get().unwrap();

        render_pass.set_pipeline(&resources.pipeline);
        render_pass.set_vertex_buffer(0, resources.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}
