use std::ops::ControlFlow;
use std::sync::Arc;

use slipstream_ir::kmp::{Checkpoint, CheckpointBox2, Checkpoints, EnemyRoutePoint, KmpHeader, KmpVisitor, RespawnPoint, Respawns};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::{Visitor, VisitorCtx};
use slipstream_shared::SlipstreamResult;
use wgpu::util::DeviceExt;

use crate::viewer::kmp::KmpRootVisitor;
use crate::viewer::pipeline::{DEPTH_FORMAT, MSAA_SAMPLE_COUNT, TARGET_FORMAT, ViewerPipeline};

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct PointData {
    position: glam::Vec3
}

pub(super) struct EnemyRoutesPipeline {
    point_count: usize,
    vertex_buffer: wgpu::Buffer,
    camera_bg: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline
}

impl EnemyRoutesPipeline {
    pub const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: 3 * 4,
        attributes: &wgpu::vertex_attr_array! [
            0 => Float32x3,
        ],
        step_mode: wgpu::VertexStepMode::Instance
    };

    pub fn new(
        device: &wgpu::Device,
        camera_bg_layout: &wgpu::BindGroupLayout,
        camera_bg: wgpu::BindGroup,
        route_points: &[EnemyRoutePoint],
    ) -> Self {
        let point_count = route_points.len();

        // 6 floats per checkpoint
        let mut bounds = Vec::with_capacity(point_count);
        for (i, point) in route_points.iter().enumerate() {
            bounds.push(PointData { position: point.position });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("enemy route point buffer"),
            contents: bytemuck::cast_slice(&bounds),
            usage: wgpu::BufferUsages::VERTEX
        });

        tracing::trace!(
            "Generated vertex buffer with {} enemy route points",
            point_count
        );

        let module =
            device.create_shader_module(wgpu::include_wgsl!("../../../shaders/enemy_route_points.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("enemy route points pipeline layout"),
            bind_group_layouts: &[Some(camera_bg_layout)],
            immediate_size: 0
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("enemy route points render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Self::VERTEX_LAYOUT)]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
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
            point_count,
            vertex_buffer,
            camera_bg,
            pipeline
        }
    }
}

impl ViewerPipeline for EnemyRoutesPipeline {
    fn draw(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.camera_bg, &[]);
        pass.draw(0..3, 0..self.point_count as u32);
    }
}