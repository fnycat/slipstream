use std::sync::Arc;

use slipstream_ir::node::arena::{IrArena, IrNodeKey};

pub struct KmpPipeline {
    pub course_kmp: IrNodeKey,
    pub arena: Arc<IrArena>
}

impl KmpPipeline {
    pub fn new(course_kmp: IrNodeKey, arena: Arc<IrArena>) -> Self {
        Self { course_kmp, arena }
    }

    pub fn draw(&self, render_pass: &mut wgpu::RenderPass) {
        todo!()
    }
}
