use crate::panes::{PaneAction, PaneId};
use crate::{
    panes::{ContentSignature, Pane},
    shared::mem_logger::GLOBAL_MEM_LOGS,
};
use slipstream_shared::{SlipstreamError, SlipstreamResult};
use std::sync::mpsc;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
};

// These are the colours from the Campbell theme. Except that the trace level is white instead of purple.
const LEVEL_COLORS: &[egui::Color32] = &[
    egui::Color32::from_rgba_unmultiplied_const(231, 72, 86, 255), // Error
    egui::Color32::from_rgba_unmultiplied_const(249, 241, 165, 255), // Warn
    egui::Color32::from_rgba_unmultiplied_const(22, 198, 12, 255), // Info
    egui::Color32::from_rgba_unmultiplied_const(97, 214, 214, 255), // Debug
    egui::Color32::from_rgba_unmultiplied_const(242, 242, 242, 255), // Trace
];

pub struct LogPane {
    cmd_sender: mpsc::Sender<PaneAction>,
}

impl LogPane {
    pub fn new(cmd_sender: mpsc::Sender<PaneAction>) -> Box<dyn Pane> {
        Box::new(LogPane { cmd_sender })
    }
}

impl Pane for LogPane {
    fn ty(&self) -> PaneId {
        PaneId::Logs
    }

    fn title(&self) -> &str {
        "Logs"
    }

    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        is_focused: bool,
    ) -> egui_tiles::UiResponse {
        let row_height = 15.0;
        let logs = GLOBAL_MEM_LOGS.lock();

        // Tracing logs cannot be used from now on, until the function has ended.
        // Logging will cause a deadlock.

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show_rows(ui, row_height, logs.len(), |ui, view_range| {
                ui.set_min_size(ui.available_size());

                for i in view_range {
                    let log = &logs[i];
                    ui.horizontal(|ui| {
                        let label_color = match log.verbosity {
                            tracing::Level::ERROR => LEVEL_COLORS[0],
                            tracing::Level::WARN => LEVEL_COLORS[1],
                            tracing::Level::INFO => LEVEL_COLORS[2],
                            tracing::Level::DEBUG => LEVEL_COLORS[3],
                            tracing::Level::TRACE => LEVEL_COLORS[4],
                        };

                        ui.colored_label(label_color, log.verbosity.as_str());
                        ui.label(&log.message);
                    });
                }
            });

        egui_tiles::UiResponse::None
    }
}
