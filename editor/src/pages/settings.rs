use crate::app::App;

impl App {
    pub fn draw_settings(&mut self, ui: &mut egui::Ui) {
        self.draw_background(ui);
        self.draw_basic_title_bar(ui);
        self.draw_tool_buttons(ui);

        let window_bg = ui.visuals().panel_fill;

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25);
                    ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);

                    egui::Frame::new()
                        .fill(window_bg)
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.set_max_width(400.0);

                            ui.heading("Settings");
                            ui.separator();

                            ui.add_space(0.05 * ui.available_height());

                            ui.menu_button("Theme", |ui| {
                                ui.menu_button("Dark", |ui| {
                                    ui.button("Hello");
                                });

                                ui.button("White");
                            });

                            // if ui
                            //     .radio(ui.theme() == egui::Theme::Dark, "Dark mode")
                            //     .clicked()
                            // {
                            //     if ui.theme() == egui::Theme::Dark {
                            //         ui.set_theme(egui::Theme::Light);
                            //     } else {
                            //         ui.set_theme(egui::Theme::Dark);
                            //     }
                            // }
                        })
                });
            });
    }
}
