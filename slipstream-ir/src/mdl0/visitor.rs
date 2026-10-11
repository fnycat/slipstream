use std::ops::ControlFlow;

use crate::{
    mdl0,
    visitor::{VisitorContext, VisitorContextMut},
};

#[allow(unused_variables)]
pub trait Mdl0Visitor {
    /// Whether to stop traversing the MDL0 file when an unimplemented visitor method is encountered.
    ///
    /// By default this is disabled.
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_mdl0(&mut self, model: VisitorContext<'_, mdl0::Mdl0Root>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_definitions(
        &mut self,
        definitions: VisitorContext<'_, mdl0::Definitions>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_bone(&mut self, bone: VisitorContext<'_, mdl0::Bone>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_vertices(
        &mut self,
        vertex_buf: VisitorContext<'_, mdl0::VertexBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_normals(
        &mut self,
        normal_buf: VisitorContext<'_, mdl0::NormalBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_colors(
        &mut self,
        color_buf: VisitorContext<'_, mdl0::ColorBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_uvs(&mut self, uv_buf: VisitorContext<'_, mdl0::UvBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_polygon(&mut self, polygon: VisitorContext<'_, mdl0::Polygon>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_material(
        &mut self,
        material: VisitorContext<'_, mdl0::MaterialBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tev(&mut self, tev: VisitorContext<'_, mdl0::Tev>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_palette_links(
        &mut self,
        links: VisitorContext<'_, mdl0::PaletteLinks>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_texture_links(
        &mut self,
        links: VisitorContext<'_, mdl0::TextureLinks>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    // Mutable MDL0 visitor methods
    // ==================================================================================================

    fn visit_mdl0_mut(&mut self, model: VisitorContextMut<'_, mdl0::Mdl0Root>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_definitions_mut(
        &mut self,
        definitions: VisitorContextMut<'_, mdl0::Definitions>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_bone_mut(&mut self, bone: VisitorContextMut<'_, mdl0::Bone>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_vertices_mut(
        &mut self,
        vertex_buf: VisitorContextMut<'_, mdl0::VertexBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_normals_mut(
        &mut self,
        normal_buf: VisitorContextMut<'_, mdl0::NormalBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_colors_mut(
        &mut self,
        color_buf: VisitorContextMut<'_, mdl0::ColorBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_uvs_mut(&mut self, uv_buf: VisitorContextMut<'_, mdl0::UvBuffer>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_polygon_mut(
        &mut self,
        polygon: VisitorContextMut<'_, mdl0::Polygon>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_material_mut(
        &mut self,
        material: VisitorContextMut<'_, mdl0::MaterialBuffer>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tev_mut(&mut self, tev: VisitorContextMut<'_, mdl0::Tev>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_palette_links_mut(
        &mut self,
        links: VisitorContextMut<'_, mdl0::PaletteLinks>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_texture_links_mut(
        &mut self,
        links: VisitorContextMut<'_, mdl0::TextureLinks>,
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
