use std::{
    any::Any,
    panic::AssertUnwindSafe,
    sync::Arc,
};

use eframe::egui_wgpu;

use crate::{
    cmd::{AppCommand, AppCommandChannel},
    config::{configure_dark_style, configure_light_style},
    decorations::handle_frameless_resize,
    pages::{RoutablePage, splash::SplashPage},
    viewer::{self},
};

const CHANNEL_SIZE: usize = 50;

pub struct App {
    pub rx: futures::channel::mpsc::Receiver<AppCommand>,

    pub panic_info: Option<Box<dyn Any + Send>>,
    pub bg_image: Option<egui::load::SizedTexture>,

    pub render_state: egui_wgpu::RenderState,

    pub ctx: egui::Context,
    pub page_state: Box<dyn RoutablePage>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        tracing::info!("Initializing app...");

        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        egui_phosphor::add_font_bytes_as_family(
            &mut fonts,
            "mktools::icons::icons::fill",
            egui_phosphor::bytes::fill::FONT,
        );

        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_theme(egui::Theme::Dark);

        cc.egui_ctx
            .set_style_of(egui::Theme::Dark, configure_dark_style());

        cc.egui_ctx
            .set_style_of(egui::Theme::Light, configure_light_style());

        let (tx, rx) = futures::channel::mpsc::channel(CHANNEL_SIZE);
        let cmd_channel = AppCommandChannel::new(tx);

        let egui_rs = cc.wgpu_render_state.as_ref().unwrap();
        let render_state = viewer::RenderState {
            instance: egui_rs.instance.clone(),
            device: egui_rs.device.clone(),
            queue: egui_rs.queue.clone(),
            renderer: Arc::clone(&egui_rs.renderer),
        };

        Self {
            rx,

            render_state: cc.wgpu_render_state.as_ref().unwrap().clone(),
            panic_info: None,
            bg_image: None,
            ctx: cc.egui_ctx.clone(),
            page_state: SplashPage::new(cc.egui_ctx.clone(), render_state, cmd_channel),
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
        handle_frameless_resize(ui);

        // Draw panic modal if a panic occurred
        if self.panic_info.is_some() {
            self.draw_panic_modal(ui);
        }

        self.page_state.draw(ui).unwrap();
    }

    pub fn handle_command(&mut self, cmd: AppCommand) {
        match cmd {
            AppCommand::CenterWindow => self.center_window(),
            AppCommand::Route(route) => {
                tracing::debug!("Routed to page `{}`", route.name());
                self.page_state = route
            }
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process channel commands
        let mut recv_result = self.rx.try_recv();
        while let Ok(cmd) = recv_result {
            self.handle_command(cmd);
            recv_result = self.rx.try_recv();
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
