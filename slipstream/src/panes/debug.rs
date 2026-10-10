use crate::panes::{Pane, PaneId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DebugPane {
    General,
    Inspection,
    Textures,
    Loaders,
    Memory,
    Style,
}

impl Pane for DebugPane {
    fn title(&self) -> &'static str {
        match self {
            Self::General => "GUI Settings",
            Self::Inspection => "Widget Inspector",
            Self::Textures => "Texture Statistics",
            Self::Loaders => "Image Loader Statistics",
            Self::Memory => "Memory Statistics",
            Self::Style => "Style Settings",
        }
    }

    #[inline]
    fn ty(&self) -> PaneId {
        PaneId::Debug
    }

    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        is_focused: bool,
    ) -> egui_tiles::UiResponse {
        let ctx = ui.ctx().clone();
        match self {
            Self::General => ctx.settings_ui(ui),
            Self::Inspection => ctx.inspection_ui(ui),
            Self::Textures => ctx.texture_ui(ui),
            Self::Loaders => ctx.loaders_ui(ui),
            Self::Memory => ctx.memory_ui(ui),
            Self::Style => ctx.style_ui(ui, ui.theme()),
        }

        egui_tiles::UiResponse::None
    }
}
