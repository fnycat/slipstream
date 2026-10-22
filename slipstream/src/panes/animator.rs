use std::sync::{Arc, mpsc};

use slipstream_ir::brres::ChrHeader;
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::node::IrNodeType;

use crate::cmd::AppCommand;
use crate::panes::{Pane, PaneAction, PaneId, PaneResponse};
use crate::shared::GraphicsState;
use crate::shared::object_dialog::{DialogResponse, ObjectDialog, ObjectFilter};

pub struct AnimationFrame {}

pub struct AnimationData {
    node: IrNodeKey,
    header: ChrHeader
}

pub struct AnimatorPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    arena: Arc<IrArena>,

    animation: Option<AnimationData>,
    current_frame: u16,

    dialog: Option<ObjectDialog>
}

impl AnimatorPane {
    pub fn new(cmd_sender: mpsc::Sender<PaneAction>, arena: Arc<IrArena>) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            arena,

            animation: None,
            current_frame: 0,

            dialog: None
        })
    }

    /// Called by the editor to retrieve the current animation contents.
    ///
    /// The editor then passes these to the viewer to display.
    pub fn update(&mut self) -> AnimationFrame {
        todo!()
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
        if let Some(response) = self.dialog.as_mut().map(|dialog| dialog.draw(ui)) {
            match response {
                DialogResponse::Closed => self.dialog = None,
                DialogResponse::Confirmed(key) => {
                    self.dialog = None;
                    // self.animation = Some(key)
                    self.animation = todo!();
                }
                _ => {}
            }
        }

        let Some(node_key) = &self.animation else {
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
        //
        //         self.arena.inspect(node_key, |node| {
        //             ui.label(format!("{}.chr0", node.label()));
        //         });

        PaneResponse::None
    }
}
