use std::sync::mpsc;

pub mod inspector;
pub mod outliner;
pub mod viewer;

pub trait Pane: Send {
    fn title(&self) -> egui::WidgetText;
    fn pane_ui(&mut self, ui: &mut egui::Ui, tile_id: egui_tiles::TileId)
    -> egui_tiles::UiResponse;
}

pub enum TreeAction {
    AddTile {
        parent: egui_tiles::TileId,
        pane: Box<dyn Pane>,
    },
    RemoveTile(egui_tiles::TileId),
}

pub struct PaneBehavior {
    pub cmd_receiver: mpsc::Receiver<TreeAction>,
}

impl egui_tiles::Behavior<Box<dyn Pane>> for PaneBehavior {
    fn tab_title_for_pane(&mut self, pane: &Box<dyn Pane>) -> egui::WidgetText {
        pane.title()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut Box<dyn Pane>,
    ) -> egui_tiles::UiResponse {
        pane.pane_ui(ui, tile_id)
    }
}
