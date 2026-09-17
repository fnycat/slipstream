use egui_phosphor::regular::X;

use crate::app::App;
use crate::error::{EditorError, EditorResult, InvalidInputError};

impl App {
    pub fn draw_inspector_window(&mut self, ui: &mut egui::Ui) -> EditorResult<()> {
        let editor = self.current_page.as_editor_mut().unwrap();
        let Some(props) = &editor.open_properties else {
            return Ok(());
        };

        // Whether the current property window should be closed.
        // The properties field cannot be set immediately in the closure due to borrowing rules.
        let mut should_close = false;

        let node_ref = editor.ref_cache.get(props.node_id).ok_or_else(|| {
            EditorError::from(InvalidInputError {
                reason: format!(
                    "attempted to open stale virtual node with ID {}",
                    props.node_id
                ),
                ..Default::default()
            })
        })?;

        let mut node = node_ref.borrow_mut();
        let node_content = node.content.evaluate()?.inspectable.as_mut().unwrap();

        egui::Panel::right(egui::Id::new("property_panel")).show(ui, |ui| {
            ui.set_max_width(475.0);

            ui.horizontal(|ui| {
                ui.heading(&props.label);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(X).clicked() {
                        should_close = true;
                        return;
                    }
                });
            });

            ui.label("TODO: ./driver_model.brres/3DModels(NW4R)/model/Bones/mouth_1");
            ui.add_space(0.02 * ui.available_height());

            if editor.open_properties.is_some() {
                node_content.draw_properties(ui);
            }
        });

        if should_close {
            editor.open_properties = None;
        }

        Ok(())
    }
}
