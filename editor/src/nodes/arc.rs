use std::io::Cursor;
use std::rc::Rc;

use szslib::arc::{self, ArcNode, FileType};

use crate::{
    nodes::brres::{VirtualBrresNode, VirtualRawNode},
    pages::editor::{FileCache, VirtualNode},
};

/// A virtual node of an ARC file. This can either be a directory or some content.
///
/// `Content` may refer to either a standard file or a file with its own internal filesystem
/// (such as BRRES files)
#[derive(Debug)]
pub enum VirtualArcNode {
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

/// Converts the given ARC node file type into a virtual node.
pub fn from_physical_file_type(
    file_cache: &mut FileCache,
    data: FileType,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    let virtual_node = match data {
        FileType::Brres(brres) => {
            let node = Rc::new(VirtualBrresNode::from_physical_node(brres)?);
            node as Rc<dyn VirtualNode>
        }
        FileType::Unknown(raw) => Rc::new(VirtualRawNode {
            name: raw.name,
            content: file_cache.insert(raw.data),
        }),
    };

    Ok(virtual_node)
}

/// Deserializes an ARC file.
pub fn deserialize_virtual_root_arc(
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
