use std::cell::RefCell;
use std::{io::Cursor, path::PathBuf, rc::Rc, sync::Arc};

use eframe::egui_wgpu;

use crate::error::{EditorError, EditorResult, InvalidInputError};
use crate::pages::editor::inspector::widgets::DraggableNodePayload;
use crate::viewer::{OFFSCREEN_FILTER_MODE, ViewerCallback};
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::node::{VirtualNode, VirtualNodeKind};
use crate::r#virtual::refs::{
    VirtualNodeId, VirtualRefCache, VirtualRefCacheExt, VirtualRefCacheMap,
};
use crate::r#virtual::root::{self};
use crate::{
    app::{App, CurrentPage},
    shared::util::RefCursor,
    viewer::Viewer,
};

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
    pub root_node: VirtualNodeId,

    pub viewer: Viewer,
    pub open_node: Option<VirtualNodeId>,
    pub ref_cache: VirtualRefCache,
}

impl Editor {
    pub fn new(filepath: PathBuf, render_state: &egui_wgpu::RenderState) -> EditorResult<Self> {
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

        let ref_cache = Rc::new(RefCell::new(VirtualRefCacheMap::new()));
        let root_node =
            root::deserialize_maybe_compressed(cursor, &ref_cache, file_name.into_owned())?;

        Ok(Self {
            viewer: Viewer::new(render_state),
            root_node,
            ref_cache,
            open_node: None,
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
            let (panel_bounds, _response) =
                ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());

            // We must update the panel size before the callback.
            //
            // Updating the texture ID requires locking the renderer, but the
            // paint callback locks it too.
            //
            // All this nonsense down here is to avoid a deadlock with the paint callback.
            {
                let mut renderer = self.render_state.renderer.write();
                let viewer = renderer.callback_resources.get_mut::<Viewer>().unwrap();

                let resized = viewer.resize_render_texture(panel_bounds);
                if resized {
                    let device = viewer.device.clone();
                    let old_tex_id = viewer.texture_id;
                    let tex_view = viewer.texture_view.clone();

                    renderer.free_texture(&old_tex_id);

                    let new_tex_id =
                        renderer.register_native_texture(&device, &tex_view, OFFSCREEN_FILTER_MODE);

                    // As `viewer` borrows `renderer` mutably, we need to temporarily
                    // drop the `viewer` borrow to modify `renderer`.
                    renderer
                        .callback_resources
                        .get_mut::<Viewer>()
                        .unwrap()
                        .texture_id = new_tex_id;
                }
            }

            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                panel_bounds,
                ViewerCallback,
            ));
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
                Self::draw_file_tree(editor.root_node, &editor.ref_cache, ui).unwrap()
            {
                editor.open_node = Some(properties);
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
        base_id: VirtualNodeId,
        ref_cache: &VirtualRefCache,
        ui: &mut egui::Ui,
    ) -> EditorResult<Option<VirtualNodeId>> {
        ui.spacing_mut().item_spacing.y = 7.5;

        let base = ref_cache
            .get(base_id)
            .ok_or_else(|| {
                EditorError::from(InvalidInputError {
                    reason: format!("virtual node {base_id} does not exist"),
                    ..Default::default()
                })
            })?
            .clone();

        let base_ref = base.borrow();

        let mut opened_node_id = None;

        let node_kind = base_ref.kind;
        if node_kind.is_directory() {
            let response = egui::CollapsingHeader::new(format!("{}: {}", base_id, &base_ref.label))
                .id_salt(base_id) // Different folders might have the same name, use the unique node ID
                .icon(move |ui, openness, response| {
                    let icon = if openness < 0.5 {
                        node_kind.icon_closed()
                    } else {
                        node_kind.icon_open()
                    };

                    let galley = egui::WidgetText::from(icon).into_galley(
                        ui,
                        Some(egui::TextWrapMode::Extend),
                        f32::INFINITY,
                        egui::TextStyle::Body,
                    );

                    let center_pos = response.rect.center() - (galley.size() * 0.5);

                    ui.painter()
                        .galley(center_pos, galley, ui.visuals().text_color());
                })
                .show(ui, |ui| {
                    // Render children if this node has already been evaluated.
                    match &base_ref.body {
                        Deferred::Evaluated(body) => {
                            for &child in &body.children {
                                let ret = Self::draw_file_tree(child, ref_cache, ui).unwrap();
                                if ret.is_some() {
                                    opened_node_id = ret;
                                }
                            }

                            return;
                        }
                        _ => {}
                    }

                    drop(base_ref);

                    base.borrow_mut().body.evaluate().unwrap();

                    let base_ref = base.borrow();
                    let Deferred::Evaluated(body) = &base_ref.body else {
                        unreachable!()
                    };

                    for &child in &body.children {
                        let ret = Self::draw_file_tree(child, ref_cache, ui).unwrap();
                        if ret.is_some() {
                            opened_node_id = ret;
                        }
                    }
                });

            // Open the properties window of the folder when clicked.
            if response.header_response.double_clicked() {
                return Ok(Some(base_id));
            }
        } else {
            ui.horizontal(|ui| {
                ui.label(base_ref.kind.icon_closed());
                if ui.label(format!("{base_id}: {}", base_ref.label)).clicked() {
                    opened_node_id = Some(base_ref.id);
                }
            });

            // if response.
            // {
            //     return Ok(Some(base_id));
            // }
        }

        Ok(opened_node_id)
    }
}
