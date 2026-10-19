use std::ops::ControlFlow;

use slipstream_ir::arc::ArcDirectory;
use slipstream_ir::brres::{
    Bone, BrresFile, ChrHeader, ClrHeader, ColorAnimation, ColorBuffer, Definitions, Material,
    MdlHeader, MdlVisitor, NormalBuffer, PaletteLinks, Polygon, SkeletalAnimation, Texture,
    TextureLinks, UvBuffer, VertexBuffer
};
use slipstream_ir::kmp::{
    EnemyRoutePoints, EnemyRoutes, ItemRoutePoints, KmpHeader, KmpVisitor, StartingPositions
};
use slipstream_ir::tpl::{TplHeader, TplImage};
use slipstream_ir::visitor::{Visitor, VisitorCtxMut};
use slipstream_shared::inspect::{FieldConfig, Inspect};

pub struct InspectorVisitor<'ui> {
    pub ui: &'ui mut egui::Ui
}

impl Visitor for InspectorVisitor<'_> {
    fn as_mdl(&mut self) -> Option<&mut dyn MdlVisitor> {
        Some(self)
    }

    fn as_kmp(&mut self) -> Option<&mut dyn KmpVisitor> {
        Some(self)
    }

    fn visit_arc_mut(&mut self, context: VisitorCtxMut<'_, ArcDirectory>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_brres_mut(&mut self, context: VisitorCtxMut<'_, BrresFile>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_chr_mut(&mut self, context: VisitorCtxMut<'_, ChrHeader>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_skeletal_animation_mut(
        &mut self,
        context: VisitorCtxMut<'_, SkeletalAnimation>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_tex_mut(&mut self, context: VisitorCtxMut<'_, Texture>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_tpl_mut(&mut self, context: VisitorCtxMut<'_, TplHeader>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_tpl_image_mut(&mut self, context: VisitorCtxMut<'_, TplImage>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_clr_mut(&mut self, context: VisitorCtxMut<'_, ClrHeader>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_color_animation_mut(
        &mut self,
        context: VisitorCtxMut<'_, ColorAnimation>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }
}

impl MdlVisitor for InspectorVisitor<'_> {
    fn visit_mdl_mut(&mut self, context: VisitorCtxMut<'_, MdlHeader>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_definitions_mut(
        &mut self,
        context: VisitorCtxMut<'_, Definitions>
    ) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_bone_mut(&mut self, context: VisitorCtxMut<'_, Bone>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_vertices_mut(&mut self, context: VisitorCtxMut<'_, VertexBuffer>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_normals_mut(&mut self, context: VisitorCtxMut<'_, NormalBuffer>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_colors_mut(&mut self, context: VisitorCtxMut<'_, ColorBuffer>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_uvs_mut(&mut self, context: VisitorCtxMut<'_, UvBuffer>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_material_mut(&mut self, context: VisitorCtxMut<'_, Material>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_polygon_mut(&mut self, context: VisitorCtxMut<'_, Polygon>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_texture_links_mut(
        &mut self,
        context: VisitorCtxMut<'_, TextureLinks>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_palette_links_mut(
        &mut self,
        context: VisitorCtxMut<'_, PaletteLinks>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }
}

impl KmpVisitor for InspectorVisitor<'_> {
    fn visit_kmp_mut(&mut self, context: VisitorCtxMut<'_, KmpHeader>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_starting_positions_mut(
        &mut self,
        context: VisitorCtxMut<'_, StartingPositions>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_enemy_route_mut(
        &mut self,
        context: VisitorCtxMut<'_, EnemyRoutes>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_enemy_route_points_mut(
        &mut self,
        context: VisitorCtxMut<'_, EnemyRoutePoints>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_item_route_points_mut(
        &mut self,
        context: VisitorCtxMut<'_, ItemRoutePoints>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }
}
