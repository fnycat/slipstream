use crate::app::App;

impl App {
    pub fn draw_animator_window(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("animator_panel").show(ui, |ui| {
            ui.heading("Keyframes");
        });
    }
}
