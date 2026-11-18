mod checkpoints;
mod enemy_routes;

use std::ops::ControlFlow;
use std::sync::Arc;

pub use checkpoints::*;
use slipstream_ir::kmp::{
    Checkpoints, EnemyRoutePoint, EnemyRoutePoints, KmpHeader, KmpVisitor, RespawnPoint, Respawns,
};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::{Visitor, VisitorCtx};
use slipstream_shared::SlipstreamResult;

use crate::viewer::kmp::enemy_routes::EnemyRoutesPipeline;
use crate::viewer::pipeline::ViewerPipeline;

#[derive(Default)]
pub struct CourseKmpLocator {
    node: Option<IrNodeKey>,
}

impl CourseKmpLocator {
    pub fn into_inner(self) -> Option<IrNodeKey> {
        self.node
    }
}

impl Visitor for CourseKmpLocator {
    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }
}

impl KmpVisitor for CourseKmpLocator {
    fn visit_kmp(&mut self, ctx: VisitorCtx<'_, KmpHeader>) -> ControlFlow<()> {
        if ctx.meta.label == "course.kmp" {
            tracing::debug!("Found `course.kmp` at node {:?}", ctx.meta.key);
            self.node = Some(ctx.meta.key);
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }
}

/// Visitor that searches for a `course.kmp` file and loads some prequisites that are needed for the pipelines later.
#[derive(Default)]
pub struct CourseKmpPreReqVisitor {
    // Required for checkpoint rendering. Respawn points are used to determine the approximate Y-location of a checkpoint.
    respawns: Vec<RespawnPoint>,
}

impl Visitor for CourseKmpPreReqVisitor {
    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }
}

impl KmpVisitor for CourseKmpPreReqVisitor {
    fn visit_respawns(&mut self, context: VisitorCtx<'_, Respawns>) -> ControlFlow<()> {
        self.respawns.clone_from(&context.content.respawns);
        ControlFlow::Continue(())
    }
}

pub struct KmpPipeline {
    pub course_kmp: IrNodeKey,
    pub arena: Arc<IrArena>,

    checkpoints: Option<CheckpointsPipeline>,
    enemy_routes: Option<EnemyRoutesPipeline>,

    /// Should the checkpoints be drawn?
    ///
    /// This has no effect if there is no checkpoint pipeline.
    draw_checkpoints: bool,
    /// Should the enemy routes be drawn?
    ///
    /// This has no effect if there is no enemy route pipeline.
    draw_enemy_routes: bool,
}

impl KmpPipeline {
    pub fn from_root(
        course_kmp: IrNodeKey,
        device: &wgpu::Device,
        camera_bg_layout: &wgpu::BindGroupLayout,
        camera_bg: wgpu::BindGroup,
        arena: Arc<IrArena>,
    ) -> SlipstreamResult<Self> {
        // // Check if a file is actually open before trying to find the node
        // if arena.is_empty() {
        //     return Ok(None);
        // }

        // Find the `course.kmp` file.
        let mut vis = CourseKmpPreReqVisitor::default();
        arena.visit_children(IrNodeKey::ROOT, &mut vis)?;

        // course.kmp was found, we now start building buffers for rendering
        let mut pipeline_vis = KmpPipelineVisitor {
            device,

            sections: vis,

            camera_bg_layout,
            camera_bg,
            checkpoint_pipeline: None,
            enemy_routes_pipeline: None,
        };
        arena.visit_all(course_kmp, &mut pipeline_vis)?;

        tracing::trace!("Generated KMP buffers");

        Ok(Self {
            course_kmp,
            arena,
            checkpoints: pipeline_vis.checkpoint_pipeline,
            enemy_routes: pipeline_vis.enemy_routes_pipeline,

            draw_checkpoints: false,
            draw_enemy_routes: false,
        })
    }
}

impl ViewerPipeline for KmpPipeline {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        if let Some(pipeline) = &self.checkpoints
            && self.draw_checkpoints
        {
            pipeline.draw(render_pass);
        }

        if let Some(pipeline) = &self.enemy_routes
            && self.draw_enemy_routes
        {
            pipeline.draw(render_pass);
        }
    }
}

pub struct KmpPipelineVisitor<'a> {
    device: &'a wgpu::Device,

    sections: CourseKmpPreReqVisitor,

    camera_bg_layout: &'a wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,

    checkpoint_pipeline: Option<CheckpointsPipeline>,
    enemy_routes_pipeline: Option<EnemyRoutesPipeline>,
}

impl KmpVisitor for KmpPipelineVisitor<'_> {
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_checkpoints(&mut self, context: VisitorCtx<'_, Checkpoints>) -> ControlFlow<()> {
        if !context.checkpoints.is_empty() {
            self.checkpoint_pipeline = Some(CheckpointsPipeline::new(
                self.device,
                self.camera_bg_layout,
                self.camera_bg.clone(),
                &context.checkpoints,
                &self.sections.respawns,
            ));
        }
        ControlFlow::Continue(())
    }

    fn visit_enemy_route_points(
        &mut self,
        context: VisitorCtx<'_, EnemyRoutePoints>,
    ) -> ControlFlow<()> {
        if !context.points.is_empty() {
            self.enemy_routes_pipeline = Some(EnemyRoutesPipeline::new(
                self.device,
                self.camera_bg_layout,
                self.camera_bg.clone(),
                &context.points,
            ));
        }
        ControlFlow::Continue(())
    }
}

impl Visitor for KmpPipelineVisitor<'_> {
    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }
}
