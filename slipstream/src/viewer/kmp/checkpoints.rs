use std::ops::ControlFlow;
use std::sync::Arc;

use slipstream_ir::kmp::{
    Checkpoint, CheckpointBox2, Checkpoints, KmpHeader, KmpVisitor, RespawnPoint, Respawns
};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::{Visitor, VisitorCtx};
use slipstream_shared::SlipstreamResult;
use wgpu::util::DeviceExt;

use crate::viewer::kmp::KmpRootVisitor;
use crate::viewer::pipeline::{DEPTH_FORMAT, MSAA_SAMPLE_COUNT, TARGET_FORMAT, ViewerPipeline};

#[derive(Debug, Copy, Clone, PartialEq, Eq, bytemuck::Zeroable, bytemuck::NoUninit)]
#[repr(u32)]
enum CheckpointType {
    Standard = 0x00,
    Key = 0x01,
    Lap = 0x02
}

impl From<u8> for CheckpointType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Lap,
            0xFF => Self::Standard,
            _ => Self::Key
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Zeroable, bytemuck::NoUninit)]
#[repr(C)]
struct CheckpointData {
    pub bounds: CheckpointBox2,
    pub respawn_y: f32,
    pub ty: CheckpointType
}

pub(super) struct CheckpointsPipeline {
    checkpoint_count: usize,
    /// Vertex buffer contains only two vec3 corners, the box itself is generated in the vertex
    /// shader.
    vertex_buffer: wgpu::Buffer,
    camera_bg: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline
}

impl CheckpointsPipeline {
    pub const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: 6 * 4,
        attributes: &wgpu::vertex_attr_array! [
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32,
            3 => Uint32
        ],
        step_mode: wgpu::VertexStepMode::Instance
    };

    pub fn new(
        device: &wgpu::Device,
        camera_bg_layout: &wgpu::BindGroupLayout,
        camera_bg: wgpu::BindGroup,
        checkpoints: &[Checkpoint],
        respawns: &[RespawnPoint]
    ) -> Self {
        let checkpoint_count = checkpoints.len();

        // 6 floats per checkpoint
        let mut bounds = Vec::with_capacity(checkpoint_count);
        for (i, checkpoint) in checkpoints.iter().enumerate() {
            let respawn_y = match respawns.get(checkpoint.respawn_index as usize) {
                Some(respawn) => respawn.position.y,
                None => {
                    tracing::error!(
                        "Respawn point index out of range for checkpoint {i} ({} > {})",
                        checkpoint.respawn_index,
                        respawns.len()
                    );
                    0.0
                }
            };

            bounds.push(CheckpointData {
                bounds: checkpoint.bounds,
                respawn_y,
                ty: CheckpointType::from(checkpoint.ty as u8)
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("checkpoint bounding box buffer"),
            contents: bytemuck::cast_slice(&bounds),
            usage: wgpu::BufferUsages::VERTEX
        });
        tracing::trace!(
            "Generated vertex buffer with {} checkpoint bounds",
            checkpoint_count
        );

        let module =
            device.create_shader_module(wgpu::include_wgsl!("../../../shaders/checkpoint.wgsl"));

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("checkpoints pipeline layout"),
            bind_group_layouts: &[Some(camera_bg_layout)],
            immediate_size: 0
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("checkpoints render pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(Self::VERTEX_LAYOUT)]
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
                    blend: Some(wgpu::BlendState::ADDITIVE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            multiview_mask: None,
            cache: None
        });

        Self {
            checkpoint_count,
            vertex_buffer,
            camera_bg,
            pipeline
        }
    }
}

impl ViewerPipeline for CheckpointsPipeline {
    fn draw(&self, pass: &mut wgpu::RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.camera_bg, &[]);
        pass.draw(0..4, 0..self.checkpoint_count as u32);
    }
}
