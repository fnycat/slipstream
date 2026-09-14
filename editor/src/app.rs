#[derive(Default)]
pub struct App {}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Hello World!");
        });
    }
}
