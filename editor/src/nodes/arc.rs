use std::io::Cursor;
use std::rc::Rc;

use szslib::{
    arc::{self, ArcNode, FileData},
    lazy::Lazy,
};

use crate::{
    nodes::{
        brres::{VirtualBrresNode, VirtualRawNode},
        lazy::LazyVirtualNode,
    },
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
    Content {
        name: String,
        content: Rc<dyn VirtualNode>,
    },
}

impl VirtualNode for VirtualArcNode {
    fn label(&self) -> &str {
        match self {
            Self::Directory { name, .. } => name,
            Self::Content { name, .. } => name,
        }
    }

    fn is_directory(&self) -> bool {
        match self {
            Self::Directory { .. } => true,
            Self::Content { content, .. } => content.is_directory(),
        }
    }

    fn children(&self) -> &[Rc<dyn VirtualNode>] {
        match self {
            Self::Directory { children, .. } => children,
            Self::Content { content, .. } => content.children(),
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
            ArcNode::File { name, data } => {
                tracing::trace!("Creating virtual file node `{name}`");

                let lazy_node = Rc::new(LazyVirtualNode::from_lazy(data));
                VirtualArcNode::Content {
                    name,
                    content: lazy_node,
                }
            }
        })
    }
}

/// Deserializes an ARC file.
pub fn deserialize_arc_root_virtual(
    reader: &mut Cursor<&[u8]>,
    file_cache: &mut FileCache,
    name: String,
) -> eyre::Result<Rc<dyn VirtualNode>> {
    let archive = arc::Archive::deserialize(reader, name)?;
    let root = archive
        .root
        .ok_or_else(|| eyre::eyre!("archive is empty"))?;

    Ok(Rc::new(VirtualArcNode::from_physical_node(
        reader, file_cache, root,
    )?))
}
