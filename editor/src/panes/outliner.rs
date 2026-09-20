use std::sync::mpsc;

use crate::{
    error::{EditorError, EditorResult, InvalidInputError},
    panes::{Pane, TreeAction},
    r#virtual::{
        defer::Deferred,
        refs::{VirtualNodeId, VirtualNodeMap},
    },
};

pub struct OutlinerPane {
    cmd_sender: mpsc::Sender<TreeAction>,
    base_node: VirtualNodeId,
    node_map: VirtualNodeMap,
}

impl OutlinerPane {
    pub fn new(
        cmd_sender: mpsc::Sender<TreeAction>,
        base_node: VirtualNodeId,
        node_map: VirtualNodeMap,
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            base_node,
            node_map,
        })
    }

    /// Draws the file tree under the current node.
    ///
    /// Lazy nodes are automatically evaluated once their folder is opened.
    ///
    /// If a specific node has been opened, this function returns the ID of its cache entry.
    fn draw_file_tree(&self, base_node: VirtualNodeId, ui: &mut egui::Ui) -> EditorResult<()> {
        ui.spacing_mut().item_spacing.y = 7.5;

        let base = self
            .node_map
            .get(base_node)
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("virtual node {} does not exist", base_node),
                    ..Default::default()
                })
            })?
            .clone();

        let mut base_ref = base.lock();

        let node_kind = base_ref.kind;
        if node_kind.is_expandable() {
            let response = egui::CollapsingHeader::new(&base_ref.label)
                .id_salt(base_node) // Different folders might have the same name, use the unique node ID
                .icon(move |ui, openness, response| {
                    let icon = if openness < 0.5 {
                        node_kind.icon_closed()
                    } else {
                        node_kind.icon_open()
                    };

                    let galley = egui::WidgetText::from(icon).into_galley(
                        ui,
                        Some(egui::TextWrapMode::Extend),
                        f32::INFINITY,
                        egui::TextStyle::Body,
                    );

                    let center_pos = response.rect.center() - (galley.size() * 0.5);

                    ui.painter()
                        .galley(center_pos, galley, ui.visuals().text_color());
                })
                .show(ui, |ui| -> EditorResult<()> {
                    // Render children if this node has already been evaluated.
                    if let Deferred::Evaluated(body) = &base_ref.body {
                        for &child in &body.children {
                            self.draw_file_tree(child, ui)?;
                        }
                    } else {
                        base_ref.evaluate()?;

                        let Deferred::Evaluated(body) = &base_ref.body else {
                            unreachable!()
                        };

                        for &child in &body.children {
                            self.draw_file_tree(child, ui)?;
                        }
                    }

                    Ok(())
                });

            // Open the properties window of the folder when clicked.
            if response.header_response.double_clicked() {
                todo!("send tree action");
            }
        } else {
            ui.horizontal(|ui| {
                ui.label(base_ref.kind.icon_closed());
                if ui.label(&base_ref.label).clicked() {
                    // opened_node_id = Some(base_ref.id);

                    todo!("send tree action");
                }
            });
        }

        Ok(())
    }
}

impl Pane for OutlinerPane {
    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("Outliner"))
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse {
        if ui.label("this is an outliner").drag_started() {
            return egui_tiles::UiResponse::DragStarted;
        }

        self.draw_file_tree(self.base_node, ui).unwrap();

        egui_tiles::UiResponse::None
    }
}
