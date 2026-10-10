use std::path::Path;
use std::sync::mpsc;
use std::{path::PathBuf, sync::Arc};

use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::encoding::serialize_node;
use slipstream_ir::node::root;
use slipstream_shared::SlipstreamError;
use slipstream_shared::cursor::{MutCursor, RefCursor};
use slipstream_shared::error::{AssertFailed, SlipstreamResult};

use crate::cmd::AppCommandChannel;
use crate::decorations::{self, WindowState};
use crate::inspector::InspectorPane;
use crate::pages::RoutablePage;
use crate::pages::intro::IntroPage;
use crate::panes::debug::DebugPane;
use crate::panes::log::LogPane;
use crate::panes::outliner::OutlinerPane;
use crate::panes::{Pane, PaneAction, PaneBehavior, RequestNewPane};
use crate::shared::GraphicsState;
use crate::viewer::ViewerPane;

pub struct Properties {
    pub label: String,
    pub node_id: IrNodeKey,
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
    pub render_state: GraphicsState,
    /// The path of the current file open in the editor.
    ///
    /// This is a regular filesystem path, pointing to the root file.
    /// Not an internal URI.
    pub file_info: OpenedFileInfo,
    /// The root node of the file.
    pub file_base_node: IrNodeKey,
    pub arena: Arc<IrArena>,

    pub pane_behavior: PaneBehavior,
    pub pane_tree: egui_tiles::Tree<Box<dyn Pane>>,
}

impl Editor {
    pub fn new(
        file_info: OpenedFileInfo,
        cmd_channel: AppCommandChannel,
        render_state: GraphicsState,
    ) -> SlipstreamResult<Box<dyn RoutablePage>> {
        let contents = file_info.content();
        let cursor = RefCursor::new(Arc::<[u8]>::from(contents));

        let arena = Arc::new(IrArena::new());
        let root_node =
            root::deserialize_maybe_compressed(cursor, &arena, file_info.file_name().to_owned())?;

        let mut tiles = egui_tiles::Tiles::default();

        let (tx, rx) = mpsc::channel();

        let outliner = OutlinerPane::new(tx.clone(), root_node, Arc::clone(&arena));
        let viewer = ViewerPane::new(tx.clone(), None, Arc::clone(&arena), render_state.clone())?;

        let tile_ids = [tiles.insert_pane(outliner), tiles.insert_pane(viewer)];

        let container =
            egui_tiles::Linear::new_binary(egui_tiles::LinearDir::Horizontal, tile_ids, 0.2);

        let container_id = tiles.insert_container(container);

        // let egui_tiles::Tile::Container(grid) = tiles.get_mut(container_id).unwrap() else {
        //     unreachable!()
        // };

        // panes.iter().for_each(|&id| grid.add_child(id));

        let pane_behavior = PaneBehavior {
            receiver: rx,
            sender: tx,
            focused_tile: None,
        };

        let pane_tree =
            egui_tiles::Tree::new(egui::Id::new("editor_pane_tree"), container_id, tiles);

        Ok(Box::new(Self {
            cmd: cmd_channel,
            render_state: render_state.clone(),

            arena,
            file_info,
            file_base_node: root_node,

            pane_behavior,
            pane_tree,
        }))
    }

    pub fn get_active_pane(&self) -> Option<egui_tiles::TileId> {
        self.pane_behavior.focused_tile
    }

    /// Returns the tile ID of the currently active container.
    ///
    /// This container will be the parent of the currently active pane.
    pub fn get_active_container(&mut self) -> Option<egui_tiles::TileId> {
        self.pane_tree.active_tiles().first().copied()
    }

    /// Handles a [`RequestNewPane`] request by either focusing an existing pane with
    /// the same contents, or creating a new pane.
    ///
    /// The existing panes are compared with the potential one by comparing [`ContentSignature`]s. If an
    /// equal signature is found, that window is focused instead of creating a new pane.
    ///
    /// If no existing pane was found, this function will first try to find the root node.
    /// In case this root node was found and happens to be an individual pane, the root will be turned into a horizontal container
    /// with both the new pane and old root pane contained in it. If this root was already a container, the new pane will simply be appended to it.
    /// If no tiles were found at all, this pane will be set as root.
    pub fn on_new_pane_request(
        &mut self,
        request: RequestNewPane,
    ) -> SlipstreamResult<egui_tiles::TileId> {
        // Check if this pane already exists.

        let existing_tile = self
            .pane_tree
            .tiles
            .iter()
            .find_map(|(id, pane)| {
                let egui_tiles::Tile::Pane(pane) = pane else {
                    return None;
                };

                (pane.ty() == request.id()).then_some(id)
            })
            .copied();

        // Pane was not found, create a new one
        let new_pane = match request {
            RequestNewPane::Outliner { root } => {
                OutlinerPane::new(self.pane_behavior.sender.clone(), root, self.arena.clone())
            }
            RequestNewPane::Inspector { inspected } => InspectorPane::new(
                self.pane_behavior.sender.clone(),
                inspected,
                self.arena.clone(),
            ),
            RequestNewPane::Viewer { viewed } => ViewerPane::new(
                self.pane_behavior.sender.clone(),
                viewed,
                self.arena.clone(),
                self.render_state.clone(),
            )?,
            RequestNewPane::Log => LogPane::new(self.pane_behavior.sender.clone()),
            RequestNewPane::Debug { ty } => Box::new(ty),
        };

        Ok(if let Some(existing_tile) = existing_tile {
            match self
                .pane_tree
                .tiles
                .get_mut(existing_tile)
                .expect("tile was removed during pane request")
            {
                egui_tiles::Tile::Pane(pane) => {
                    *pane = new_pane;
                }
                _ => {
                    return Err(AssertFailed {
                        reason: format!(
                            "tile {existing_tile:?} was a container, expected it to be a pane"
                        ),
                        ..Default::default()
                    }
                    .into());
                }
            }

            self.pane_tree
                .make_active(|tile_id, _tile| tile_id == existing_tile);

            existing_tile
        } else {
            let new_pane_id = self.pane_tree.tiles.insert_pane(new_pane);

            match self.pane_tree.root {
                None => {
                    // Tree is completely empty, just make the pane the root.
                    self.pane_tree.root = Some(new_pane_id);
                    tracing::trace!("Handled open pane request, setting it as root");
                }
                Some(root_id) => match self.pane_tree.tiles.get_mut(root_id) {
                    // Tree has some content
                    Some(egui_tiles::Tile::Container(container)) => {
                        // If the root is a container, just add to it.
                        container.add_child(new_pane_id);
                        tracing::trace!("Handled open pane request, adding it to root");
                    }
                    Some(egui_tiles::Tile::Pane(_)) => {
                        // If the root is a pane, we can't add another pane to it.
                        // Thus we create a container and add both panes as children.
                        // Then the container is set as root.

                        let new_root = self
                            .pane_tree
                            .tiles
                            .insert_horizontal_tile(vec![root_id, new_pane_id]);

                        self.pane_tree.root = Some(new_root);
                        tracing::trace!(
                            "Handled open pane request, creating a new root container and moving the panes into it"
                        );
                    }
                    None => {
                        tracing::error!(
                            "Tile root points to a non-existent tile, overriding root with new pane"
                        );

                        self.pane_tree.root = Some(new_pane_id);
                    }
                },
            }

            new_pane_id
        })
    }

    /// Draws the editor's upper toolbar.
    ///
    /// These are the `File`, `Edit`, buttons you often see in application .
    fn draw_upper_toolbar(&mut self, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        let decorations_id = egui::Id::new("title_panel");

        let egui::InnerResponse { inner, .. } = egui::Panel::top(decorations_id)
            .frame(
                egui::Frame::new()
                    .fill(ui.visuals().window_fill)
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
                    ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                let egui::InnerResponse { inner, .. } = ui.horizontal_centered(|ui| {
                    let egui::InnerResponse { inner, .. } = egui::MenuBar::new().ui(ui, |ui| {
                        let egui::InnerResponse { inner, .. } = ui.menu_button("File", |ui| {
                            if ui.button("Save").clicked() {
                                todo!("save file");
                            }

                            if ui.button("Save as").clicked() {
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    if let Some(path) = rfd::FileDialog::new().save_file() {
                                        self.save_file(&path)?;
                                    }
                                }

                                #[cfg(target_arch = "wasm32")]
                                {
                                    todo!("save as dialog on wasm32")
                                }
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

                            Ok::<_, SlipstreamError>(())
                        });

                        if let Some(inner) = inner {
                            inner?;
                        }

                        if ui.button("Logs").clicked() {
                            self.on_new_pane_request(RequestNewPane::Log)?;
                        }

                        let egui::InnerResponse { inner, .. } = ui.menu_button("Debug", |ui| {
                            if ui.button("Widget Inspector").clicked() {
                                self.on_new_pane_request(RequestNewPane::Debug {
                                    ty: DebugPane::Inspection,
                                })?;
                            }

                            if ui.button("GUI Settings").clicked() {
                                self.on_new_pane_request(RequestNewPane::Debug {
                                    ty: DebugPane::General,
                                })?;
                            }

                            if ui.button("Style Settings").clicked() {
                                self.on_new_pane_request(RequestNewPane::Debug {
                                    ty: DebugPane::Style,
                                })?;
                            }

                            if ui.button("Image Loader Statistics").clicked() {
                                self.on_new_pane_request(RequestNewPane::Debug {
                                    ty: DebugPane::Loaders,
                                })?;
                            }

                            if ui.button("Memory Statistics").clicked() {
                                self.on_new_pane_request(RequestNewPane::Debug {
                                    ty: DebugPane::Memory,
                                })?;
                            }

                            if ui.button("Texture Statistics").clicked() {
                                self.on_new_pane_request(RequestNewPane::Debug {
                                    ty: DebugPane::Textures,
                                })?;
                            }

                            Ok::<_, SlipstreamError>(())
                        });

                        if let Some(inner) = inner {
                            inner?;
                        }

                        Ok::<_, SlipstreamError>(())
                    });
                    inner?;

                    decorations::draw_title_buttons(ui);

                    Ok::<_, SlipstreamError>(())
                });

                inner
            });

        inner
    }

    fn save_file(&self, path: &Path) -> SlipstreamResult<()> {
        let mut writer = MutCursor::new();
        self.arena
            .inspect(self.file_base_node, |node| {
                serialize_node(&self.arena, node, &mut writer)
            })
            .transpose()?;

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::write(path, writer.into_inner())?;
        }

        #[cfg(target_arch = "wasm32")]
        {
            todo!("save file on wasm")
        }

        Ok(())
    }
}

impl RoutablePage for Editor {
    fn name(&self) -> &str {
        "Editor"
    }

    fn update(&mut self) -> SlipstreamResult<()> {
        while let Ok(cmd) = self.pane_behavior.receiver.try_recv() {
            match cmd {
                PaneAction::RequestPaneEdit(_) => todo!(),
                PaneAction::RequestNewPane(request) => {
                    self.on_new_pane_request(request).unwrap();
                }
                PaneAction::RemoveTile(tile) => {
                    self.pane_tree.tiles.remove(tile);
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        self.draw_upper_toolbar(ui)?;
        self.pane_tree.ui(&mut self.pane_behavior, ui);

        Ok(())
    }
}
