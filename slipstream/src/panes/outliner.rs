use std::sync::{Arc, mpsc};

use slipstream_ir::node::{
    arena::{IrArena, IrNodeKey},
    node::{IrNode, IrNodeType},
};
use slipstream_shared::{
    error::{SlipstreamError, SlipstreamResult},
    fill_icon, reg_icon,
};

use crate::panes::{ContentSignature, Pane, PaneAction, PaneId, RequestNewPane};
use slipstream_shared::widgets::{self, CollapseDescriptor, DropdownType, HeaderIcons};

/// The outliner displays a file tree.
///
/// It only needs a root node and to start from and will explore and draw the rest
/// of the file tree by itself.
///
/// New suboutliners can be made by creating new panes with a child node set to root.
pub struct OutlinerPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    root: IrNodeKey,
    arena: Arc<IrArena>,
}

impl OutlinerPane {
    /// Creates a new outliner pane.
    ///
    /// `root_key` is the node that will be the root of the outliner. This makes it possible
    /// to create outliners of subsets of the project.
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        root_key: IrNodeKey,
        arena: Arc<IrArena>,
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            root: root_key,
            arena,
        })
    }

    /// Generates a persistent ID for the collapsible state of the given node.
    ///
    /// Top 32 bits are the pane ID, bottom 32 bits are the node ID.
    fn get_state_id(node_key: IrNodeKey, ui: &mut egui::Ui) -> egui::Id {
        let salt = ((PaneId::Outliner as u64) << 32) | u64::from(node_key);
        ui.make_persistent_id(salt)
    }

    fn draw_context_menu(&self, node: &IrNode, ui: &mut egui::Ui) {
        if ui.button("Rename").clicked() {
            todo!("rename");
        }

        // Only draw the open in 3D viewer button for MDL0 roots.
        if node.ty == IrNodeType::Mdl0Root {
            if ui.button("Open in 3D viewer").clicked() {
                self.cmd_sender
                    .send(PaneAction::RequestNewPane(RequestNewPane::Viewer {
                        viewed: Some(node.key()),
                    }))
                    .expect("failed to send context command");
            }
        }

        if ui.button("Open in new outliner").clicked() {
            self.cmd_sender
                .send(PaneAction::RequestNewPane(RequestNewPane::Outliner {
                    root: node.key(),
                }))
                .expect("failed to send context command");
        }

        if ui.button("Properties").clicked() {
            self.cmd_sender
                .send(PaneAction::RequestNewPane(RequestNewPane::Inspector {
                    inspected: node.key(),
                }))
                .expect("failed to send context command");
        }
    }

    fn draw_directory_node(&self, node: &IrNode, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        widgets::draw_collapsing_state(
            CollapseDescriptor::with_context_menu(
                Self::get_state_id(node.key(), ui),
                node.label.clone().into(),
                Some(HeaderIcons {
                    open: node.ty.open_icon(),
                    closed: node.ty.closed_icon(),
                }),
                DropdownType::Regular,
                false,
                true,
                |ui| {
                    // Render the children of this node.
                    for &child in node.children_keys() {
                        // Then start the whole file tree process over again, but for this sub node.
                        // `ui` needs to be reborrowed to ensure it does not move.
                        self.draw_file_tree(child, &mut *ui)?;
                    }

                    Ok(())
                },
                |ui| self.draw_context_menu(node, ui),
            ),
            ui,
        )
    }

    /// Draws a file in the outliner.
    ///
    /// This file will have no more subnodes.
    fn draw_leaf_node(&self, node: &IrNode, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        ui.horizontal(|ui| {
            ui.label(node.ty.closed_icon());

            let response = ui.button(node.label());
            if response.clicked() {
                self.cmd_sender
                    .send(PaneAction::RequestNewPane(RequestNewPane::Inspector {
                        inspected: node.key(),
                    }))
                    .expect("failed to send inspector pane open request");
            }

            response.context_menu(|ui| self.draw_context_menu(node, ui));
        });

        Ok(())
    }

    /// Draws the file tree under the current node.
    ///
    /// Lazy nodes are automatically evaluated once their folder is opened.
    ///
    /// If a specific node has been opened, this function returns the ID of its cache entry.
    fn draw_file_tree(&self, root_node: IrNodeKey, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        self.arena
            .inspect(root_node, |curr_node| {
                if curr_node.children.is_empty() {
                    self.draw_leaf_node(curr_node, ui)
                } else {
                    self.draw_directory_node(curr_node, ui)
                }
            })
            .transpose()?;

        Ok(())
    }
}

impl Pane for OutlinerPane {
    fn ty(&self) -> PaneId {
        PaneId::Outliner
    }

    fn title(&self) -> &str {
        "Explorer"
    }

    fn draw_content(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
    ) -> egui_tiles::UiResponse {
        ui.set_min_size(ui.available_size());

        ui.spacing_mut().item_spacing.y = 7.5;

        egui::ScrollArea::both().show(ui, |ui| {
            self.draw_file_tree(self.root, ui).unwrap();
        });

        egui_tiles::UiResponse::None
    }
}

/// Draws the icon of files and folders in the outliner.
fn draw_outliner_node_icon(
    ui: &mut egui::Ui,
    openness: f32,
    node_kind: IrNodeType,
    response: &egui::Response,
) {
    let icon = if openness < 0.5 {
        node_kind.closed_icon()
    } else {
        node_kind.open_icon()
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
}

/// Extends [`IrNodeType`], providing UI specific utilities to node types.
pub trait NodeIconsExt {
    /// The icon to display when this node is open.
    fn open_icon(&self) -> egui::RichText;
    /// The icon to display when this node is closed.
    fn closed_icon(&self) -> egui::RichText;
}

impl NodeIconsExt for IrNodeType {
    fn open_icon(&self) -> egui::RichText {
        match self {
            Self::ArcDirectory { empty: false, .. } | Self::BrresFile | Self::Nw4rDirectory => {
                reg_icon!(FOLDER_OPEN)
            }
            // Just reuse the closed icon for everything else.
            _ => self.closed_icon(),
        }
    }

    fn closed_icon(&self) -> egui::RichText {
        match self {
            Self::ArcDirectory { empty: false, .. } | Self::Nw4rDirectory => reg_icon!(FOLDER),
            Self::ArcDirectory { empty: true, .. } => reg_icon!(FOLDER_DASHED),
            Self::BrresFile => reg_icon!(FOLDER),

            Self::Mdl0Root => reg_icon!(PERSON_SIMPLE),
            Self::Definitions => reg_icon!(FILE_CODE),
            Self::Bone { end: false } => reg_icon!(BONE),
            Self::Bone { end: true } => fill_icon!(BONE),
            Self::VertexBuffer => reg_icon!(POLYGON),
            Self::NormalBuffer => reg_icon!(ARROW_ELBOW_RIGHT),
            Self::ColorBuffer => reg_icon!(PAINT_BRUSH_HOUSEHOLD),
            Self::UvBuffer => reg_icon!(BOUNDING_BOX),
            Self::Material => reg_icon!(PALETTE),
            Self::Tevs => reg_icon!(GRAPHICS_CARD),
            Self::Polygon => reg_icon!(CUBE),
            Self::TextureLinks => reg_icon!(LINK),
            Self::PaletteLinks => reg_icon!(LINK),

            Self::Chr0Root => reg_icon!(FILM_SLATE),
            Self::SkeletalAnimation => reg_icon!(BONE),

            Self::Texture => reg_icon!(IMAGES),

            Self::Unknown => reg_icon!(FILE),
        }
    }
}
