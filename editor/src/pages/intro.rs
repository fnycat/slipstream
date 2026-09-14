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

                            ui.heading("Recent files");
                            ui.separator();

                            ui.add_space(0.05 * ui.available_height());

                            ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);

                            ui.columns(2, |ui| {
                                for ui in ui {
                                    for i in 0..4 {
                                        if ui.button(format!("driver_{i}.brres")).clicked() {
                                            tracing::debug!("whooshdsd");
                                        }
                                    }
                                }
                            });
                        });
                });
            });
    }
}
