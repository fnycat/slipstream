use egui_phosphor::regular::X;

use crate::error::{EditorError, EditorResult, InvalidInputError};
use crate::pages::editor::Editor;

impl Editor {
    pub fn draw_inspector_window(&mut self, ui: &mut egui::Ui) -> EditorResult<()> {
        let Some(open_node_id) = self.open_node else {
            // Cannot draw inspector window, as no file is open.
            return Ok(());
        };

        // Whether the current property window should be closed.
        // The properties field cannot be set immediately in the closure due to borrowing rules.
        let mut should_close = false;

        let open_node = self.ref_cache.get(open_node_id).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: format!(
                    "attempted to open stale virtual node with ID {}",
                    open_node_id
                ),
                ..Default::default()
            })
        })?;

        egui::Panel::right(egui::Id::new("property_panel")).show(ui, |ui| {
            let mut node_ref = open_node.lock();

            ui.horizontal(|ui| {
                ui.heading(&node_ref.label);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(X).clicked() {
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
            self.open_node = None;
        }

        Ok(())
    }
}
