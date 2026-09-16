use egui_phosphor::regular::X;

use crate::{app::App, pages::editor::Editor};

impl App {
    pub fn draw_property_window(&mut self, ui: &mut egui::Ui) -> eyre::Result<()> {
        let editor = self.current_page.as_editor_mut().unwrap();
        let Some(cache_id) = editor.open_properties else {
            return Ok(());
        };

        let cache = editor.cache_store.get_mut(cache_id)?;

        egui::Panel::right(egui::Id::new("property_panel")).show(ui, |ui| {
            ui.heading("Properties");
            ui.separator();

            if ui.button(X).clicked() {
                editor.open_properties = None;
                return;
            }

            cache.draw_properties(ui);
        });

        Ok(())
    }
}
