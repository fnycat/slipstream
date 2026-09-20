use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
};

use crate::panes::{ContentSignature, Pane};

/// All log panes have the same ID because they simply show the same content.
static LOG_PANE_CONTENT_ID: LazyLock<ContentSignature> = LazyLock::new(|| {
    let mut hasher = DefaultHasher::new();
    "logs".hash(&mut hasher);

    ContentSignature(hasher.finish())
});

pub struct LogPane;

impl LogPane {
    pub fn new() -> Box<dyn Pane> {
        Box::new(LogPane)
    }
}

impl Pane for LogPane {
    fn content_signature(&self) -> ContentSignature {
        *LOG_PANE_CONTENT_ID
    }

    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("Logs"))
    }

    fn draw(&mut self, ui: &mut egui::Ui, tile_id: egui_tiles::TileId) -> egui_tiles::UiResponse {
        todo!()
    }
}
