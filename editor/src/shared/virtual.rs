use std::{collections::HashMap, io::Cursor, path::PathBuf, rc::Rc};

use szslib::{
    arc::{self, ArcNode, FileType},
    brres,
    yaz0::{self, YAZ0_MAGIC},
};

use crate::pages::editor::{FileCache, ResourceId, VirtualNode};

pub fn deserialize_maybe_compressed(
    mut reader: Cursor<Vec<u8>>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    // Is this file compressed?
    if &reader.get_ref()[..4] == YAZ0_MAGIC {
        // then decompress it.
        reader = Cursor::new(yaz0::decompress(reader.get_ref())?);
    }

    let mut reader = Cursor::new(reader.get_ref().as_slice());
    deserialize_unknown(&mut reader, file_cache, name)
}

/// Deserializes an uncompressed file.
///
/// This function works with OS level files, not files within archives.
pub fn deserialize_unknown(
    reader: &mut Cursor<&[u8]>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    let magic: &[u8; 4] = reader.get_ref()[..4]
        .try_into()
        .expect("array of size 4 does not have size 4?");

    let contents = match magic {
        &arc::ARC_MAGIC => deserialize_virtual_root_arc(reader, file_cache, name)?,
        &brres::BRRES_MAGIC => deserialize_virtual_root_brres(reader, file_cache, name)?,
        _ => eyre::bail!(
            "unknown or unsupported file magic: `{}`",
            String::from_utf8_lossy(magic)
        ),
    };

    Ok(contents)
}

enum VirtualArcNode {
    /// The node is an ARC directory
    Directory {
        name: String,
        children: Vec<Rc<dyn VirtualNode>>,
    },
    /// The node is an ARC file.
    ///
    /// These files may however have their own file tree.
    Content { content: Rc<dyn VirtualNode> },
}

impl VirtualNode for VirtualArcNode {
    fn label(&self) -> &str {
        match self {
            Self::Directory { name, .. } => name,
            Self::Content { content } => content.label(),
        }
    }

    fn is_directory(&self) -> bool {
        match self {
            Self::Directory { .. } => true,
            Self::Content { content } => content.is_directory(),
        }
    }

    fn children(&self) -> &[Rc<dyn VirtualNode>] {
        match self {
            Self::Directory { children, .. } => children,
            Self::Content { content } => content.children(),
        }
    }
}

impl VirtualArcNode {
    pub fn from_physical_node(
        reader: &mut Cursor<&[u8]>,
        file_cache: &mut FileCache,
        node: ArcNode,
    ) -> eyre::Result<Self> {
        Ok(match node {
            ArcNode::Directory { name, children } => {
                tracing::trace!("Creating virtual directory node `{name}`");

                let mut virtual_children = Vec::with_capacity(children.len());
                for child in children {
                    let virtual_child =
                        Rc::new(Self::from_physical_node(reader, file_cache, child)?);

                    virtual_children.push(virtual_child as Rc<dyn VirtualNode>);
                }

                VirtualArcNode::Directory {
                    name,
                    children: virtual_children,
                }
            }
            ArcNode::File { data } => {
                let name = data.name().to_owned();
                let resource_id = file_cache.next_id();

                tracing::trace!(
                    "Creating virtual file node `{name}`, with resource ID `{resource_id}`"
                );

                VirtualArcNode::Content {
                    content: from_physical_file_type(file_cache, data, name)?,
                }
            }
        })
    }
}

struct VirtualRawNode {
    name: String,
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

enum VirtualBrresNode {
    Directory {
        name: String,
        children: Vec<Rc<dyn VirtualNode>>,
    },
    File {
        name: String,
    },
}

impl VirtualBrresNode {
    pub fn from_physical_node(archive: brres::Archive) -> eyre::Result<Self> {
        let mut directories = Vec::with_capacity(archive.directories.len() as usize);
        for directory in archive.directories {
            let mut files = Vec::with_capacity(directory.files.len() as usize);
            for file in directory.files {
                let virtual_file = Rc::new(VirtualBrresNode::File { name: file.name });

                files.push(virtual_file as Rc<dyn VirtualNode>);
            }

            let virtual_directory = Rc::new(VirtualBrresNode::Directory {
                name: directory.name,
                children: files,
            });

            directories.push(virtual_directory as Rc<dyn VirtualNode>);
        }

        Ok(Self::Directory {
            name: archive.name,
            children: directories,
        })
    }
}

impl VirtualNode for VirtualBrresNode {
    fn label(&self) -> &str {
        match self {
            Self::Directory { name, .. } => name,
            Self::File { name } => name,
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

fn from_physical_file_type(
    file_cache: &mut FileCache,
    data: FileType,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    let virtual_node = match data {
        FileType::Brres(brres) => {
            let node = Rc::new(VirtualBrresNode::from_physical_node(brres)?);
            node as Rc<dyn VirtualNode>
        }
        FileType::Raw(raw) => Rc::new(VirtualRawNode { name: raw.name }),
    };

    Ok(virtual_node)
}

fn deserialize_virtual_root_brres(
    reader: &mut Cursor<&[u8]>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    todo!("root brres");
}

fn deserialize_virtual_root_arc(
    reader: &mut Cursor<&[u8]>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    let arc = arc::Archive::deserialize(reader, name)?;
    let root_node = arc
        .root
        .ok_or_else(|| eyre::eyre!("root node does not exist"))?;

    Ok(Rc::new(VirtualArcNode::from_physical_node(
        reader, file_cache, root_node,
    )?))
}
