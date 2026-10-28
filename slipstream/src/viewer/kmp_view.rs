use std::ops::ControlFlow;
use std::sync::Arc;

use slipstream_ir::kmp::{Checkpoint, CheckpointBox2, Checkpoints, KmpHeader, KmpVisitor};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::{Visitor, VisitorCtx};
use slipstream_shared::SlipstreamResult;
use wgpu::util::DeviceExt;

use crate::viewer::pipeline::{DEPTH_FORMAT, MSAA_SAMPLE_COUNT, TARGET_FORMAT, ViewerPipeline};

#[derive(Default)]
struct KmpRootVisitor {
    node: Option<IrNodeKey>
}

impl Visitor for KmpRootVisitor {
    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }
}

impl KmpVisitor for KmpRootVisitor {
    fn visit_kmp(&mut self, ctx: VisitorCtx<'_, KmpHeader>) -> ControlFlow<()> {
        if ctx.meta.label == "course.kmp" {
            tracing::debug!("Found `course.kmp` at node {:?}", ctx.meta.key);
            self.node = Some(ctx.meta.key);

            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

pub struct KmpPipeline {
    pub course_kmp: IrNodeKey,
    pub arena: Arc<IrArena>,
    checkpoints: Option<CheckpointsPipeline>
}

impl KmpPipeline {
    pub fn from_root(
        device: &wgpu::Device,
        camera_bg_layout: &wgpu::BindGroupLayout,
        camera_bg: wgpu::BindGroup,
        arena: Arc<IrArena>
    ) -> SlipstreamResult<Option<Self>> {
        // Check if a file is actually open before trying to find the node
        if arena.is_empty() {
            return Ok(None);
        }

        // Find the `course.kmp` file.
        let mut vis = KmpRootVisitor::default();
        arena.visit_children(IrNodeKey::ROOT, &mut vis)?;

        let Some(root) = vis.node else {
            return Ok(None);
        };

        // course.kmp was found, we now start building buffers for rendering
        let mut buffer_vis = KmpDrawVisitor {
            device,
            camera_bg_layout,
            camera_bg,
            checkpoints_buffer: None
        };
        arena.visit_all(root, &mut buffer_vis)?;

        tracing::trace!("Generated KMP buffers");

        Ok(Some(Self {
            course_kmp: root,
            arena,
            checkpoints: buffer_vis.checkpoints_buffer
        }))
    }
}

impl ViewerPipeline for KmpPipeline {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        if let Some(checkpoints) = &self.checkpoints {
            checkpoints.draw(render_pass);
        }
    }
}

struct CheckpointsPipeline {
    checkpoint_count: usize,
    /// Vertex buffer contains only two vec3 corners, the box itself is generated in the vertex
    /// shader.
    vertex_buffer: wgpu::Buffer,
    camera_bg: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline
}

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
    pub ty: CheckpointType
}

impl CheckpointsPipeline {
    pub const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<glam::Vec2>() as u64 * 2 + size_of::<u32>() as u64,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: size_of::<glam::Vec2>() as u64,
                shader_location: 1
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: size_of::<glam::Vec2>() as u64 * 2,
                shader_location: 2
            }
        ],
        step_mode: wgpu::VertexStepMode::Instance
    };

    pub fn new(
        device: &wgpu::Device,
        camera_bg_layout: &wgpu::BindGroupLayout,
        camera_bg: wgpu::BindGroup,
        checkpoints: &[Checkpoint]
    ) -> Self {
        let checkpoint_count = checkpoints.len();

        // 6 floats per checkpoint
        let mut bounds = Vec::with_capacity(checkpoint_count);
        for checkpoint in checkpoints {
            bounds.push(CheckpointData {
                bounds: checkpoint.bounds,
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
            device.create_shader_module(wgpu::include_wgsl!("../../shaders/checkpoint.wgsl"));

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

struct KmpDrawVisitor<'a> {
    device: &'a wgpu::Device,

    camera_bg_layout: &'a wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,

    checkpoints_buffer: Option<CheckpointsPipeline>
}

impl KmpVisitor for KmpDrawVisitor<'_> {
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_checkpoints(&mut self, context: VisitorCtx<'_, Checkpoints>) -> ControlFlow<()> {
        self.checkpoints_buffer = Some(CheckpointsPipeline::new(
            self.device,
            self.camera_bg_layout,
            self.camera_bg.clone(),
            &context.checkpoints
        ));
        ControlFlow::Continue(())
    }
}

impl Visitor for KmpDrawVisitor<'_> {
    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }
}
