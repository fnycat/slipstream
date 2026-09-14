use szslib::{arc, brres, chr0, mdl0, pat0};

use crate::pages::editor::FileData;

pub trait DrawFileTree {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str);
}

fn draw_arc_node(node: &arc::ArcNode, ui: &mut egui::Ui) {
    match node {
        arc::ArcNode::File { name, .. } => {
            ui.button(name);
        }
        arc::ArcNode::Directory { name, children } => {
            egui::CollapsingHeader::new(name).show(ui, |ui| {
                for child in children {
                    draw_arc_node(child, ui);
                }
            });
        }
    }
}

impl DrawFileTree for FileData {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        match self {
            FileData::Szs(v) => v.draw_tree(ui, name),
            FileData::Brres(v) => v.draw_tree(ui, name),
        }
    }
}

impl DrawFileTree for arc::Archive {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        // Skip nameless root node
        let Some(root) = &self.root else { return };
        root.draw_tree(ui, name);
    }
}

impl DrawFileTree for arc::FileType {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        match self {
            Self::Brres(archive) => archive.draw_tree(ui, name),
            Self::Raw(..) => {
                ui.button(format!("RAW: {name}"));
            }
        }
    }
}

impl DrawFileTree for arc::ArcNode {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        match self {
            arc::ArcNode::File { name, data } => {
                // Forward the drawing to the individual file.
                //
                // Some files might have further internal structure that the archive
                // does not know about.
                data.draw_tree(ui, name);
            }
            arc::ArcNode::Directory { name, children } => {
                egui::CollapsingHeader::new(name).show(ui, |ui| {
                    for child in children {
                        child.draw_tree(ui, child.name());
                    }
                });
            }
        }
    }
}

impl DrawFileTree for brres::Archive {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        egui::CollapsingHeader::new(name).show(ui, |ui| {
            for directory in &self.directories {
                directory.draw_tree(ui, &directory.name);
            }
        });
    }
}

impl DrawFileTree for brres::Directory {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        egui::CollapsingHeader::new(name).show(ui, |ui| {
            for subfile in &self.files {
                subfile.draw_tree(ui, &subfile.name);
            }
        });
    }
}

impl DrawFileTree for brres::File {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        // Some subfiles may also have further internal structure.

        match &self.file {
            brres::SubfileData::Root(_) => {}
            brres::SubfileData::Mdl0(mdl0) => mdl0.draw_tree(ui, name),
            brres::SubfileData::Chr0(chr0) => chr0.draw_tree(ui, name),
            brres::SubfileData::Pat0(pat0) => pat0.draw_tree(ui, name),
            _ => todo!(),
        }
    }
}

impl DrawFileTree for mdl0::Mdl0Subfile {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        egui::CollapsingHeader::new(name).show(ui, |ui| {
            // Create folders for all MDL0 subsections, if they exist.

            self.definitions
                .inspect(|defs| defs.draw_tree(ui, "Definitions"));

            self.bones.inspect(|bones| bones.draw_tree(ui, "Bones"));
        });
    }
}

impl DrawFileTree for mdl0::Definitions {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        ui.button("TODO: definitions");
    }
}

impl DrawFileTree for mdl0::Bones {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {}
}

impl DrawFileTree for chr0::Chr0Subfile {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        egui::CollapsingHeader::new(name).show(ui, |ui| {});
    }
}

impl DrawFileTree for pat0::Pat0Subfile {
    fn draw_tree(&self, ui: &mut egui::Ui, name: &str) {
        egui::CollapsingHeader::new(name).show(ui, |ui| {});
    }
}
