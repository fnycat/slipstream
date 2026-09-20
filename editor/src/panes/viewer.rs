use crate::panes::Pane;

pub struct ViewerPane {}

impl ViewerPane {
    pub fn new() -> Box<dyn Pane> {
        Box::new(Self {})
    }
}

impl Pane for ViewerPane {
    fn title(&self) -> egui::WidgetText {
        egui::WidgetText::Text(String::from("3D Viewer"))
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse {
        if ui.label("this is a viewer").drag_started() {
            return egui_tiles::UiResponse::DragStarted;
        }

        egui_tiles::UiResponse::None
    }
}
