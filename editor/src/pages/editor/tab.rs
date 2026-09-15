use crate::{app::App, pages::editor::EditorPageData, shared::uri::Uri};

pub struct EditorTab {
    pub uri: Uri,
}

impl EditorPageData {
    pub fn draw_tab_list(&self, ui: &mut egui::Ui) {
        let id = egui::Id::new("tab_list_panel");
        egui::Panel::top(id).show(ui, |ui| {
            for tab in &self.tabs {
                // ui.button(&tab.name);
                ui.button("test tab");
            }
        });
    }
}
