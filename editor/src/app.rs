use std::time::{Duration, Instant};

use egui_phosphor::regular::{MINUS, SQUARE, X};

use crate::config::{APP_TITLE, DEFAULT_SIZE, LAUNCH_DELAY};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CurrentPage {
    Launch,
    Intro,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum WindowState {
    #[default]
    Normal,
    Maximized,
    Minimized,
}

pub struct App {
    window_state: WindowState,
    first_frame: bool,
    /// When the app was first launched in the current session.
    /// Set to `none` when the app has fully been loaded.
    launch_timestamp: Option<Instant>,

    ctx: egui::Context,
    current_page: CurrentPage,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        Self {
            window_state: WindowState::Normal,
            first_frame: true,
            launch_timestamp: Some(Instant::now()),
            ctx: cc.egui_ctx.clone(),
            current_page: CurrentPage::Launch,
        }
    }

    fn center_window(&mut self) {
        let window_rect = self.ctx.viewport_rect();
        let sizex = window_rect.max.x - window_rect.min.x;
        let sizey = window_rect.max.y - window_rect.min.y;

        if let Some(monitor_size) = self.ctx.input(|i| i.viewport().monitor_size) {
            let monitor_pos = egui::pos2(0.0, 0.0);

            let center_x = monitor_pos.x + (monitor_size.x - sizex) / 2.0;
            let center_y = monitor_pos.y + (monitor_size.y - sizey) / 2.0;

            self.ctx
                .send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    center_x, center_y,
                )));
        }
    }

    fn draw_upper_toolbar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.menu_button("File", |ui| {
            if ui.button("Exit").clicked() {
                ui.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }

    /// Renders the home screen
    ///
    /// The home screen is simply a prompt to open a file, including a list of recently opened files.
    fn render_launch(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Mario Kart Wii editor").heading());
            });
        });
    }

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

    fn draw_decorations(&mut self, ui: &mut egui::Ui) {
        let viewport_rect = ui.ctx().viewport_rect();

        let bg_image = egui::include_image!("../images/intro_bg.png");
        egui::Image::new(bg_image).paint_at(ui, viewport_rect);

        let bg_overlay = egui::Color32::from_black_alpha(160);
        ui.painter().rect_filled(viewport_rect, 0.0, bg_overlay);

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

    fn render_intro(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                // Layout container to keep inner window centered
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25); // Top spacing

                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(40, 40, 45))
                        .corner_radius(0.0)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(65, 65, 70)))
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

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.first_frame {
            ctx.request_repaint_after(LAUNCH_DELAY);

            self.center_window();
            ctx.send_viewport_cmd(egui::ViewportCommand::Title("Launching...".into()));

            self.first_frame = false;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if let Some(launch) = self.launch_timestamp {
            if launch.elapsed() < LAUNCH_DELAY {
                self.render_launch(ui, frame);
                self.ctx.request_repaint_after(Duration::from_millis(100));
            } else {
                // Reset decorations

                self.ctx
                    .send_viewport_cmd(egui::ViewportCommand::Resizable(true));
                self.ctx
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(DEFAULT_SIZE));

                self.ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    "Mario Kart Wii editor".to_owned(),
                ));

                self.center_window();

                self.launch_timestamp = None;
                self.current_page = CurrentPage::Intro;
            }

            return;
        }

        self.draw_decorations(ui);
        match self.current_page {
            CurrentPage::Intro => self.render_intro(ui, frame),
            _ => todo!(),
        }
    }
}
