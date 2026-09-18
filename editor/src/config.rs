use std::time::Duration;

pub const FILE_INDENTATION_SIZE: f32 = 8.0;
pub const LAUNCH_DELAY: Duration = Duration::from_millis(500);
pub const DEFAULT_SIZE: egui::Vec2 = egui::Vec2::new(800.0, 600.0);
pub const APP_TITLE: &str = "Mario Kart Wii Editor";

pub fn configure_dark_style() -> egui::Style {
    egui::Style {
        visuals: egui::Visuals {
            interact_cursor: Some(egui::CursorIcon::PointingHand),
            window_fill: egui::Color32::from_gray(30),
            // panel_fill: egui::Color32::from_gray(60),/
            panel_fill: egui::Color32::from_gray(40),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn configure_light_style() -> egui::Style {
    egui::Style {
        visuals: egui::Visuals {
            interact_cursor: Some(egui::CursorIcon::PointingHand),
            panel_fill: egui::Color32::WHITE,
            ..Default::default()
        },
        ..Default::default()
    }
}
