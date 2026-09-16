use std::{io::Cursor, path::PathBuf, rc::Rc, sync::Arc};

use eframe::egui_wgpu;

use crate::{
    app::{App, CurrentPage},
    model_renderer::ModelRenderer,
    shared::{
        node::{self},
        r#virtual::{ResourceCache, VirtualNode, VirtualNodeKind},
    },
};

/// Data specific to the editor page.
pub struct EditorPageData {
    /// The path of the current file open in the editor.
    ///
    /// This is a regular filesystem path, pointing to the root file.
    /// Not an internal URI.
    pub filepath: PathBuf,
    /// The whole file currently open in the editor.
    pub root_node: VirtualNode,
    pub res_store: ResourceCache,
}

impl EditorPageData {
    pub fn new(filepath: PathBuf) -> eyre::Result<Self> {
        let contents = Cursor::new(std::fs::read(&filepath)?);
        let file_name = filepath
            .file_name()
            .ok_or_else(|| eyre::eyre!("unable to find file name of `{filepath:?}`"))?
            .to_string_lossy();

        let mut res_store = ResourceCache::new();
        let root_node =
            node::deserialize_maybe_compressed(contents, &mut res_store, file_name.into_owned())?;

        // tracing::debug!("{root_node:#?}");

        Ok(Self {
            root_node,
            res_store,
            filepath,
        })
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
                            if ui.button("Save").clicked() {
                                todo!()
                            }

                            if ui.button("Save as").clicked() {
                                todo!()
                            }

                            if ui.button("Close").clicked() {
                                self.current_page = CurrentPage::Intro;
                            }

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

    pub fn draw_editor(&mut self, ui: &mut egui::Ui) {
        self.draw_upper_toolbar(ui);

        // Check whether we have switched back to the home menu
        if !self.current_page.is_editor() {
            return;
        }

        // Draw file explorer
        let panel_id = egui::Id::new("file_tree_panel");
        egui::Panel::left(panel_id).min_size(500.0).show(ui, |ui| {
            let root = &self.current_page.as_editor().unwrap().root_node;
            self.draw_file_tree(root, ui);
        });

        self.draw_property_window(ui);
        self.draw_editor_view(ui);
    }

    /// Draws the file tree under the current node.
    ///
    /// Lazy nodes are automatically evaluated once their folder is opened.
    fn draw_file_tree(&self, base: &VirtualNode, ui: &mut egui::Ui) {
        if base.kind == VirtualNodeKind::Container {
            egui::CollapsingHeader::new(&base.label).show(ui, |ui| {
                for child in &base.children {
                    self.draw_file_tree(child, ui);
                }
            });
        } else {
            if ui.button(&base.label).clicked() {
                // Open the inspector window for this file's content
                tracing::trace!("should open: {}", base.label);

                // Check whether the file has already been lazily loaded.
            }
        }
    }
}
