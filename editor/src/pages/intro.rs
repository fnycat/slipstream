use std::sync::mpsc;

use crate::app::App;

impl App {
    pub fn draw_intro(&mut self, ui: &mut egui::Ui) {
        let window_bg = ui.visuals().panel_fill;

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ui, |ui| {
                // Layout container to keep inner window centered
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25); // Top spacing

                    egui::Frame::new()
                        .fill(window_bg)
                        .corner_radius(0.0)
                        // .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(65, 65, 70)))
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.set_max_width(360.0);

                            ui.heading("Centered container");
                            ui.separator();
                            ui.label("Fixed-position launcher UI.");
                        });
                });
            });
    }
}
