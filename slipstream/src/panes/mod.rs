use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::mpsc,
};

use egui_phosphor::thin::X;
use slipstream_ir::node::arena::IrNodeKey;

pub mod inspector;
pub mod log;
pub mod outliner;

/// Unlike the `egui_tiles`'s [`TileId`], this ID is created based on the content of the pane.
///
/// This makes it possible to detect whether a newly created pane already exists.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ContentSignature(u64);

impl From<u64> for ContentSignature {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

pub const PANE_HEADER_MARGIN: f32 = 5.0;
pub const PANE_TITLE_SIZE: f32 = 15.0;

/// A general pane that can be used in the editor.
///
/// The tile manager stores every pane as a trait object of this type and does
/// not know about the pane contents.
pub trait Pane: Send + Sync {
    fn ty(&self) -> PaneId;
    /// The title of the current pane.
    fn title(&self) -> &str;

    fn draw_pane_header(&self, ui: &mut egui::Ui) -> egui_tiles::UiResponse {
        let mut drag_response = egui_tiles::UiResponse::None;
        egui::Frame::new()
            .inner_margin(PANE_HEADER_MARGIN)
            .fill(ui.visuals().faint_bg_color)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .interact(
                            ui.available_rect_before_wrap(),
                            ui.id().with("pane_header"),
                            egui::Sense::drag(),
                        )
                        .drag_started()
                    {
                        drag_response = egui_tiles::UiResponse::DragStarted;
                    }

                    let label = egui::Label::new(
                        egui::RichText::new(self.title())
                            .heading()
                            .size(PANE_TITLE_SIZE),
                    )
                    .selectable(false);

                    ui.add(label);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button = egui::Button::new(X).fill(egui::Color32::TRANSPARENT);
                        if ui.add(button).clicked() {
                            todo!("close")
                        }
                    })
                });
            });

        drag_response
    }

    /// Draws the pane window itself.
    ///
    /// The pane can override this to change the window decorations.
    fn draw_window(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse {
        let mut response = self.draw_pane_header(ui);

        egui::Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(5.0)
            .show(ui, |ui| {
                if self.draw_content(ui, tile_id) == egui_tiles::UiResponse::DragStarted {
                    response = egui_tiles::UiResponse::DragStarted;
                }
            });

        response
    }

    /// Draws the content of the pane.
    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse;

    fn highlight(&self, _painter: &mut egui::Painter) {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PaneId {
    Outliner,
    Inspector,
    Viewer,
    Logs,
}

/// Requests to the tile manager to open a new pane.
///
/// If a pane with the exact same content signature is found, that pane will be focused instead.
/// If no equivalent pane is found, a new one will be opened.
///
/// See [``] for an explanation on how it is decided where to put the new pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestNewPane {
    /// Opens a new [`Outliner`]
    ///
    /// [`Outliner`]: outliner::OutlinerPane.
    Outliner {
        root: IrNodeKey,
    },
    Inspector {
        inspected: IrNodeKey,
    },
    Viewer {
        viewed: Option<IrNodeKey>,
    },
    Log,
}

impl RequestNewPane {
    pub const fn id(&self) -> PaneId {
        match self {
            Self::Outliner { .. } => PaneId::Outliner,
            Self::Inspector { .. } => PaneId::Inspector,
            Self::Viewer { .. } => PaneId::Viewer,
            Self::Log => PaneId::Logs,
        }
    }

    /// Computes the content signature of the new pane.
    ///
    /// This is compared with existing tiles.
    pub fn content_signature(&self) -> ContentSignature {
        let pane_id = self.id();

        let mut hasher = DefaultHasher::new();
        pane_id.hash(&mut hasher);

        match self {
            RequestNewPane::Outliner { root } => {
                root.hash(&mut hasher);
            }
            RequestNewPane::Inspector { inspected } => {
                inspected.hash(&mut hasher);
            }
            RequestNewPane::Viewer { viewed } => {
                viewed.hash(&mut hasher);
            }
            RequestNewPane::Log => {}
        }

        ContentSignature::from(hasher.finish())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPaneEdit {
    pub tile_id: egui_tiles::TileId,
    pub new_node: IrNodeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneAction {
    /// Requests a pane to be modified.
    ///
    /// This generally means changing the open node.
    RequestPaneEdit(RequestPaneEdit),
    /// Request a pane to be created.
    ///
    /// If this specific pane already exists, it will become active instead.
    RequestNewPane(RequestNewPane),
    /// Removes the pane with the given tile ID.
    RemoveTile(egui_tiles::TileId),
}

pub struct PaneBehavior {
    pub sender: mpsc::Sender<PaneAction>,
    pub receiver: mpsc::Receiver<PaneAction>,

    pub focused_tile: Option<egui_tiles::TileId>,
}

impl egui_tiles::Behavior<Box<dyn Pane>> for PaneBehavior {
    fn tab_title_for_pane(&mut self, pane: &Box<dyn Pane>) -> egui::WidgetText {
        pane.title().into()
    }

    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<Box<dyn Pane>>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        true
    }

    fn paint_drag_preview(
        &self,
        visuals: &egui::Visuals,
        painter: &egui::Painter,
        parent_rect: Option<egui::Rect>,
        preview_rect: egui::Rect,
    ) {
        let preview_stroke = self.drag_preview_stroke(visuals);
        let preview_color = self.drag_preview_color(visuals);

        if let Some(parent_rect) = parent_rect {
            painter.rect_stroke(parent_rect, 0.0, preview_stroke, egui::StrokeKind::Inside);
        }

        painter.rect(
            preview_rect,
            0.0,
            preview_color,
            preview_stroke,
            egui::StrokeKind::Inside,
        );
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut Box<dyn Pane>,
    ) -> egui_tiles::UiResponse {
        pane.draw_window(ui, tile_id)
    }
}
