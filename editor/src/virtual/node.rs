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
    Container,
    /// This is the final node in this branch.
    ///
    /// This is used for files that are not split up any further.
    Terminal,
}
