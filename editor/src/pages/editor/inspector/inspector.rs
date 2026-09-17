use egui_phosphor::regular::X;

use crate::app::App;
use crate::error::{EditorError, EditorResult, InvalidInputError};
use crate::r#virtual::node::VirtualNodeRefExt;
use crate::r#virtual::refs::VirtualRefCacheExt;

impl App {
    pub fn draw_inspector_window(&mut self, ui: &mut egui::Ui) -> EditorResult<()> {
        let editor = self.current_page.as_editor_mut().unwrap();
        let Some(open_node_id) = editor.open_node else {
            return Ok(());
        };

        // Whether the current property window should be closed.
        // The properties field cannot be set immediately in the closure due to borrowing rules.
        let mut should_close = false;

        let node_ref = editor.ref_cache.get(open_node_id).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: format!(
                    "attempted to open stale virtual node with ID {}",
                    open_node_id
                ),
                ..Default::default()
            })
        })?;

        egui::Panel::right(egui::Id::new("property_panel")).show(ui, |ui| {
            ui.set_max_width(475.0);

            ui.horizontal(|ui| {
                ui.heading(node_ref.label());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(X).clicked() {
                        should_close = true;
                        return;
                    }
                });
            });

            ui.label("TODO: ./driver_model.brres/3DModels(NW4R)/model/Bones/mouth_1");
            ui.add_space(0.02 * ui.available_height());

            let mut node = node_ref.borrow_mut();
            let inspectable = node.content.evaluate().unwrap().inspectable.as_mut();

            if let Some(node_content) = inspectable {
                node_content.draw_properties(ui);
            }
        });

        if should_close {
            editor.open_node = None;
        }

        Ok(())
    }
}
