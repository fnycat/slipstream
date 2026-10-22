mod visitor;

use crate::inspector::visitor::InspectorVisitor;
use crate::panes::{Pane, PaneAction, PaneId, PaneResponse};
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::VisitorCtxMetaMut;
use slipstream_shared::{SlipstreamError, SlipstreamResult};
use std::sync::{Arc, mpsc};

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
    ) -> Box<Self> {
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
    ) -> PaneResponse {
        self.draw_properties(ui).unwrap();

        PaneResponse::None
    }
}
