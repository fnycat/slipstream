use egui::load::TexturePoll;

use crate::{app::App, config::APP_TITLE};

impl App {
    fn poll_preload(&mut self) -> Result<egui::load::TexturePoll, egui::load::LoadError> {
        let bg_image = egui::include_image!("../../images/intro_bg.png");
        bg_image.load(
            &self.ctx,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                mipmap_mode: None,
                wrap_mode: egui::TextureWrapMode::Repeat,
            },
            egui::SizeHint::Size {
                width: 3840,
                height: 2160,
                maintain_aspect_ratio: true,
            },
        )
    }

    /// Renders the splash screen
    pub fn draw_splash(&mut self, ui: &mut egui::Ui) {
        egui::Area::new(egui::Id::new("splash_panel"))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui, |ui| {
                ui.set_max_width(360.0);

                ui.vertical_centered(|ui| {
                    let title_text = egui::RichText::new(APP_TITLE);
                    let title = egui::Label::new(title_text).selectable(false);
                    ui.add(title);

                    ui.add_space(0.2 * ui.available_height());

                    let spinner = egui::Spinner::new().size(32.0);
                    ui.add(spinner);
                });
            });

        let poll = self.poll_preload();
        match poll {
            Ok(TexturePoll::Ready { texture }) => {
                tracing::trace!("Background image loaded");

                self.bg_image = Some(texture);
                self.preload_finished = true;
            }
            Ok(TexturePoll::Pending { .. }) => {
                ui.ctx().request_repaint();
            }
            Err(err) => {
                tracing::error!("failed to load background texture: {err}");
            }
        }
    }
}
