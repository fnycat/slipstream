use std::{io::Cursor, path::PathBuf};

use szslib::{
    arc::{self, ArcNode},
    brres,
    encoding::Deserialize,
    yaz0::{self, YAZ0_MAGIC},
};

use crate::shared::uri::UriSlice;

/// Figures out the format of the given file and tries to deserialize it.
pub fn deserialize_unknown(
    name: String,
    mut contents: Vec<u8>,
) -> eyre::Result<Box<dyn EditorNode>> {
    // Is this file compressed?
    if &contents[..4] == YAZ0_MAGIC {
        // then decompress it.
        contents = yaz0::decompress(&contents)?;
    }

    let mut cursor = Cursor::new(contents.as_slice());

    let magic: &[u8; 4] = contents[..4]
        .try_into()
        .expect("array of size 4 does not have size 4?");

    let contents = match magic {
        &arc::ARC_MAGIC => {
            let boxed = Box::new(arc::Archive::deserialize(&mut cursor, name)?);
            boxed as Box<dyn EditorNode>
        }
        &brres::BRRES_MAGIC => {
            let boxed = Box::new(brres::Archive::deserialize(&mut cursor, name)?);
            boxed as Box<dyn EditorNode>
        }
        _ => eyre::bail!("unknown file magic: `{}`", String::from_utf8_lossy(magic)),
    };

    Ok(contents)
}

/// Implemented by all types of files that can be opened in the editor.
pub trait EditorNode {
    fn name(&self) -> &str;

    /// Draws the file tree in this file (if it has one).
    fn draw_tree(&self, ui: &mut egui::Ui);

    fn uri(&self) -> UriSlice;

    // fn resolve_uri<'a>(&self, uri: UriSlice<'a>) -> Option<Box<dyn EditorNode>>;
}

impl EditorNode for arc::Archive {
    fn name(&self) -> &str {
        &self.name
    }

    fn uri(&self) -> UriSlice {}

    fn resolve_uri<'a>(&self, uri: UriSlice<'a>) -> Option<Box<dyn EditorNode>> {
        let curr = uri.head()?;
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(format!("{}//.", &self.name)).show(ui, |ui| {
            // An ARC file always has an empty top root, skip it
            let Some(root) = &self.root else {
                return;
            };

            let arc::ArcNode::Directory { children, .. } = root else {
                return;
            };

            for child in children {
                child.draw_tree(ui);
            }
        });
    }
}

impl EditorNode for arc::FileType {
    fn name(&self) -> &str {
        self.name()
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        match self {
            Self::Raw(_) => {
                ui.button(self.name());
            }
            Self::Brres(archive) => archive.draw_tree(ui),
        }
    }
}

impl EditorNode for arc::ArcNode {
    fn name(&self) -> &str {
        // Calls ArcNode::name
        self.name()
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        match self {
            arc::ArcNode::Directory { name, children } => {
                egui::CollapsingHeader::new(name).show(ui, |ui| {
                    for child in children {
                        child.draw_tree(ui);
                    }
                });
            }
            arc::ArcNode::File { data } => {
                // This file might have a further file tree of its own
                // (albeit not in ARC format)
                //
                // This for example happens in brres files.
                data.draw_tree(ui);
            }
        }
    }
}

impl EditorNode for arc::RawFile {
    fn name(&self) -> &str {
        &self.name
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        ui.button(format!("RAW: {}", self.name));
    }
}

impl EditorNode for brres::Archive {
    fn name(&self) -> &str {
        &self.name
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(&self.name).show(ui, |ui| {
            for dir in &self.directories {
                dir.draw_tree(ui);
            }
        });
    }
}

impl EditorNode for brres::Directory {
    fn name(&self) -> &str {
        &self.name
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(&self.name).show(ui, |ui| {
            for file in &self.files {
                file.draw_tree(ui);
            }
        });
    }
}

impl EditorNode for brres::File {
    fn name(&self) -> &str {
        &self.name
    }

    fn draw_tree(&self, ui: &mut egui::Ui) {
        ui.button(&self.name);
    }
}
