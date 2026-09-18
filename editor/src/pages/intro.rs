use crate::{
    app::{App, CurrentPage},
    pages::editor::Editor,
    viewer::Viewer,
};

egui_phosphor::subset! {
    pub mod icons {
        use regular::{GEAR_FINE, GITHUB_LOGO, INFO, MOON, POWER, SUN};
    }
}

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

                    let mut frame_width = 0.0;

                    egui::Frame::new()
                        .fill(window_bg)
                        .corner_radius(0.0)
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.set_max_width(400.0);

                            frame_width = ui.available_width();

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
                            ui.set_width(frame_width);
                            ui.set_height(40.0);

                            ui.columns_const(|[col1, col2]| {
                                col1.vertical_centered(|ui| {
                                    if ui
                                        .button(egui::RichText::new("Open another file"))
                                        .clicked()
                                    {
                                        let selected_file = rfd::FileDialog::new()
                                            .set_title("Select a file to edit")
                                            .pick_file();

                                        if let Some(selected_file) = selected_file {
                                            self.current_page = CurrentPage::Editor(
                                                Editor::new(selected_file, &self.render_state)
                                                    .unwrap(),
                                            );
                                        }
                                    }
                                });

                                col2.vertical_centered(|ui| {
                                    ui.add_space(0.2 * ui.available_height());

                                    let label = egui::RichText::new("Or drop a file here");
                                    ui.label(label)
                                });
                            });
                        });
                });

                self.draw_tool_buttons(ui);
            });

        let hovered_file =
            ui.input_mut(|input| input.raw.hovered_files.pop().map(|file| file.path.unwrap()));

        let alpha = (ui.animate_bool_with_time(
            ui.make_persistent_id("file_drop_overlay_fade"),
            hovered_file.is_some(),
            0.25,
        ) * 100.0) as u8;

        if alpha > 0 {
            let viewport_rect = ui.ctx().viewport_rect();

            let bg_overlay = if ui.theme() == egui::Theme::Dark {
                egui::Color32::from_white_alpha(alpha)
            } else {
                egui::Color32::from_black_alpha(alpha)
            };

            ui.painter().rect_filled(viewport_rect, 0.0, bg_overlay);

            egui::Area::new(egui::Id::new("file_hover_overlay"))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .order(egui::Order::Foreground)
                .show(ui, |ui| {
                    let heading =
                        egui::RichText::new("Drop to edit file").color(egui::Color32::BLACK);
                    ui.label(heading);
                });
        }
    }

    pub fn draw_tool_buttons(&mut self, ui: &mut egui::Ui) {
        egui::Area::new(egui::Id::new("home_tool_buttons"))
            .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-10.0, 0.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(10.0, 10.0);
                    ui.spacing_mut().item_spacing = egui::vec2(5.0, 5.0);

                    let power_button = egui::Button::new(icons::regular::POWER);
                    if ui
                        .add(power_button)
                        .on_hover_text_at_pointer("Quit")
                        .clicked()
                    {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    };

                    if ui.theme() == egui::Theme::Dark {
                        if ui
                            .button(icons::regular::SUN)
                            .on_hover_text("Switch to light theme")
                            .clicked()
                        {
                            ui.ctx().set_theme(egui::Theme::Light);
                        }
                    } else {
                        if ui
                            .button(icons::regular::MOON)
                            .on_hover_text("Switch to dark theme")
                            .clicked()
                        {
                            ui.ctx().set_theme(egui::Theme::Dark);
                        }
                    }

                    if ui
                        .button(icons::regular::GEAR_FINE)
                        .on_hover_text("Open settings")
                        .clicked()
                    {
                        self.current_page = CurrentPage::Settings;
                    }

                    if ui
                        .button(icons::regular::INFO)
                        .on_hover_text("Open app info")
                        .clicked()
                    {
                        self.current_page = CurrentPage::Info;
                    }

                    if ui
                        .button(icons::regular::GITHUB_LOGO)
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
