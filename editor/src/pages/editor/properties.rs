use crate::app::App;

impl App {
    pub fn draw_property_window(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom(egui::Id::new("property_panel")).show(ui, |ui| {
            ui.heading("Property window");
            ui.separator();
        });
    }
}
