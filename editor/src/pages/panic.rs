use crate::app::App;

impl App {
    pub fn draw_panic_modal(&mut self, ui: &mut egui::Ui) {
        let modal_id = egui::Id::new("panic_modal");
        egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            let panic = self.panic_info.as_ref().unwrap();

            ui.heading("Panic caught:");

            let text = if let Some(&msg) = panic.downcast_ref::<&'static str>() {
                msg
            } else if let Some(msg) = panic.downcast_ref::<String>() {
                msg
            } else {
                "Unknown"
            };

            let code = egui::RichText::new(text).code().monospace();
            ui.label(code);

            if ui.button("Acknowledge").clicked() {
                self.panic_info = None;
            }
        });
    }
}
