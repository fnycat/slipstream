use std::{collections::HashMap, io::Cursor, path::PathBuf, rc::Rc};

use szslib::{
    arc::{self, ArcNode},
    yaz0::{self, YAZ0_MAGIC},
};

use crate::pages::editor::{FileCache, ResourceId, VirtualNode};

/// Figures out the format of the given file and tries to deserialize it.
pub fn deserialize_unknown(
    name: String,
    file_cache: &mut FileCache,
    mut contents: Vec<u8>,
) -> eyre::Result<Rc<dyn VirtualNode>> {
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
        &arc::ARC_MAGIC => decode_virtual_arc(&mut cursor, file_cache, name)?,
        _ => eyre::bail!(
            "unknown or unsupported file magic: `{}`",
            String::from_utf8_lossy(magic)
        ),
    };

    Ok(contents)
}

enum VirtualArcNode {
    Directory {
        name: String,
        children: Vec<Rc<dyn VirtualNode>>,
    },
    File {
        name: String,
        /// Points to the file contents in the file cache.
        ///
        /// Each file automatically gets assigned a resource ID,
        /// but its resource might not exist yet until it is accessed.
        content: ResourceId,
    },
}

impl VirtualNode for VirtualArcNode {
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

impl VirtualArcNode {
    pub fn from_physical_node(file_cache: &mut FileCache, node: ArcNode) -> Self {
        match node {
            ArcNode::Directory { name, children } => {
                let mut virtual_children = Vec::with_capacity(children.len());
                for child in children {
                    let virtual_child = Rc::new(Self::from_physical_node(file_cache, child));
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
                tracing::trace!("Assigned resource ID {resource_id} to `{name}`");

                VirtualArcNode::File {
                    name,
                    content: resource_id,
                }
            }
        }
    }
}

fn decode_virtual_arc(
    reader: &mut Cursor<&[u8]>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    let arc = arc::Archive::deserialize(reader, name)?;
    let root_node = arc
        .root
        .ok_or_else(|| eyre::eyre!("root node does not exist"))?;

    Ok(Rc::new(VirtualArcNode::from_physical_node(
        file_cache, root_node,
    )))
}
