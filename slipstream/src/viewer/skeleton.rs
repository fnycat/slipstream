use slipstream_shared::SlipstreamResult;

use crate::viewer::pipeline::{DEPTH_FORMAT, MSAA_SAMPLE_COUNT, TARGET_FORMAT};

pub struct SkeletonPipeline {
    bone_count: u32,
    bone_transform_bind_group: wgpu::BindGroup,
    camera_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline
}

impl SkeletonPipeline {
    pub fn new(
        device: &wgpu::Device,
        camera_bg_layout: &wgpu::BindGroupLayout,
        camera_bg: wgpu::BindGroup,
        bone_bg_layout: &wgpu::BindGroupLayout,
        bone_bg: wgpu::BindGroup,
        bone_count: u32
    ) -> Self {
        let module =
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/skeleton.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skeleton visual pipeline layout"),
            bind_group_layouts: &[Some(camera_bg_layout), Some(bone_bg_layout)],
            immediate_size: 0
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("skeleton visual pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 3 * size_of::<f32>() as u64,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0
                    }],
                    step_mode: wgpu::VertexStepMode::Vertex
                })]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            // To run this pipeline within the same render pass as the other drawing commands, we need to include a compatible depth stencil.
            // We will not use this stencil and will not write to it either.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: None,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TARGET_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            multiview_mask: None,
            cache: None
        });

        Self {
            pipeline,
            camera_bind_group: camera_bg,
            bone_transform_bind_group: bone_bg,
            bone_count
        }
    }

    pub fn update_skeleton(&mut self) {}

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.bone_transform_bind_group, &[]);
        render_pass.draw(0..4, 0..self.bone_count);
    }
}
