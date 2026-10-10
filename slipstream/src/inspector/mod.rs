use crate::panes::{ContentSignature, Pane, PaneAction, PaneId};
use slipstream_ir::chr0::{self, SkeletalAnimation};
use slipstream_ir::mdl0::{
    Bone, ColorBuffer, Definitions, MaterialBuffer, Mdl0Root, NormalBuffer, PaletteLinks, Polygon,
    TextureLinks, UvBuffer, VertexBuffer,
};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::tex0::Texture;
use slipstream_ir::visitor::{Visitor, VisitorContextMut, VisitorContextNodeMut};
use slipstream_shared::inspect::{FieldConfig, Inspect};
use slipstream_shared::{SlipstreamError, SlipstreamResult};
use std::ops::ControlFlow;
use std::sync::{Arc, mpsc};

struct InspectorVisitor<'ui> {
    pub ui: &'ui mut egui::Ui,
}

impl Visitor for InspectorVisitor<'_> {
    fn visit_mdl0_mut(&mut self, context: VisitorContextMut<'_, Mdl0Root>) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_definitions_mut(
        &mut self,
        context: VisitorContextMut<'_, Definitions>,
    ) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_bone_mut(&mut self, context: VisitorContextMut<'_, Bone>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_vertices_mut(
        &mut self,
        context: VisitorContextMut<'_, VertexBuffer>,
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_normals_mut(
        &mut self,
        context: VisitorContextMut<'_, NormalBuffer>,
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_colors_mut(&mut self, context: VisitorContextMut<'_, ColorBuffer>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_uvs_mut(&mut self, context: VisitorContextMut<'_, UvBuffer>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_material_mut(
        &mut self,
        context: VisitorContextMut<'_, MaterialBuffer>,
    ) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_polygon_mut(&mut self, context: VisitorContextMut<'_, Polygon>) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_texture_links_mut(
        &mut self,
        context: VisitorContextMut<'_, TextureLinks>,
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_palette_links_mut(
        &mut self,
        context: VisitorContextMut<'_, PaletteLinks>,
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_tex0_mut(&mut self, context: VisitorContextMut<'_, Texture>) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_chr0_mut(
        &mut self,
        context: VisitorContextMut<'_, chr0::Chr0Root>,
    ) -> ControlFlow<()> {
        let _changes = context
            .content
            .draw_properties(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_skeletal_animation_mut(
        &mut self,
        context: VisitorContextMut<'_, SkeletalAnimation>,
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
    arena: Arc<IrArena>,
}

impl InspectorPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        inspected: IrNodeKey,
        arena: Arc<IrArena>,
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            inspected,
            arena,
        })
    }

    fn draw_properties(&mut self, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        let avail_size = ui.available_size();
        ui.set_min_size(avail_size);

        ui.spacing_mut().item_spacing.y = 7.5;

        self.arena
            .update(self.inspected, |node| {
                let context = VisitorContextNodeMut {
                    key: node.key(),
                    label: &mut node.label,
                    ty: node.ty,
                    children: &mut node.children,
                };

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
        is_focused: bool,
    ) -> egui_tiles::UiResponse {
        self.draw_properties(ui).unwrap();

        egui_tiles::UiResponse::None
    }
}
