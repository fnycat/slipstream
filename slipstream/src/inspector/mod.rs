use crate::panes::{ContentSignature, Pane, PaneAction, PaneId};
use slipstream_ir::arc::ArcDirectory;
use slipstream_ir::brres::BrresFile;
use slipstream_ir::chr::{self, Chr0Visitor, SkeletalAnimation};
use slipstream_ir::mdl::{
    Bone, ColorBuffer, Definitions, MaterialBuffer, Mdl0Root, Mdl0Visitor, NormalBuffer,
    PaletteLinks, Polygon, TextureLinks, UvBuffer, VertexBuffer
};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::tex::Texture;
use slipstream_ir::tpl::{TplHeader, TplImage};
use slipstream_ir::visitor::{Visitor, VisitorCtxMetaMut, VisitorCtxMut};
use slipstream_shared::inspect::{FieldConfig, Inspect};
use slipstream_shared::{SlipstreamError, SlipstreamResult};
use std::ops::ControlFlow;
use std::sync::{Arc, mpsc};

struct InspectorVisitor<'ui> {
    pub ui: &'ui mut egui::Ui
}

impl Visitor for InspectorVisitor<'_> {
    fn as_mdl(&mut self) -> Option<&mut dyn Mdl0Visitor> {
        Some(self)
    }

    fn as_chr(&mut self) -> Option<&mut dyn Chr0Visitor> {
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
}

impl Mdl0Visitor for InspectorVisitor<'_> {
    fn visit_mdl_mut(&mut self, context: VisitorCtxMut<'_, Mdl0Root>) -> ControlFlow<()> {
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

    fn visit_material_mut(
        &mut self,
        context: VisitorCtxMut<'_, MaterialBuffer>
    ) -> ControlFlow<()> {
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

impl Chr0Visitor for InspectorVisitor<'_> {
    fn visit_chr_mut(&mut self, context: VisitorCtxMut<'_, chr::Chr0Root>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_animation_mut(
        &mut self,
        context: VisitorCtxMut<'_, SkeletalAnimation>
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }
}

pub struct InspectorPane {
    cmd_sender: mpsc::Sender<PaneAction>,

    inspected: IrNodeKey,
    arena: Arc<IrArena>
}

impl InspectorPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        inspected: IrNodeKey,
        arena: Arc<IrArena>
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            inspected,
            arena
        })
    }

    fn draw_properties(&mut self, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        let avail_size = ui.available_size();
        ui.set_min_size(avail_size);

        ui.spacing_mut().item_spacing.y = 7.5;

        self.arena
            .update(self.inspected, |node| {
                let context = VisitorCtxMetaMut {
                    key: node.key(),
                    label: &mut node.label,
                    ty: node.ty,
                    children: &mut node.children
                };

                ui.spacing_mut().item_spacing = egui::vec2(2.0, 4.0);

                // Evaluate contents if lazy
                let contents = node.contents.get_or_try_init_mut()?.unwrap();
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    let mut visitor = InspectorVisitor { ui };
                    let _ = contents.accept_mut(context, &mut visitor); // ignore the control flow as we're not continuing anyways.
                });

                Ok::<_, SlipstreamError>(())
            })
            .transpose()?;

        Ok(())
    }
}

impl Pane for InspectorPane {
    fn ty(&self) -> PaneId {
        PaneId::Inspector
    }

    fn title(&self) -> &str {
        "Properties"
    }

    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        _is_focused: bool
    ) -> egui_tiles::UiResponse {
        self.draw_properties(ui).unwrap();

        egui_tiles::UiResponse::None
    }
}
