use crate::panes::{ContentSignature, Pane, PaneAction, PaneId};
use slipstream_ir::mdl0::{
    Bone, Definitions, MaterialBuffer, Model, Polygon, TextureLinks, VertexBuffer,
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
    fn visit_mdl0_mut(&mut self, context: VisitorContextMut<'_, Model>) -> ControlFlow<()> {
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
            .draw_inspect(self.ui, &FieldConfig::default());

        ControlFlow::Continue(())
    }

    fn visit_vertices_mut(
        &mut self,
        mut context: VisitorContextMut<'_, VertexBuffer>,
    ) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_polygon_mut(&mut self, context: VisitorContextMut<'_, Polygon>) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_material_mut(
        &mut self,
        context: VisitorContextMut<'_, MaterialBuffer>,
    ) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_tex0_mut(&mut self, context: VisitorContextMut<'_, Texture>) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
        ControlFlow::Continue(())
    }

    fn visit_texture_links_mut(
        &mut self,
        context: VisitorContextMut<'_, TextureLinks>,
    ) -> ControlFlow<()> {
        self.ui.label(format!("{:#?}", context.content));
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
                egui::ScrollArea::both().show(ui, |ui| {
                    // Take up the whole pane.
                    ui.set_min_size(ui.available_size());

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
        tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse {
        self.draw_properties(ui).unwrap();

        egui_tiles::UiResponse::None
    }
}
