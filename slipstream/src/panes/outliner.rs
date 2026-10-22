use std::sync::{Arc, mpsc};

use slipstream_ir::brres::MdlSectionType;
use slipstream_ir::kmp::KmpSectionType;
use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::node::node::{IrNode, IrNodeType};
use slipstream_shared::error::SlipstreamResult;
use slipstream_shared::reg_icon;

use crate::panes::{Pane, PaneAction, PaneId, PaneResponse, RequestNewPane};
use slipstream_shared::widgets::{self, CollapseDescriptor, DropdownType, HeaderIcons};

/// The outliner displays a file tree.
///
/// It only needs a root node and to start from and will explore and draw the rest
/// of the file tree by itself.
///
/// New suboutliners can be made by creating new panes with a child node set to root.
pub struct OutlinerPane {
    cmd_sender: mpsc::Sender<PaneAction>,
    base_node: Option<IrNodeKey>,
    arena: Arc<IrArena>
}

impl OutlinerPane {
    /// Creates a new outliner pane.
    ///
    /// `root_key` is the node that will be the root of the outliner. This makes it possible
    /// to create outliners of subsets of the project.
    pub fn new(
        cmd_sender: mpsc::Sender<PaneAction>,
        base_node: Option<IrNodeKey>,
        arena: Arc<IrArena>
    ) -> Box<dyn Pane> {
        Box::new(Self {
            cmd_sender,
            base_node,
            arena
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

        // Only draw the open in 3D viewer button for MDL roots.
        if node.ty == IrNodeType::MdlHeader {
            if ui.button("Open in 3D viewer").clicked() {
                self.cmd_sender
                    .send(PaneAction::RequestNewPane(RequestNewPane::Viewer {
                        viewed: Some(node.key())
                    }))
                    .expect("failed to send context command");
            }
        }

        if ui.button("Open in new outliner").clicked() {
            self.cmd_sender
                .send(PaneAction::RequestNewPane(RequestNewPane::Outliner {
                    base: node.key()
                }))
                .expect("failed to send context command");
        }

        if ui.button("Properties").clicked() {
            self.cmd_sender
                .send(PaneAction::RequestNewPane(RequestNewPane::Inspector {
                    inspected: node.key()
                }))
                .expect("failed to send context command");
        }
    }

    fn draw_directory_node(&self, node: &IrNode, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        widgets::draw_collapsing_state(
            CollapseDescriptor::new(
                Self::get_state_id(node.key(), ui),
                node.label.clone().into(),
                Some(HeaderIcons {
                    open: node.ty.open_icon(),
                    closed: node.ty.closed_icon()
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
                Some(&mut |ui| self.draw_context_menu(node, ui))
            ),
            ui
        )
    }

    /// Draws a file in the outliner.
    ///
    /// This file will have no more subnodes.
    fn draw_leaf_node(&self, node: &IrNode, ui: &mut egui::Ui) -> SlipstreamResult<()> {
        ui.horizontal(|ui| {
            ui.label(node.ty.closed_icon());

            let response = ui.selectable_label(false, node.label());
            if response.clicked() {
                self.cmd_sender
                    .send(PaneAction::RequestNewPane(RequestNewPane::Inspector {
                        inspected: node.key()
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
        _is_focused: bool
    ) -> PaneResponse {
        ui.set_min_size(ui.available_size());

        ui.spacing_mut().item_spacing.y = 7.5;

        if let Some(base_node) = self.base_node {
            // If we have a base node, draw the file tree.
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                self.draw_file_tree(base_node, ui).unwrap();
            });
        } else {
            // Otherwise offer to open a file.
            ui.label("No files are open");
        }

        PaneResponse::None
    }
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
            _ => self.closed_icon()
        }
    }

    fn closed_icon(&self) -> egui::RichText {
        match self {
            Self::ArcDirectory { empty: false, .. } | Self::Nw4rDirectory => reg_icon!(FOLDER),
            Self::ArcDirectory { empty: true, .. } => reg_icon!(FOLDER_DASHED),
            Self::BrresFile => reg_icon!(FOLDER),

            Self::MdlHeader => reg_icon!(PERSON_SIMPLE),
            Self::MdlSection { ty } => match ty {
                MdlSectionType::Definitions => reg_icon!(FILE_CODE),
                MdlSectionType::Bones => reg_icon!(BONE),
                MdlSectionType::Vertices => reg_icon!(POLYGON),
                MdlSectionType::Normals => reg_icon!(ARROW_ELBOW_RIGHT),
                MdlSectionType::Colors => reg_icon!(PAINT_BRUSH_HOUSEHOLD),
                MdlSectionType::UvCoordinates => reg_icon!(BOUNDING_BOX),
                MdlSectionType::Material => reg_icon!(PALETTE),
                MdlSectionType::Shader => reg_icon!(GRAPHICS_CARD),
                MdlSectionType::Polygon => reg_icon!(CUBE),
                MdlSectionType::FurLayers => reg_icon!(PAW_PRINT),
                MdlSectionType::FurVectors => reg_icon!(PAW_PRINT),
                MdlSectionType::TextureLinks => reg_icon!(LINK),
                MdlSectionType::PaletteLinks => reg_icon!(LINK),
                MdlSectionType::UserData => reg_icon!(FOLDER_USER)
            },

            Self::ChrHeader => reg_icon!(FILM_SLATE),
            Self::SkeletalAnimation => reg_icon!(BONE),

            Self::Tpl => reg_icon!(IMAGES),
            Self::TplImage => reg_icon!(IMAGE),

            Self::ClrHeader => reg_icon!(PALETTE),
            Self::ColorAnimation => reg_icon!(PALETTE),

            Self::Texture => reg_icon!(IMAGES),

            Self::KmpHeader => reg_icon!(PATH),
            Self::KmpSection { ty } => match ty {
                KmpSectionType::Areas => reg_icon!(SELECTION),
                KmpSectionType::Cameras => reg_icon!(VIDEO_CAMERA),
                KmpSectionType::CannonTargetPositions => reg_icon!(ROCKET_LAUNCH),
                KmpSectionType::Checkpoints => reg_icon!(SHIELD_CHECK),
                KmpSectionType::CheckpointGroups => reg_icon!(PATH),
                KmpSectionType::MissionSuccessPoints => reg_icon!(FLAG_CHECKERED),
                KmpSectionType::EnemyRoute => reg_icon!(PATH),
                KmpSectionType::EnemyRoutePoints => reg_icon!(MAP_PIN),
                KmpSectionType::GeoObjects => reg_icon!(CUBE),
                KmpSectionType::ItemRoute => reg_icon!(PATH),
                KmpSectionType::ItemRoutePoints => reg_icon!(MAP_PIN),
                KmpSectionType::RespawnPositions => reg_icon!(LIFEBUOY),
                KmpSectionType::Routes => reg_icon!(PATH),
                KmpSectionType::StageInformation => reg_icon!(INFO),
                KmpSectionType::StartingPositions => reg_icon!(FLAG_BANNER)
            },

            Self::Unknown => reg_icon!(FILE)
        }
    }
}
