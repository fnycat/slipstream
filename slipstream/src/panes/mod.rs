use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::{BitOr, BitOrAssign};
use std::sync::mpsc;

use egui_phosphor::thin::X;
use slipstream_ir::node::arena::IrNodeKey;

use crate::panes::debug::DebugPane;

pub mod animator;
pub mod debug;
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

pub const PANE_MARGIN: f32 = 5.0;
pub const PANE_TITLE_SIZE: f32 = 15.0;

/// A response of a pane.
///
/// Use the `|` operator to combine responses. The priority of responses is as follows:
/// ```
/// 1. Closed
/// 2. DragStarted
/// 3. None
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PaneResponse {
    None,
    DragStarted,
    Closed
}

impl PaneResponse {
    pub fn into_tiles_response(self) -> egui_tiles::UiResponse {
        match self {
            Self::DragStarted => egui_tiles::UiResponse::DragStarted,
            _ => egui_tiles::UiResponse::None
        }
    }
}

impl BitOr for PaneResponse {
    type Output = PaneResponse;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::None, rhs) => rhs,
            (Self::DragStarted, Self::Closed) => Self::Closed,
            (Self::DragStarted, _) => Self::DragStarted,
            (Self::Closed, _) => Self::Closed
        }
    }
}

impl BitOrAssign for PaneResponse {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

/// A general pane that can be used in the editor.
///
/// The tile manager stores every pane as a trait object of this type and does
/// not know about the pane contents.
pub trait Pane: Send + Sync {
    fn ty(&self) -> PaneId;
    /// The title of the current pane.
    fn title(&self) -> &str;

    fn draw_pane_header(&self, ui: &mut egui::Ui, is_focused: bool) -> PaneResponse {
        let mut pane_response = PaneResponse::None;

        egui::Frame::new()
            .inner_margin(PANE_MARGIN)
            .fill(if is_focused {
                ui.visuals()
                    .faint_bg_color
                    .lerp_to_gamma(egui::Color32::WHITE, 0.1)
            } else {
                ui.visuals().faint_bg_color
            })
            .inner_margin(PANE_MARGIN)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .interact(
                            ui.available_rect_before_wrap(),
                            ui.id().with("pane_header"),
                            egui::Sense::drag()
                        )
                        .drag_started()
                    {
                        pane_response |= PaneResponse::DragStarted;
                    }

                    let label = egui::Label::new(
                        egui::RichText::new(self.title())
                            .heading()
                            .size(PANE_TITLE_SIZE)
                    )
                    .selectable(false);

                    ui.add(label);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let button = egui::Button::new(X).fill(egui::Color32::TRANSPARENT);
                        if ui.add(button).clicked() {
                            pane_response |= PaneResponse::Closed;
                        }
                    })
                });
            });

        pane_response
    }

    /// Draws the pane window itself.
    ///
    /// The pane can override this to change the window decorations.
    fn draw_window(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        is_focused: bool
    ) -> PaneResponse {
        let mut response = self.draw_pane_header(ui, is_focused);

        egui::Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(5.0)
            .show(ui, |ui| {
                response |= self.draw_content(ui, tile_id, is_focused);
            });

        response
    }

    /// Draws the content of the pane.
    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        is_focused: bool
    ) -> PaneResponse;

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
    Debug,
    Animator
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
        root: IrNodeKey
    },
    Inspector {
        inspected: IrNodeKey
    },
    Viewer {
        viewed: Option<IrNodeKey>
    },
    Log,
    Debug {
        ty: DebugPane
    }
}

impl RequestNewPane {
    pub const fn id(&self) -> PaneId {
        match self {
            Self::Outliner { .. } => PaneId::Outliner,
            Self::Inspector { .. } => PaneId::Inspector,
            Self::Viewer { .. } => PaneId::Viewer,
            Self::Log => PaneId::Logs,
            Self::Debug { .. } => PaneId::Debug
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
            RequestNewPane::Debug { ty } => ty.hash(&mut hasher)
        }

        ContentSignature::from(hasher.finish())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPaneEdit {
    pub tile_id: egui_tiles::TileId,
    pub new_node: IrNodeKey
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
    RemoveTile(egui_tiles::TileId)
}

pub struct PaneBehavior {
    pub sender: mpsc::Sender<PaneAction>,
    pub receiver: mpsc::Receiver<PaneAction>,

    pub focused_pane: Option<egui_tiles::TileId>
}

impl egui_tiles::Behavior<Box<dyn Pane>> for PaneBehavior {
    fn tab_title_for_pane(&mut self, pane: &Box<dyn Pane>) -> egui::WidgetText {
        pane.title().into()
    }

    #[inline]
    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<Box<dyn Pane>>,
        _tile_id: egui_tiles::TileId
    ) -> bool {
        true
    }

    #[inline]
    fn tab_hover_cursor_icon(&self) -> egui::CursorIcon {
        egui::CursorIcon::PointingHand
    }

    fn paint_drag_preview(
        &self,
        visuals: &egui::Visuals,
        painter: &egui::Painter,
        parent_rect: Option<egui::Rect>,
        preview_rect: egui::Rect
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
            egui::StrokeKind::Inside
        );
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut Box<dyn Pane>
    ) -> egui_tiles::UiResponse {
        let pane_rect = ui.cursor();
        let clicked = ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|pos| pane_rect.contains(pos))
        });

        if clicked {
            // User clicked in this pane.
            self.focused_pane = Some(tile_id);
        }

        let is_focused = self.focused_pane == Some(tile_id);

        let response = pane.draw_window(ui, tile_id, is_focused);
        if response == PaneResponse::Closed {
            // The window wants to be closed.
            if let Err(err) = self.sender.send(PaneAction::RemoveTile(tile_id)) {
                tracing::error!("Failed to send pane close command: {err:?}");
            };
        }

        response.into_tiles_response()
    }
}
