use crate::{
    error::{EditorError, InvalidInputError},
    panes::Pane,
    r#virtual::refs::{VirtualNodeId, VirtualNodeMap},
};

pub struct InspectorPane {
    node_map: VirtualNodeMap,
    node: VirtualNodeId,
}

impl InspectorPane {
    pub fn new(node: VirtualNodeId, node_map: VirtualNodeMap) -> Box<dyn Pane> {
        Box::new(Self { node, node_map })
    }
}

impl Pane for InspectorPane {
    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("Inspector"))
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse {
        if ui.label("this is an inspector").drag_started() {
            return egui_tiles::UiResponse::DragStarted;
        }

        // Whether the current property window should be closed.
        // The properties field cannot be set immediately in the closure due to borrowing rules.
        let mut should_close = false;

        let open_node = self.node_map.get(self.node).unwrap();

        egui::Panel::right(egui::Id::new("property_panel")).show(ui, |ui| {
            let mut node_ref = open_node.lock();

            ui.horizontal(|ui| {
                ui.heading(&node_ref.label);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(reg_icon!(X)).clicked() {
                        should_close = true;
                        return;
                    }
                });
            });

            ui.add_space(0.02 * ui.available_height());

            let inspectable = node_ref.body.evaluate().unwrap().inspectable.as_mut();
            if let Some(node_content) = inspectable {
                node_content.draw_properties(ui);
            }
        });

        if should_close {
            todo!();
        }

        egui_tiles::UiResponse::None
    }
}
