use crate::error::EditorResult;
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::refs::VirtualNodeId;
use std::cell::Ref;
use std::rc::Weak;
use std::{cell::RefCell, rc::Rc};

pub trait Inspectable: std::fmt::Debug {
    fn draw_properties(&mut self, ui: &mut egui::Ui);
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

impl VirtualNode {
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
    Directory,
    DirectoryEmpty,
    Bone,
    BoneFinal,
    Vertices,
    Normals,
    Unknown,
}

impl VirtualNodeKind {
    pub fn is_directory(&self) -> bool {
        match self {
            Self::BoneFinal | Self::Unknown => false,
            _ => true,
        }
    }

    /// The icon to use when the folder/file is open.
    pub fn icon_open(&self) -> egui::RichText {
        match self {
            Self::Directory => reg_icon!(FOLDER_OPEN),
            Self::DirectoryEmpty => reg_icon!(FOLDER_DASHED),
            Self::Bone => reg_icon!(BONE),
            Self::BoneFinal => fill_icon!(BONE),
            Self::Vertices => reg_icon!(POLYGON),
            Self::Normals => reg_icon!(ARROW_ELBOW_RIGHT),
            Self::Unknown => reg_icon!(FILE),
        }
    }

    /// The icon to use when the folder/file is closed.
    pub fn icon_closed(&self) -> egui::RichText {
        match self {
            Self::Directory => reg_icon!(FOLDER),
            Self::DirectoryEmpty => reg_icon!(FOLDER_DASHED),
            Self::Bone => reg_icon!(BONE),
            Self::BoneFinal => fill_icon!(BONE),
            Self::Vertices => reg_icon!(POLYGON),
            Self::Normals => reg_icon!(ARROW_ELBOW_RIGHT),
            Self::Unknown => reg_icon!(FILE),
        }
    }
}
