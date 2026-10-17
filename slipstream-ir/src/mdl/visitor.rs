use std::ops::ControlFlow;

use crate::mdl;
use crate::visitor::{VisitorCtx, VisitorCtxMut};

#[allow(unused_variables)]
pub trait Mdl0Visitor {
    /// Whether to stop traversing the MDL file when an unimplemented visitor method is encountered.
    ///
    /// By default this is disabled.
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_mdl(&mut self, model: VisitorCtx<'_, mdl::Mdl0Root>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_definitions(
        &mut self,
        definitions: VisitorCtx<'_, mdl::Definitions>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_bone(&mut self, bone: VisitorCtx<'_, mdl::Bone>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_vertices(&mut self, vertex_buf: VisitorCtx<'_, mdl::VertexBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_normals(&mut self, normal_buf: VisitorCtx<'_, mdl::NormalBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_colors(&mut self, color_buf: VisitorCtx<'_, mdl::ColorBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_uvs(&mut self, uv_buf: VisitorCtx<'_, mdl::UvBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_polygon(&mut self, polygon: VisitorCtx<'_, mdl::Polygon>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_material(&mut self, material: VisitorCtx<'_, mdl::MaterialBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tev(&mut self, tev: VisitorCtx<'_, mdl::Tev>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_palette_links(&mut self, links: VisitorCtx<'_, mdl::PaletteLinks>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_texture_links(&mut self, links: VisitorCtx<'_, mdl::TextureLinks>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    // Mutable MDL visitor methods
    // ==================================================================================================

    fn visit_mdl_mut(&mut self, model: VisitorCtxMut<'_, mdl::Mdl0Root>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_definitions_mut(
        &mut self,
        definitions: VisitorCtxMut<'_, mdl::Definitions>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_bone_mut(&mut self, bone: VisitorCtxMut<'_, mdl::Bone>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_vertices_mut(
        &mut self,
        vertex_buf: VisitorCtxMut<'_, mdl::VertexBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_normals_mut(
        &mut self,
        normal_buf: VisitorCtxMut<'_, mdl::NormalBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_colors_mut(
        &mut self,
        color_buf: VisitorCtxMut<'_, mdl::ColorBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_uvs_mut(&mut self, uv_buf: VisitorCtxMut<'_, mdl::UvBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_polygon_mut(&mut self, polygon: VisitorCtxMut<'_, mdl::Polygon>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_material_mut(
        &mut self,
        material: VisitorCtxMut<'_, mdl::MaterialBuffer>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tev_mut(&mut self, tev: VisitorCtxMut<'_, mdl::Tev>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_palette_links_mut(
        &mut self,
        links: VisitorCtxMut<'_, mdl::PaletteLinks>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_texture_links_mut(
        &mut self,
        links: VisitorCtxMut<'_, mdl::TextureLinks>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
