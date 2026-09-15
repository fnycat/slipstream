use std::{collections::HashMap, num::NonZeroUsize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(NonZeroUsize);

pub struct ResourceStore {
    next_id: usize,
    cache: HashMap<ResourceId, Vec<u8>>,
}

impl ResourceStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            cache: HashMap::new(),
        }
    }

    pub fn insert(&mut self, raw: Vec<u8>) -> ResourceId {
        let id = self.next_id();
        self.cache.insert(id, raw);
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
    pub children: Vec<VirtualNode>,
}

impl VirtualNode {
    pub fn draw_node_tree(&self, ui: &mut egui::Ui) {
        if !self.children.is_empty() {
            egui::CollapsingHeader::new(&self.label).show(ui, |ui| {
                for child in &self.children {
                    child.draw_node_tree(ui);
                }
            });
        } else {
            ui.button(&self.label);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VirtualNodeKind {
    Directory,
    File { cache_id: Option<ResourceId> },
}
