use std::{io::Cursor, path::PathBuf};

use eframe::egui_wgpu;
use egui_phosphor::regular::CARET_DOWN;
use szslib::{
    arc, brres,
    yaz0::{self, YAZ0_MAGIC},
};

use crate::{
    app::App, config::FILE_INDENTATION_SIZE, model_renderer::ModelRenderer,
    shared::tree::DrawFileTree,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Filetype {
    Yaz0Arc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileData {
    Szs(arc::Archive),
    Brres(brres::Archive),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorPageData {
    pub filepath: PathBuf,
    pub data: FileData,
}

impl EditorPageData {
    pub fn from_file(filepath: &PathBuf) -> eyre::Result<FileData> {
        // This function loads only the file tree.
        // It does not read any files.

        let mut file_data = std::fs::read(&filepath)?;
        if &file_data[..4] == YAZ0_MAGIC {
            let decompressed = yaz0::decompress(&file_data)?;
            file_data = decompressed;
        }

        let archive = arc::Archive::read_filetree(&file_data)?;
        Ok(FileData::Szs(archive))
    }
}

impl App {
    fn draw_upper_toolbar(&mut self, ui: &mut egui::Ui) {
        let layout_bg = ui.style().visuals.panel_fill;
        let decorations_id = egui::Id::new("title_panel");

        egui::Panel::top(decorations_id)
            .frame(
                egui::Frame::new()
                    .fill(layout_bg)
                    .inner_margin(egui::Margin::symmetric(8, 0))
                    .outer_margin(egui::Margin::ZERO),
            )
            .resizable(false)
            .show(ui, |ui| {
                // Respond to dragging and click of the title bar.
                let response = ui.interact(
                    ui.ctx().viewport_rect(),
                    decorations_id,
                    egui::Sense::click_and_drag(),
                );

                if response.double_clicked() {
                    self.toggle_maximized(ui);
                }

                if response.drag_started() {
                    ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal_centered(|ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                ui.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });

                    self.draw_title_buttons(ui);
                });
            });
    }

    fn draw_editor_view(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let (rect, _response) =
                ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

            ui.painter()
                .add(egui_wgpu::Callback::new_paint_callback(rect, ModelRenderer))
        });
    }

    fn draw_property_window(&self, ui: &mut egui::Ui) {}

    pub fn draw_editor(&mut self, ui: &mut egui::Ui) {
        self.draw_upper_toolbar(ui);

        // Draw file explorer
        let panel_id = egui::Id::new("file_tree_panel");
        egui::Panel::left(panel_id).show(ui, |ui| {
            self.current_page
                .as_editor()
                .unwrap()
                .data
                .draw_tree(ui, "<null>");
        });

        self.draw_editor_view(ui);
    }
}
