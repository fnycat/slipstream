use std::ops::ControlFlow;

use crate::brres::{
    Bone, ColorBuffer, Definitions, Material, MdlHeader, NormalBuffer, PaletteLinks, Polygon, Tev,
    TextureLinks, UvBuffer, VertexBuffer
};
use crate::kmp::KmpHeader;
use crate::visitor::{VisitorCtx, VisitorCtxMut};

#[allow(unused_variables)]
pub trait KmpVisitor {
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_kmp(&mut self, context: VisitorCtx<'_, KmpHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_kmp_mut(&mut self, context: VisitorCtxMut<'_, KmpHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
