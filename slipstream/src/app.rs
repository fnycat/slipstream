use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};

use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::encoding::serialize_node;
use slipstream_shared::SlipstreamResult;
use slipstream_shared::cursor::MutCursor;
use slipstream_shared::error::AssertFailed;

use crate::inspector::InspectorPane;
use crate::panes::animator::AnimatorPane;
use crate::panes::log::LogPane;
use crate::panes::outliner::OutlinerPane;
use crate::panes::{Pane, PaneAction, PaneBehavior, RequestNewPane};
use crate::shared::GraphicsState;
use crate::viewer::ViewerPane;

const CHANNEL_SIZE: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenedFileInfo {
    Native {
        path: PathBuf,
        file_name: String,
        content: Vec<u8>
    },
    Web {
        file_name: String,
        content: Vec<u8>
    }
}

impl OpenedFileInfo {
    pub fn content(&self) -> &[u8] {
        match self {
            Self::Native { content, .. } => content,
            Self::Web { content, .. } => content
        }
    }

    pub fn file_name(&self) -> &str {
        match self {
            Self::Native { file_name, .. } => file_name,
            Self::Web { file_name, .. } => file_name
        }
    }
}

/// Generates the initial window layout at startup.
fn create_initial_layout(
    arena: &Arc<IrArena>,
    graphics_state: GraphicsState
) -> SlipstreamResult<(PaneBehavior, egui_tiles::Tree<Box<dyn Pane>>)> {
    let mut tiles = egui_tiles::Tiles::default();

    let (tx, rx) = mpsc::channel();

    let outliner = OutlinerPane::new(tx.clone(), None, Arc::clone(&arena));
    let viewer = ViewerPane::new(tx.clone(), None, Arc::clone(&arena), graphics_state)?;
    let animator = AnimatorPane::new(tx.clone(), Arc::clone(&arena));

    let outliner = tiles.insert_pane(outliner);
    let viewer = tiles.insert_pane(viewer);
    let animator = tiles.insert_pane(animator);

    let horizontal_split =
        egui_tiles::Linear::new_binary(egui_tiles::LinearDir::Horizontal, [outliner, viewer], 0.2);
    let horizontal_id = tiles.insert_container(horizontal_split);

    let vertical_split = egui_tiles::Linear::new_binary(
        egui_tiles::LinearDir::Vertical,
        [horizontal_id, animator],
        0.7
    );

    let container_id = tiles.insert_container(vertical_split);

    let pane_behavior = PaneBehavior {
        receiver: rx,
        sender: tx,
        focused_pane: None
    };

    let pane_tree = egui_tiles::Tree::new(egui::Id::new("editor_pane_tree"), container_id, tiles);

    Ok((pane_behavior, pane_tree))
}

/// The `App` handles the basic state of the application and processes input events.
pub struct App {
    /// Whenever a panic occurs in the UI code, its metadata is stored in this option.
    /// When the user closes the panic window, the state is set back to `None`.
    pub panic_info: Option<Box<dyn Any + Send>>,
    /// The state of the renderer. This is purely here to pass it onto pages.
    pub render_state: GraphicsState,
    /// The UI state context.
    pub ctx: egui::Context,
    /// The path of the current file open in the editor.
    ///
    /// This is a regular filesystem path, pointing to the root file.
    /// Not an internal URI.
    pub file_info: Option<OpenedFileInfo>,
    /// Arena that holds all file nodes.
    pub arena: Arc<IrArena>,

    /// Controls `egui_tiles` behavior
    pub pane_behavior: PaneBehavior,
    /// Contains the editor panes.
    pub pane_tree: egui_tiles::Tree<Box<dyn Pane>>
}

impl App {
    /// Creates a new app using the given `eframe` creation context.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        tracing::info!("Initializing app...");

        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        egui_phosphor::add_font_bytes_as_family(
            &mut fonts,
            "slipstream_shared::icons::icons::fill",
            egui_phosphor::bytes::fill::FONT
        );

        cc.egui_ctx.set_fonts(fonts);

        let egui_rs = cc.wgpu_render_state.as_ref().unwrap();
        tracing::info!(
            "Using graphics API `{}`",
            egui_rs.adapter.get_info().backend
        );

        let graphics_state = GraphicsState {
            instance: egui_rs.instance.clone(),
            adapter: egui_rs.adapter.clone(),
            device: egui_rs.device.clone(),
            queue: egui_rs.queue.clone(),
            renderer: Arc::clone(&egui_rs.renderer)
        };

        let arena = Arc::new(IrArena::new());
        let (pane_behavior, pane_tree) = create_initial_layout(&arena, graphics_state.clone())
            .expect("failed to create initial app layout");

        Self {
            arena,
            render_state: graphics_state,
            panic_info: None,
            ctx: cc.egui_ctx.clone(),

            file_info: None,
            pane_behavior,
            pane_tree
        }
    }

    /// Draws the main app UI.
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        // We don't need resizing or the title bar on WASM platforms.
        if let Err(err) = self.draw_upper_toolbar(ui) {
            tracing::error!("while drawing upper toolbar: {err}");
        };

        self.pane_tree.ui(&mut self.pane_behavior, ui);

        // Draw panic modal if a panic occurred
        if self.panic_info.is_some() {
            self.draw_panic_modal(ui);
        }
    }

    pub fn get_active_pane(&self) -> Option<egui_tiles::TileId> {
        self.pane_behavior.focused_pane
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
        request: RequestNewPane
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
            RequestNewPane::Outliner { base } => OutlinerPane::new(
                self.pane_behavior.sender.clone(),
                Some(base),
                self.arena.clone()
            ),
            RequestNewPane::Inspector { inspected } => InspectorPane::new(
                self.pane_behavior.sender.clone(),
                inspected,
                self.arena.clone()
            ),
            RequestNewPane::Viewer { viewed } => ViewerPane::new(
                self.pane_behavior.sender.clone(),
                viewed,
                self.arena.clone(),
                self.render_state.clone()
            )?,
            RequestNewPane::Log => LogPane::new(self.pane_behavior.sender.clone()),
            RequestNewPane::Debug { ty } => Box::new(ty)
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
                }
            }

            new_pane_id
        })
    }

    /// Puts the window at the center of the screen.
    ///
    /// This will only work if the window size is up to date, i.e. the next frame after resizing
    /// the window. Centering before the size is updated will center based on the old size and misalign the window.
    fn center_window(&mut self) {
        let window_rect = self.ctx.viewport_rect();
        let sizex = window_rect.max.x - window_rect.min.x;
        let sizey = window_rect.max.y - window_rect.min.y;

        if let Some(monitor_size) = self.ctx.input(|i| i.viewport().monitor_size) {
            let monitor_pos = egui::pos2(0.0, 0.0);

            let center_x = monitor_pos.x + (monitor_size.x - sizex) / 2.0;
            let center_y = monitor_pos.y + (monitor_size.y - sizey) / 2.0;

            self.ctx
                .send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    center_x, center_y
                )));
        }
    }

    pub fn save_file(&self, path: &Path) -> SlipstreamResult<()> {
        let mut writer = MutCursor::new();
        self.arena
            .inspect(IrNodeKey::ROOT, |node| {
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

    pub fn handle_commands(&mut self) -> SlipstreamResult<()> {
        while let Ok(cmd) = self.pane_behavior.receiver.try_recv() {
            match cmd {
                PaneAction::RequestPaneEdit(_) => todo!(),
                PaneAction::RequestNewPane(request) => {
                    self.on_new_pane_request(request)?;
                }
                PaneAction::RemoveTile(tile) => {
                    self.pane_tree.tiles.remove(tile);
                }
            }
        }

        Ok(())
    }

    /// Draws a modal displaying information of a panic that was caught.
    ///
    /// When the user closes the window, the app's panic info is reset to `None`.
    pub fn draw_panic_modal(&mut self, ui: &mut egui::Ui) {
        let modal_id = egui::Id::new("panic_modal");
        egui::Modal::new(modal_id).show(ui.ctx(), |ui| {
            let panic = self.panic_info.as_ref().unwrap();

            ui.heading("Panic caught:");

            let text = if let Some(&msg) = panic.downcast_ref::<&'static str>() {
                msg
            } else if let Some(msg) = panic.downcast_ref::<String>() {
                msg
            } else {
                "Unknown"
            };

            let code = egui::RichText::new(text).code().monospace();
            ui.label(code);

            if ui.button("Acknowledge").clicked() {
                self.panic_info = None;
            }
        });
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process channel commands
        self.handle_commands();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Err(err) = std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.draw_ui(ui);
        })) {
            self.panic_info = Some(err);
        }
        // self.draw_ui(ui);
    }
}
