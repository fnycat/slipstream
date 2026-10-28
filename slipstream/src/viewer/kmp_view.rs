use std::ops::ControlFlow;
use std::sync::Arc;

use slipstream_ir::kmp::{KmpHeader, KmpVisitor};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::{Visitor, VisitorCtx};

use crate::viewer::pipeline::ViewerPipeline;

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
        }

        ControlFlow::Break(())
    }
}

pub struct KmpPipeline {
    pub course_kmp: IrNodeKey,
    pub arena: Arc<IrArena>
}

impl KmpPipeline {
    pub fn new(course_kmp: IrNodeKey, arena: Arc<IrArena>) -> Self {
        Self { course_kmp, arena }
    }

    pub fn from_root(arena: Arc<IrArena>) -> Option<Self> {
        // Find the `course.kmp` file.
        let mut vis = KmpRootVisitor::default();
        arena.visit_children(IrNodeKey::ROOT, &mut vis);

        let Some(root) = vis.node else { return None };

        Some(Self {
            course_kmp: root,
            arena
        })
    }
}

impl ViewerPipeline for KmpPipeline {
    fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        todo!()
    }
}
