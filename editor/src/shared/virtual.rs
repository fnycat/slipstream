use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId(usize);

impl ResourceId {
    pub const ZERO: Self = Self(0);
}

pub struct ResourceCache {
    next_id: usize,
    cache: HashMap<ResourceId, ()>,
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
    File {
        format_tag: &'static str,
        cache_id: ResourceId,
    },
}
