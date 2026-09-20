use std::sync::mpsc;
use std::{path::PathBuf, sync::Arc};

use eframe::egui_wgpu;

use crate::cmd::AppCommandChannel;
use crate::decorations::{self, WindowState};
use crate::error::{EditorError, EditorResult, InvalidInputError};
use crate::pages::RoutablePage;
use crate::pages::intro::IntroPage;
use crate::panes::outliner::OutlinerPane;
use crate::panes::viewer::ViewerPane;
use crate::panes::{Pane, PaneBehavior, TreeAction};
use crate::viewer::camera::CameraController;
use crate::viewer::{self, TEXTURE_FILTER_MODE, ViewerCallback};
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::refs::{VirtualNodeId, VirtualNodeMap, VirtualRefCacheMap};
use crate::r#virtual::root::{self};
use crate::{shared::util::RefCursor, viewer::ViewerState};

pub struct Properties {
    pub label: String,
    pub node_id: VirtualNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenedFileInfo {
    Native {
        path: PathBuf,
        file_name: String,
        content: Vec<u8>,
    },
    Web {
        file_name: String,
        content: Vec<u8>,
    },
}

impl OpenedFileInfo {
    pub fn content(&self) -> &[u8] {
        match self {
            Self::Native { content, .. } => content,
            Self::Web { content, .. } => content,
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            Self::Native { file_name, .. } => file_name,
            Self::Web { file_name, .. } => file_name,
        }
    }
}

/// Data specific to the editor page.
pub struct Editor {
    pub cmd: AppCommandChannel,
    pub render_state: viewer::RenderState,
    /// The path of the current file open in the editor.
    ///
    /// This is a regular filesystem path, pointing to the root file.
    /// Not an internal URI.
    pub file_info: OpenedFileInfo,
    /// The root node of the file.
    pub file_base_node: VirtualNodeId,
    pub node_map: VirtualNodeMap,

    pub pane_behavior: PaneBehavior,
    pub pane_tree: egui_tiles::Tree<Box<dyn Pane>>,
}

impl Editor {
    pub fn new(
        file_info: OpenedFileInfo,
        cmd_channel: AppCommandChannel,
        render_state: viewer::RenderState,
    ) -> EditorResult<Box<dyn RoutablePage>> {
        let contents = file_info.content();
        let cursor = RefCursor::new(Arc::<[u8]>::from(contents));

        ViewerCallback::init(&render_state);

        let node_map = Arc::new(VirtualRefCacheMap::new());
        let root_node = root::deserialize_maybe_compressed(
            cursor,
            &node_map,
            file_info.file_name().to_owned(),
        )?;

        let mut tiles = egui_tiles::Tiles::default();

        let (tx, rx) = mpsc::channel();

        let grid = egui_tiles::Grid::new(Vec::new());
        let grid_id = tiles.insert_container(grid);

        let panes = [
            OutlinerPane::new(tx.clone(), grid_id, root_node, Arc::clone(&node_map)),
            ViewerPane::new(),
        ]
        .into_iter()
        .map(|pane| tiles.insert_pane(pane))
        .collect::<Vec<_>>();

        let egui_tiles::Tile::Container(grid) = tiles.get_mut(grid_id).unwrap() else {
            unreachable!()
        };

        panes.iter().for_each(|&id| grid.add_child(id));

        let pane_behavior = PaneBehavior { cmd_receiver: rx };
        let pane_tree = egui_tiles::Tree::new(egui::Id::new("editor_pane_tree"), grid_id, tiles);

        Ok(Box::new(Self {
            cmd: cmd_channel,
            render_state: render_state.clone(),

            node_map,
            file_info,
            file_base_node: root_node,

            pane_behavior,
            pane_tree,
        }))
    }

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
                    let window_state = WindowState::get_state(ui);
                    if window_state == WindowState::Maximized {
                        ui.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                    } else {
                        ui.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                    }
                }

                if response.drag_started() {
                    // ui.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
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
                                self.cmd
                                    .try_route(IntroPage::new(
                                        self.cmd.clone(),
                                        self.render_state.clone(),
                                    ))
                                    .unwrap();
                            }

                            if ui.button("Quit").clicked() {
                                ui.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });

                        ui.menu_button("Settings", |_ui| {});
                    });

                    decorations::draw_title_buttons(ui);
                });
            });
    }

    fn draw_editor_view(&self, ui: &mut egui::Ui) {
        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let target_size = ui.available_size();
            let panel_bounds = egui::Rect::from_min_size(ui.cursor().min, target_size);

            // We must update the panel size before the callback.
            //
            // Updating the texture ID requires locking the renderer, but the
            // paint callback locks it too.
            //
            // All this nonsense down here is to avoid a deadlock with the paint callback.

            let texture_id = {
                let mut renderer = self.render_state.renderer.write();
                let viewer = renderer
                    .callback_resources
                    .get_mut::<ViewerState>()
                    .unwrap();
                let texture_id = viewer.texture_data.texture_id;

                let resized = viewer.resize_viewport(panel_bounds);
                if resized {
                    let device = viewer.device.clone();
                    let tex_view = viewer.texture_data.texture_view.clone();

                    renderer.update_egui_texture_from_wgpu_texture(
                        &device,
                        &tex_view,
                        TEXTURE_FILTER_MODE,
                        texture_id,
                    );
                }

                texture_id
            };

            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                panel_bounds,
                ViewerCallback,
            ));

            let image_widget = egui::Image::new(egui::load::SizedTexture {
                id: texture_id,
                size: panel_bounds.size(),
            })
            .sense(egui::Sense::drag());

            let response = ui.add(image_widget);
            if response.dragged() {
                let mut renderer = self.render_state.renderer.write();
                let viewer = renderer
                    .callback_resources
                    .get_mut::<ViewerState>()
                    .unwrap();

                let delta = response.drag_delta();

                viewer
                    .camera
                    .as_orbit_mut()
                    .drag_delta(glam::vec2(delta.x, delta.y));

                viewer.on_camera_update();
            }

            ui.input(|i| {
                if i.is_scrolling() && response.contains_pointer() {
                    let mut renderer = self.render_state.renderer.write();
                    let viewer = renderer
                        .callback_resources
                        .get_mut::<ViewerState>()
                        .unwrap();

                    let delta = i.smooth_scroll_delta();
                    viewer.camera.scroll_delta(glam::vec2(delta.x, delta.y));
                    viewer.on_camera_update();
                }
            });
        });
    }
}

impl RoutablePage for Editor {
    fn name(&self) -> &str {
        "Editor"
    }

    fn update(&mut self) -> EditorResult<()> {
        while let Ok(cmd) = self.pane_behavior.cmd_receiver.try_recv() {
            match cmd {
                TreeAction::AddTile { parent, pane } => {
                    let new_id = self.pane_tree.tiles.insert_pane(pane);

                    let Some(parent) = self.pane_tree.tiles.get_mut(parent) else {
                        todo!();
                    };

                    let egui_tiles::Tile::Container(container) = parent else {
                        todo!();
                    };

                    container.add_child(new_id);
                }
                TreeAction::RemoveTile(tile) => {
                    self.pane_tree.tiles.remove(tile);
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ui: &mut egui::Ui) -> EditorResult<()> {
        self.draw_upper_toolbar(ui);
        self.pane_tree.ui(&mut self.pane_behavior, ui);

        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        ViewerCallback::deinit(&self.render_state.renderer);
    }
}
