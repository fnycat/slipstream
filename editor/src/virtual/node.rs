use crate::r#virtual::refs::VirtualNodeId;
use crate::r#virtual::defer::Deferred;
use std::{
    cell::RefCell

    ,
    rc::Rc,
};
use crate::error::EditorResult;

pub type VirtualNodeRef = Rc<RefCell<VirtualNode>>;

impl From<VirtualNode> for VirtualNodeRef {
    fn from(value: VirtualNode) -> Self {
        Rc::new(RefCell::new(value))
    }
}

pub trait Inspectable: std::fmt::Debug {
    fn draw_properties(&mut self, ui: &mut egui::Ui);
}

#[derive(Debug)]
pub struct VirtualNodeContent {
    pub inspectable: Option<Box<dyn Inspectable>>,
    pub children: Vec<VirtualNodeRef>,
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
    pub content: Deferred<VirtualNodeContent>,
}

impl VirtualNode {
    pub fn evaluate(&mut self) -> EditorResult<()> {
        self.content.evaluate()?;
        Ok(())
    }

    pub fn is_deferred(&self) -> bool {
        self.content.is_deferred()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VirtualNodeKind {
    /// This virtual node can contain other nodes.
    ///
    /// This is used for both directories and files that contain multiple subfiles/sections.
    Container,
    /// This is the final node in this branch.
    ///
    /// This is used for files that are not split up any further.
    Terminal,
}
