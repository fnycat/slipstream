use std::{
    any::Any,
    panic::AssertUnwindSafe,
    time::{Duration, Instant},
};

use eframe::egui_wgpu;
use egui_phosphor::regular::{MINUS, SQUARE, X};

use crate::{
    config::{APP_TITLE, DEFAULT_SIZE, LAUNCH_DELAY, configure_dark_style, configure_light_style},
    decorations::WindowState,
    pages::editor::EditorPageData,
};

#[derive(Debug, Clone, PartialEq)]
pub enum CurrentPage {
    Splash,
    Intro,
    Editor(EditorPageData),
}

impl CurrentPage {
    pub fn as_editor(&self) -> Option<&EditorPageData> {
        match self {
            Self::Editor(data) => Some(data),
            _ => None,
        }
    }
}

pub struct App {
    pub panic_info: Option<Box<dyn Any + Send>>,
    pub bg_image: Option<egui::load::SizedTexture>,

    pub render_state: egui_wgpu::RenderState,
    pub window_state: WindowState,
    pub first_frame: bool,
    pub preload_finished: bool,
    /// When the app was first launched in the current session.
    /// Set to `none` when the app has fully been loaded.
    pub launch_timestamp: Option<Instant>,

    pub ctx: egui::Context,
    pub current_page: CurrentPage,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        cc.egui_ctx
            .set_style_of(egui::Theme::Dark, configure_dark_style());

        cc.egui_ctx
            .set_style_of(egui::Theme::Light, configure_light_style());

        Self {
            render_state: cc.wgpu_render_state.as_ref().unwrap().clone(),
            panic_info: None,
            preload_finished: false,
            bg_image: None,
            window_state: WindowState::Normal,
            first_frame: true,
            launch_timestamp: Some(Instant::now()),
            ctx: cc.egui_ctx.clone(),
            current_page: CurrentPage::Splash,
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

    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        // Draw panic modal if a panic occurred
        if self.panic_info.is_some() {
            self.draw_panic_modal(ui);
        }

        if let Some(launch) = self.launch_timestamp {
            if launch.elapsed() < LAUNCH_DELAY || !self.preload_finished {
                self.draw_splash(ui);
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

                // `center_window` does not work here since it would still be using the old window size.

                let mut window_rect = self.ctx.viewport_rect();
                // adjust the existing window rect to include the new size.
                window_rect.max = window_rect.min + DEFAULT_SIZE;

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

                self.launch_timestamp = None;
                self.current_page = CurrentPage::Intro;
            }

            return;
        }

        match &self.current_page {
            CurrentPage::Intro => self.draw_intro(ui),
            CurrentPage::Editor { .. } => self.draw_editor(ui),
            v => todo!("{v:?}"),
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            ctx.request_repaint_after(LAUNCH_DELAY);

            self.center_window();

            ctx.send_viewport_cmd(egui::ViewportCommand::Transparent(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Title("Launching...".into()));

            self.first_frame = false;
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Err(err) = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.draw_ui(ui);
        })) {
            tracing::error!("{err:?}");
            self.panic_info = Some(err);
        }
    }
}
