use std::{any::Any, collections::HashMap, num::NonZeroUsize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(NonZeroUsize);

pub trait Inspectable {}

pub type LazyParser = fn(String, &mut ResourceCache, Vec<u8>) -> eyre::Result<Box<dyn Inspectable>>;

pub enum Lazy<T> {
    Deferred {
        parse_fn: LazyParser,
        bytes: Vec<u8>,
    },
    Parsed(T),
}

pub enum ResourceKind {
    Bones,
    Vertices,
    Normals,
}

pub struct ResourceCache {
    next_id: usize,
    cache: HashMap<ResourceId, Lazy<ResourceKind>>,
}

impl ResourceCache {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            cache: HashMap::new(),
        }
    }

    pub fn insert_deferred(&mut self, bytes: Vec<u8>, parse_fn: LazyParser) -> ResourceId {
        let id = self.next_id();
        self.cache.insert(id, Lazy::Deferred { bytes, parse_fn });
        id
    }

    pub fn next_id(&mut self) -> ResourceId {
        self.next_id += 1;
        ResourceId(NonZeroUsize::new(self.next_id - 1).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualNode {
    pub label: String,
    pub kind: VirtualNodeKind,
    pub content: Option<ResourceId>,
    pub children: Vec<VirtualNode>,
}

impl VirtualNode {
    /// Draws the file tree under the current node.
    ///
    /// Lazy nodes are automatically evaluated once their folder is opened.
    pub fn draw_node_tree(&self, ui: &mut egui::Ui) {
        if self.kind == VirtualNodeKind::Container {
            egui::CollapsingHeader::new(&self.label).show(ui, |ui| {
                for child in &self.children {
                    child.draw_node_tree(ui);
                }
            });
        } else {
            if ui.button(&self.label).clicked() {};
        }
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
