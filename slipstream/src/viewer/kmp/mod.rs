mod checkpoints;
mod enemy_routes;

use std::{ops::ControlFlow, sync::Arc};

pub use checkpoints::*;
use slipstream_ir::{kmp::{Checkpoints, EnemyRoutePoint, EnemyRoutePoints, KmpHeader, KmpVisitor, RespawnPoint, Respawns}, node::arena::{IrArena, IrNodeKey}, visitor::{Visitor, VisitorCtx}};
use slipstream_shared::SlipstreamResult;

use crate::viewer::{kmp::enemy_routes::EnemyRoutesPipeline, pipeline::ViewerPipeline};


#[derive(Default)]
struct KmpRootVisitor {
    node: Option<IrNodeKey>,
    respawns: Vec<RespawnPoint>
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
        }
        ControlFlow::Continue(())
    }

    fn visit_respawns(&mut self, context: VisitorCtx<'_, Respawns>) -> ControlFlow<()> {
        self.respawns = context.content.respawns.clone();
        ControlFlow::Continue(())
    }
}

pub struct KmpPipeline {
    pub course_kmp: IrNodeKey,
    pub arena: Arc<IrArena>,

    checkpoints: Option<CheckpointsPipeline>,
    enemy_routes: Option<EnemyRoutesPipeline>
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
        let mut pipeline_vis = KmpPipelineVisitor {
            device,

            sections: vis,

            camera_bg_layout,
            camera_bg,
            checkpoint_pipeline: None,
            enemy_routes_pipeline: None
        };
        arena.visit_all(root, &mut pipeline_vis)?;

        tracing::trace!("Generated KMP buffers");

        Ok(Some(Self {
            course_kmp: root,
            arena,
            checkpoints: pipeline_vis.checkpoint_pipeline,
            enemy_routes: pipeline_vis.enemy_routes_pipeline
        }))
    }
}

impl ViewerPipeline for KmpPipeline {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        if let Some(pipeline) = &self.checkpoints {
            pipeline.draw(render_pass);
        }

        if let Some(pipeline) = &self.enemy_routes {
            pipeline.draw(render_pass);
        }
    }
}

pub struct KmpPipelineVisitor<'a> {
    device: &'a wgpu::Device,

    sections: KmpRootVisitor,

    camera_bg_layout: &'a wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,

    checkpoint_pipeline: Option<CheckpointsPipeline>,
    enemy_routes_pipeline: Option<EnemyRoutesPipeline>
}

impl KmpVisitor for KmpPipelineVisitor<'_> {
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_checkpoints(&mut self, context: VisitorCtx<'_, Checkpoints>) -> ControlFlow<()> {
        self.checkpoint_pipeline = Some(CheckpointsPipeline::new(
            self.device,
            self.camera_bg_layout,
            self.camera_bg.clone(),
            &context.checkpoints,
            &self.sections.respawns
        ));
        ControlFlow::Continue(())
    }

    fn visit_enemy_route_points(
        &mut self,
        context: VisitorCtx<'_, EnemyRoutePoints>
    ) -> ControlFlow<()>
    {
        self.enemy_routes_pipeline = Some(EnemyRoutesPipeline::new(
            self.device,
            self.camera_bg_layout,
            self.camera_bg.clone(),
            &context.points,
        ));
        ControlFlow::Continue(())
    }
}

impl Visitor for KmpPipelineVisitor<'_> {
    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }
}
