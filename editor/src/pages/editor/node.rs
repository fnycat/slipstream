use std::{collections::HashMap, rc::Rc};

use szslib::{arc, brres};

use crate::shared::uri::UriSlice;

pub trait VirtualNode: std::fmt::Debug {
    fn label(&self) -> &str;
    fn is_directory(&self) -> bool;
    fn children(&self) -> &[Rc<dyn VirtualNode>];
}

/// Draws the file tree of the given virtual node.
pub fn draw_virtual_node_tree(base: &dyn VirtualNode, ui: &mut egui::Ui) {
    if base.is_directory() {
        egui::CollapsingHeader::new(base.label()).show(ui, |ui| {
            for child in base.children() {
                draw_virtual_node_tree(child.as_ref(), ui);
            }
        });
    } else {
        ui.button(base.label());
    }
}
