use std::io::Cursor;
use std::rc::Rc;

use szslib::brres;
use szslib::{chr0::Chr0Subfile, mdl0::Mdl0Subfile};

use crate::pages::editor::{FileCache, ResourceId, VirtualNode};

#[derive(Debug)]
pub enum VirtualBrresNode {
    Directory {
        name: String,
        children: Vec<Rc<dyn VirtualNode>>,
    },
    File {
        name: String,
        content: ResourceId,
    },
}

impl VirtualBrresNode {
    fn from_mdl0_node(name: String, mdl0: Mdl0Subfile) -> eyre::Result<Self> {
        let mut children = Vec::new();

        Ok(VirtualBrresNode::Directory { name, children })
    }

    fn from_chr0_node(name: String, chr0: Chr0Subfile) -> eyre::Result<Self> {
        let mut children = Vec::new();

        Ok(VirtualBrresNode::Directory { name, children })
    }

    fn from_subfile_node(file: brres::File) -> eyre::Result<Self> {
        match file.content {
            brres::SubfileData::Mdl0(mdl0) => Self::from_mdl0_node(file.name, mdl0),
            brres::SubfileData::Chr0(chr0) => Self::from_chr0_node(file.name, chr0),
            _ => todo!(),
        }
    }

    pub fn from_physical_node(archive: brres::Archive, name: String) -> eyre::Result<Self> {
        let mut directories = Vec::with_capacity(archive.directories.len() as usize);
        for directory in archive.directories {
            // Examples of directories are `3DModels(NW4R)`.

            let mut files = Vec::with_capacity(directory.files.len() as usize);
            for file in directory.files {
                // These are the MDL0, CHR0, PAT0, files, etc

                let virtual_subfile = Rc::new(Self::from_subfile_node(file)?);
                files.push(virtual_subfile as Rc<dyn VirtualNode>);
            }

            let virtual_directory = Rc::new(VirtualBrresNode::Directory {
                name: directory.name,
                children: files,
            });

            directories.push(virtual_directory as Rc<dyn VirtualNode>);
        }

        Ok(Self::Directory {
            name,
            children: directories,
        })
    }
}

impl VirtualNode for VirtualBrresNode {
    fn label(&self) -> &str {
        match self {
            Self::Directory { name, .. } => name,
            Self::File { name, .. } => name,
        }
    }

    fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }

    fn children(&self) -> &[Rc<dyn VirtualNode>] {
        match self {
            Self::Directory { children, .. } => children,
            Self::File { .. } => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub struct VirtualRawNode {
    pub name: String,
    pub content: ResourceId,
}

impl VirtualNode for VirtualRawNode {
    fn label(&self) -> &str {
        &self.name
    }

    fn is_directory(&self) -> bool {
        false
    }

    fn children(&self) -> &[Rc<dyn VirtualNode>] {
        unimplemented!()
    }
}

/// Deserializes a BRRES file.
pub fn deserialize_virtual_root_brres(
    reader: &mut Cursor<&[u8]>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    todo!("root brres");
}
