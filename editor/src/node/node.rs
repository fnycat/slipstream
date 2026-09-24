use std::any::Any;
use std::sync::{Arc, mpsc};

use parking_lot::Mutex;

use crate::error::EditorResult;
use crate::node::defer::Deferred;
use crate::node::refs::VirtualNodeId;
use crate::panes::{PaneAction, RequestNewPane, RequestPaneEdit};
use crate::shared::util::AssertSendSync;
use crate::{fill_icon, reg_icon};

pub trait Inspectable: Send + Sync + std::fmt::Debug {
    fn draw_properties(&mut self, ui: &mut egui::Ui);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug)]
pub struct VirtualNodeBody {
    pub children: Vec<VirtualNodeId>,
    pub inspectable: Option<Box<dyn Inspectable>>,
}

#[derive(Debug)]
pub struct VirtualNode {
    pub label: String,
    pub id: VirtualNodeId,
    /// Determines how this node is displayed in the file tree.
    ///
    /// If the node kind is [`Container`], it will be displayed as a directory.
    ///
    /// [`Container`](VirtualNodeKind::Container)
    pub kind: VirtualNodeKind,
    pub parent: Option<VirtualNodeId>,
    pub body: Deferred<VirtualNodeBody>,
}

impl AssertSendSync for VirtualNode {}

impl VirtualNode {
    /// Renders the context menu of this node.
    pub fn draw_context_menu(&self, cmd: &mut mpsc::Sender<PaneAction>, ui: &mut egui::Ui) {
        if ui.button("Export").clicked() {
            todo!("export");
        }

        if ui.button("Rename").clicked() {
            todo!("rename");
        }

        if ui.button("Inspect").clicked() {
            cmd.send(PaneAction::RequestNewPane(RequestNewPane::Inspector {
                inspected: self.id,
            }));
        }

        if self.kind == VirtualNodeKind::Mdl0Root {
            if ui.button("Open in 3D viewer").clicked() {
                cmd.send(PaneAction::RequestNewPane(RequestNewPane::Viewer {
                    viewed: Some(self.id),
                }));
            }
        }

        if ui.button("Open in new Outliner").clicked() {
            cmd.send(PaneAction::RequestNewPane(RequestNewPane::Outliner {
                root: self.id,
            }));
        }
    }

    pub fn evaluate(&mut self) -> EditorResult<()> {
        self.body.evaluate()?;
        Ok(())
    }

    pub fn is_deferred(&self) -> bool {
        self.body.is_deferred()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VirtualNodeKind {
    /// This virtual node can contain other nodes.
    ///
    /// This is used for both directories and files that contain multiple subfiles/sections.
    ArcDirectory {
        empty: bool,
    },
    BrresDirectory,
    Mdl0Root,
    Bytecode,
    Bone {
        end: bool,
    },
    Vertices,
    Normals,
    Colors,
    Uvs,
    Materials,
    Tevs,
    Polygon,
    TextureLinks,
    PaletteLinks,
    Unknown,
}

impl VirtualNodeKind {
    /// Whether this node is expandable.
    ///
    /// This determines whether this node will have a collapsible header.
    pub fn is_expandable(&self) -> bool {
        match self {
            Self::ArcDirectory { empty: true }
            | Self::Bytecode
            | Self::Bone { end: true }
            | Self::Vertices
            | Self::Normals
            | Self::Colors
            | Self::Uvs
            | Self::Materials
            | Self::Tevs
            | Self::Polygon
            | Self::TextureLinks
            | Self::PaletteLinks
            | Self::Unknown => false,
            _ => true,
        }
    }

    /// The icon to use when the folder/file is open.
    pub fn icon_open(&self) -> egui::RichText {
        match self {
            Self::ArcDirectory { empty: false } | Self::BrresDirectory => reg_icon!(FOLDER_OPEN),
            Self::ArcDirectory { empty: true } => reg_icon!(FOLDER_DASHED),
            Self::Mdl0Root => reg_icon!(PERSON),
            Self::Bytecode => reg_icon!(FILE_CODE),
            Self::Bone { end: false } => reg_icon!(BONE),
            Self::Bone { end: true } => fill_icon!(BONE),
            Self::Vertices => reg_icon!(POLYGON),
            Self::Normals => reg_icon!(ARROW_ELBOW_RIGHT),
            Self::Colors => reg_icon!(PAINT_BRUSH_HOUSEHOLD),
            Self::Uvs => reg_icon!(BOUNDING_BOX),
            Self::Materials => reg_icon!(PALETTE),
            Self::Tevs => reg_icon!(GRAPHICS_CARD),
            Self::Polygon => reg_icon!(CUBE),
            Self::TextureLinks => reg_icon!(LINK),
            Self::PaletteLinks => reg_icon!(LINK),
            Self::Unknown => reg_icon!(FILE),
        }
    }

    /// The icon to use when the folder/file is closed.
    pub fn icon_closed(&self) -> egui::RichText {
        match self {
            Self::ArcDirectory { empty: false } | Self::BrresDirectory => reg_icon!(FOLDER),
            _ => self.icon_open(),
        }
    }
}
