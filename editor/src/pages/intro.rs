use egui_phosphor::regular::{GEAR_FINE, GITHUB_LOGO, INFO, MOON, POWER, SUN};

use crate::{
    app::{App, CurrentPage},
    model_renderer::ModelRenderer,
    pages::editor::EditorPageData,
};

impl App {
    pub fn draw_intro(&mut self, ui: &mut egui::Ui) {
        self.draw_version_details(ui);
        self.draw_background(ui);
        self.draw_basic_title_bar(ui);

        let window_bg = ui.visuals().panel_fill;

        egui::CentralPanel::default()
            .frame(egui::Frame::default())
            .show(ui, |ui| {
                // Layout container to keep inner window centered
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25); // Top spacing
                    ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);

                    egui::Frame::new()
                        .fill(window_bg)
                        .corner_radius(0.0)
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.set_max_width(360.0);

                            ui.heading("Recent files");
                            ui.separator();

                            ui.add_space(0.05 * ui.available_height());

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

                    egui::Frame::new()
                        .fill(window_bg)
                        .corner_radius(0.0)
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.set_max_width(360.0);

                            if ui.button("Open another file").clicked() {
                                let selected_file = rfd::FileDialog::new()
                                    .set_title("Select a file to edit")
                                    .pick_file();

                                if let Some(selected_file) = selected_file {
                                    ModelRenderer::init(&self.render_state);

                                    self.current_page = CurrentPage::Editor(
                                        EditorPageData::new(selected_file).unwrap(),
                                    );
                                }
                            }
                        });
                });
            });

        egui::Area::new(egui::Id::new("home_tool_buttons"))
            .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-10.0, 0.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(10.0, 10.0);
                    ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);

                    let power_button = egui::Button::new(POWER);
                    if ui
                        .add(power_button)
                        .on_hover_text_at_pointer("Quit")
                        .clicked()
                    {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    };

                    if ui.theme() == egui::Theme::Dark {
                        if ui
                            .button(SUN)
                            .on_hover_text("Switch to light theme")
                            .clicked()
                        {
                            ui.ctx().set_theme(egui::Theme::Light);
                        }
                    } else {
                        if ui
                            .button(MOON)
                            .on_hover_text("Switch to dark theme")
                            .clicked()
                        {
                            ui.ctx().set_theme(egui::Theme::Dark);
                        }
                    }

                    if ui
                        .button(GEAR_FINE)
                        .on_hover_text("Open settings")
                        .clicked()
                    {
                        self.current_page = CurrentPage::Settings;
                    }

                    if ui.button(INFO).on_hover_text("Open app info").clicked() {
                        self.current_page = CurrentPage::Info;
                    }

                    if ui
                        .button(GITHUB_LOGO)
                        .on_hover_text("Open the project on GitHub")
                        .clicked()
                    {
                        ui.ctx().open_url(egui::OpenUrl {
                            url: "https://github.com/RadiatedMonkey/mktools".to_owned(),
                            new_tab: true,
                        });
                    }
                });
            });
    }
}
