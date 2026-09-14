use egui_phosphor::regular::{MINUS, SQUARE, X};

use crate::{app::App, config::APP_TITLE};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum WindowState {
    #[default]
    Normal,
    Maximized,
    Minimized,
}

impl App {
    fn toggle_maximized(&mut self, ui: &mut egui::Ui) {
        if self.window_state == WindowState::Maximized {
            self.window_state = WindowState::Normal;
            ui.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
        } else {
            self.window_state = WindowState::Maximized;
            ui.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
    }

    fn draw_unmaximize(ui: &mut egui::Ui) -> egui::Response {
        let size = egui::vec2(12.0, 12.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let stroke_color = if response.hovered() {
                ui.visuals().widgets.hovered.fg_stroke.color
            } else {
                ui.visuals().widgets.noninteractive.fg_stroke.color
            };

            let stroke = egui::Stroke::new(1.0, stroke_color);

            let box_size = egui::vec2(8.0, 8.0);
            let back_min = rect.min + egui::vec2(3.0, 0.0);
            let back_rect = egui::Rect::from_min_size(back_min, box_size);

            painter.line_segment([back_rect.left_top(), back_rect.right_top()], stroke);

            painter.line_segment([back_rect.right_top(), back_rect.right_bottom()], stroke);

            let front_min = rect.min + egui::vec2(0.0, 3.0);
            let front_rect = egui::Rect::from_min_size(front_min, box_size);

            painter.rect_filled(front_rect, 0.0, ui.visuals().window_fill());
            painter.rect_stroke(front_rect, 0.0, stroke, egui::StrokeKind::Middle);
        }

        response
    }

    pub fn draw_version_details(&self, ui: &mut egui::Ui) {
        let screen_rect = ui.viewport_rect();
        let pos = egui::pos2(screen_rect.center().x, screen_rect.max.y - 12.0);

        ui.ctx()
            .layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("version_overlay"),
            ))
            .text(
                pos,
                egui::Align2::CENTER_BOTTOM,
                format!(
                    "Version {} ({})",
                    env!("CARGO_PKG_VERSION"),
                    &env!("VERGEN_GIT_SHA")[..8]
                ),
                egui::FontId::proportional(12.0),
                egui::Color32::from_white_alpha(200),
            );
    }

    /// Draws the background image with a gray overlay.
    pub fn draw_background(&self, ui: &mut egui::Ui) {
        let viewport_rect = ui.ctx().viewport_rect();

        let bg_image = self.bg_image.unwrap();
        egui::Image::new(bg_image).paint_at(ui, viewport_rect);

        let bg_overlay = egui::Color32::from_black_alpha(160);
        ui.painter().rect_filled(viewport_rect, 0.0, bg_overlay);
    }

    /// Draws the window title and buttons (close, minimize, maximize).
    pub fn draw_title_bar(&mut self, ui: &mut egui::Ui) {
        let decorations_id = egui::Id::new("decorations_panel");
        let layout_bg = ui.style().visuals.panel_fill;

        egui::Panel::top(decorations_id)
            .frame(
                egui::Frame::new()
                    .fill(layout_bg)
                    .inner_margin(egui::Margin::ZERO)
                    .outer_margin(egui::Margin::ZERO),
            )
            .resizable(false)
            .show(ui, |ui| {
                // Respond to dragging and click of the title bar.
                let response = ui.interact(
                    ui.ctx().viewport_rect(),
                    decorations_id,
                    egui::Sense::click_and_drag(),
                );

                if response.double_clicked() {
                    self.toggle_maximized(ui);
                }

                if response.drag_started() {
                    ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal_centered(|ui| {
                    // Draws the title text in the center.
                    let rect = ui.max_rect();
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        APP_TITLE,
                        egui::FontId::proportional(14.0),
                        ui.visuals().text_color(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let old_hover = ui.visuals().widgets.hovered.weak_bg_fill;

                        // Make the close button red on hover.
                        let red_color = ui.visuals().error_fg_color;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill = red_color;

                        // Remove margins between buttons
                        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

                        // Disable stroke
                        ui.visuals_mut().widgets.hovered.bg_stroke =
                            egui::Stroke::new(0.0, egui::Color32::BLACK);

                        ui.visuals_mut().widgets.active.bg_stroke =
                            egui::Stroke::new(0.0, egui::Color32::BLACK);

                        // Set button background to layout background
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = layout_bg;
                        ui.spacing_mut().button_padding = egui::vec2(16.0, 8.0);

                        let close_button = egui::Button::new(X).corner_radius(0.0);
                        if ui.add(close_button).clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // Don't make the other buttons red on hover.
                        ui.visuals_mut().widgets.hovered.weak_bg_fill = old_hover;

                        if self.window_state == WindowState::Maximized {
                            if Self::draw_unmaximize(ui).clicked() {
                                self.toggle_maximized(ui);
                            }
                        } else {
                            let maximize_button = egui::Button::new(SQUARE).corner_radius(0.0);
                            if ui.add(maximize_button).clicked() {
                                self.toggle_maximized(ui);
                            }
                        }

                        let minimize_button = egui::Button::new(MINUS).corner_radius(0.0);
                        if ui.add(minimize_button).clicked() {
                            self.window_state = WindowState::Minimized;
                            ui.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                })
            });
    }
}
