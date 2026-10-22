use std::sync::{Arc, mpsc};

use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::node::IrNodeType;

use crate::cmd::AppCommand;
use crate::panes::{Pane, PaneAction, PaneId, PaneResponse};
use crate::shared::object_dialog::{DialogResponse, ObjectDialog, ObjectFilter};

pub struct AnimatorPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    open_node: Option<IrNodeKey>,
    arena: Arc<IrArena>,

    dialog: Option<ObjectDialog>
}

impl AnimatorPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        open_node: Option<IrNodeKey>,
        arena: Arc<IrArena>
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            arena,
            open_node,

            dialog: None
        })
    }
}

impl Pane for AnimatorPane {
    fn title(&self) -> &str {
        "Animator"
    }

    fn ty(&self) -> PaneId {
        PaneId::Animator
    }

    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        is_focused: bool
    ) -> PaneResponse {
        let response = self.dialog.as_mut().map(|dialog| dialog.draw(ui));
        if response == Some(DialogResponse::Closed) {
            self.dialog = None;
        }

        let Some(node_key) = self.open_node else {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label("Open an animation");
                if ui.button("Choose a CHR0 file").clicked() {
                    // Open object dialog
                    self.dialog = Some(ObjectDialog::new(
                        ObjectFilter::Type(IrNodeType::ChrHeader),
                        self.arena.clone()
                    ));
                }
            });

            return PaneResponse::None;
        };

        self.arena.inspect(node_key, |node| {
            ui.label(format!(
                "You have opened node {node_key:?} with name {}",
                node.label()
            ));
        });

        PaneResponse::None
    }
}
