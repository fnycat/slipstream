use std::ops::ControlFlow;

use crate::brres::{
    Bone, ColorBuffer, Definitions, MaterialBuffer, MdlHeader, NormalBuffer, PaletteLinks, Polygon,
    Tev, TextureLinks, UvBuffer, VertexBuffer
};
use crate::visitor::{VisitorCtx, VisitorCtxMut};

#[allow(unused_variables)]
pub trait MdlVisitor {
    /// Whether to stop traversing the MDL file when an unimplemented visitor method is encountered.
    ///
    /// By default this is disabled.
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_mdl(&mut self, model: VisitorCtx<'_, MdlHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_definitions(&mut self, definitions: VisitorCtx<'_, Definitions>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_bone(&mut self, bone: VisitorCtx<'_, Bone>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_vertices(&mut self, vertex_buf: VisitorCtx<'_, VertexBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_normals(&mut self, normal_buf: VisitorCtx<'_, NormalBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_colors(&mut self, color_buf: VisitorCtx<'_, ColorBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_uvs(&mut self, uv_buf: VisitorCtx<'_, UvBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_polygon(&mut self, polygon: VisitorCtx<'_, Polygon>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_material(&mut self, material: VisitorCtx<'_, MaterialBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tev(&mut self, tev: VisitorCtx<'_, Tev>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_palette_links(&mut self, links: VisitorCtx<'_, PaletteLinks>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_texture_links(&mut self, links: VisitorCtx<'_, TextureLinks>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    // Mutable MDL visitor methods
    // ==================================================================================================

    fn visit_mdl_mut(&mut self, model: VisitorCtxMut<'_, MdlHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_definitions_mut(
        &mut self,
        definitions: VisitorCtxMut<'_, Definitions>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_bone_mut(&mut self, bone: VisitorCtxMut<'_, Bone>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_vertices_mut(
        &mut self,
        vertex_buf: VisitorCtxMut<'_, VertexBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_normals_mut(
        &mut self,
        normal_buf: VisitorCtxMut<'_, NormalBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_colors_mut(&mut self, color_buf: VisitorCtxMut<'_, ColorBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_uvs_mut(&mut self, uv_buf: VisitorCtxMut<'_, UvBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_polygon_mut(&mut self, polygon: VisitorCtxMut<'_, Polygon>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_material_mut(
        &mut self,
        material: VisitorCtxMut<'_, MaterialBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tev_mut(&mut self, tev: VisitorCtxMut<'_, Tev>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_palette_links_mut(
        &mut self,
        links: VisitorCtxMut<'_, PaletteLinks>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_texture_links_mut(
        &mut self,
        links: VisitorCtxMut<'_, TextureLinks>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
