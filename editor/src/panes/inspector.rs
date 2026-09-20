use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::mpsc,
};

use crate::{
    error::{EditorError, InvalidInputError},
    node::refs::{VirtualNodeId, VirtualNodeMap},
    panes::{ContentSignature, Pane, PaneAction},
};

pub struct InspectorPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    content_sig: ContentSignature,

    node_map: VirtualNodeMap,
    node: VirtualNodeId,
}

impl InspectorPane {
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        content_sig: ContentSignature,
        node: VirtualNodeId,
        node_map: VirtualNodeMap,
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            content_sig,
            node,
            node_map,
        })
    }
}

impl Pane for InspectorPane {
    fn content_signature(&self) -> ContentSignature {
        self.content_sig
    }

    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("Inspector"))
    }

    fn draw(&mut self, ui: &mut egui::Ui, tile_id: egui_tiles::TileId) -> egui_tiles::UiResponse {
        let open_node = self.node_map.get(self.node).unwrap();
        let mut node_ref = open_node.lock();

        ui.horizontal(|ui| {
            ui.heading(&node_ref.label);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(reg_icon!(X)).clicked() {
                    self.cmd_sender.send(PaneAction::RemoveTile(tile_id));
                }
            });
        });

        ui.add_space(0.02 * ui.available_height());

        let inspectable = node_ref.body.evaluate().unwrap().inspectable.as_mut();
        if let Some(node_content) = inspectable {
            node_content.draw_properties(ui);
        }

        egui_tiles::UiResponse::None
    }
}
