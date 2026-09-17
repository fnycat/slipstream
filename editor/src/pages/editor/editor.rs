use std::{io::Cursor, path::PathBuf, rc::Rc, sync::Arc};

use eframe::egui_wgpu;
use egui_phosphor::regular::{CARET_DOWN, CARET_RIGHT, FOLDER, FOLDER_OPEN};

use crate::error::{EditorError, EditorResult, InvalidInputError};
use crate::r#virtual::refs::{VirtualNodeId, VirtualRefCache, VirtualRefCacheMap};
use crate::r#virtual::node::VirtualNodeRef;
use crate::{
    app::{App, CurrentPage},
    model_renderer::ModelRenderer,
    shared::util::RefCursor,
};
use crate::r#virtual::node::{VirtualNode, VirtualNodeKind};
use crate::r#virtual::root::{self};

pub struct Properties {
    pub label: String,
    pub node_id: VirtualNodeId,
}

/// Data specific to the editor page.
pub struct Editor {
    /// The path of the current file open in the editor.
    ///
    /// This is a regular filesystem path, pointing to the root file.
    /// Not an internal URI.
    pub filepath: PathBuf,
    /// The whole file currently open in the editor.
    pub root_node: VirtualNodeRef,

    pub open_properties: Option<Properties>,
    pub ref_cache: VirtualRefCache,
}

impl Editor {
    pub fn new(filepath: PathBuf) -> EditorResult<Self> {
        let contents = std::fs::read(&filepath)?;
        let cursor = RefCursor::new(Rc::<[u8]>::from(contents));

        let file_name = filepath
            .file_name()
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("unable to find file name of `{filepath:?}`"),
                    ..Default::default()
                })
            })?
            .to_string_lossy();

        let mut ref_cache = VirtualRefCacheMap::new();
        let root_node =
            root::deserialize_maybe_compressed(cursor, &mut ref_cache, file_name.into_owned())?;

        Ok(Self {
            root_node,
            ref_cache,
            open_properties: None,
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
        egui::Panel::left(panel_id).show(ui, |ui| {
            let editor = self.current_page.as_editor_mut().unwrap();
            if let Some(properties) =
                Self::draw_file_tree(&editor.root_node, &editor.ref_cache, ui).unwrap()
            {
                editor.open_properties = Some(properties);
            }
        });

        self.draw_inspector_window(ui).unwrap();
        self.draw_animator_window(ui);
        self.draw_editor_view(ui);
    }

    /// Draws the file tree under the current node.
    ///
    /// Lazy nodes are automatically evaluated once their folder is opened.
    ///
    /// If a specific node has been opened, this function returns the ID of its cache entry.
    fn draw_file_tree(
        base: &VirtualNodeRef,
        ref_cache: &VirtualRefCache,
        ui: &mut egui::Ui,
    ) -> EditorResult<Option<Properties>> {
        ui.visuals_mut().collapsing_header_frame = true;

        // Has this node been evaluated?
        // If not, evaluate it
        base.borrow_mut().content.evaluate()?;
        let base = base.borrow();
        let base_content = base.content.get().expect("virtual node content not loaded");

        let mut opened_cache_id = None;
        if base.kind == VirtualNodeKind::Container {
            // egui::CollapsingHeader::new(&base.label)
            egui::CollapsingHeader::new(format!("{}: {}", base.id, base.label))
                .icon(|ui, openness, response| {
                    let icon = if openness < 0.5 { FOLDER } else { FOLDER_OPEN };

                    ui.painter().text(
                        response.rect.center(),
                        egui::Align2::CENTER_CENTER,
                        icon,
                        egui::FontId::default(),
                        ui.visuals().text_color(),
                    );
                })
                .show(ui, |ui| {
                    // Check if the current node has been evaluated.

                    for child in &base_content.children {
                        let ret = Self::draw_file_tree(child, ref_cache, ui).unwrap();
                        if ret.is_some() {
                            opened_cache_id = ret;
                        }
                    }
                });
        } else {
            if ui.button(format!("{}: {}", base.id, base.label)).clicked() {
            // if ui.button(&base.label).clicked() {
                // Open the inspector window for this file's content
                tracing::trace!("Opening file");

                return Ok(Some(Properties {
                    label: format!("Inspector ({})", base.label),
                    node_id: base.id,
                }));
            }
        }

        Ok(opened_cache_id)
    }
}
