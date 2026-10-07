use std::io::Write;
use std::ops::ControlFlow;
use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_shared::cursor::{MutCursor, RefCursor};
use slipstream_shared::error::{
    CorruptionError, IncorrectFormat, SlipstreamError, SlipstreamResult,
};

use crate::brres::{self, BRRES_MAGIC};
use crate::deferred_pass::{
    DEFER_PLACEHOLDER, DEFER_PLACEHOLDER24, DeferredPass, DeferredString, StringPool,
};
use crate::encoding::{ReadArrayExt, ReadStringExt, WriteArrayExt};
use crate::node::arena::{IrArena, IrNodeDescriptor, IrNodeKey};
use crate::node::node::{ContentSlot, IrNode, IrNodeType};
use crate::visitor::{
    Visitable, Visitor, VisitorContext, VisitorContextMut, VisitorContextNode,
    VisitorContextNodeMut,
};

/// Magic of an ARC file.
pub const ARC_MAGIC: [u8; 4] = [0x55, 0xAA, 0x38, 0x2D];
pub const ARC_NODE_SIZE: usize = 0xC;

/// See [`Custom Mario Kart Wiiki`](https://mkwiiki.org/wiki/ARC_(File_Format)) for more info.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    /// Offset to the first node in the archive.
    pub node_offset: i32,
    /// Size of all nodes including the string table.
    pub size: i32,
    /// File offset of data.
    pub file_offset: i32,
}

impl Header {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != ARC_MAGIC {
            return Err(IncorrectFormat {
                expected_magic: ARC_MAGIC.to_vec(),
                found_magic: magic.to_vec(),
                location: Some(reader.position()),
            }
            .into());
        }

        let node_offset = reader.read_i32::<BigEndian>()?;
        let size = reader.read_i32::<BigEndian>()?;
        let file_offset = reader.read_i32::<BigEndian>()?;
        let _reserved = reader.read_i32_array::<4, BigEndian>()?;

        Ok(Self {
            node_offset,
            size,
            file_offset,
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u8_array(ARC_MAGIC)?;
        writer.write_i32::<BigEndian>(self.node_offset)?;
        writer.write_i32::<BigEndian>(self.size)?;
        writer.write_i32::<BigEndian>(self.file_offset)?;
        writer.write_i32_array::<_, BigEndian>([0; 4])?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum NodeType {
    File,
    Directory,
}

impl NodeType {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let b = reader.read_u8()?;
        Self::try_from(b)
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl TryFrom<u8> for NodeType {
    type Error = SlipstreamError;

    fn try_from(value: u8) -> SlipstreamResult<Self> {
        Ok(match value {
            0 => NodeType::File,
            1 => NodeType::Directory,
            v => {
                return Err(CorruptionError {
                    reason: format!(
                        "arc node type is expected to be either 0 (file) or 1 (directory), got {v}"
                    ),
                    ..Default::default()
                }
                .into());
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeContent {
    File { data: RefCursor<[u8]> },
    Directory { parent: u32, skip_node: u32 },
}

impl NodeContent {
    pub fn is_directory(&self) -> bool {
        matches!(self, Self::Directory { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub data: NodeContent,
}

impl Node {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        string_pool: &mut RefCursor<[u8]>,
    ) -> SlipstreamResult<Self> {
        let ty = NodeType::deserialize(reader)?;
        let name_offset = reader.read_u24::<BigEndian>()?;
        let data1 = reader.read_u32::<BigEndian>()?;
        let data2 = reader.read_u32::<BigEndian>()?;

        let spool_start = string_pool.position();
        string_pool.set_position(spool_start + name_offset as u64);

        let name = string_pool.read_null_string::<BigEndian>()?;
        string_pool.set_position(spool_start);

        tracing::trace!("Discovered node `{name}`");

        let data = match ty {
            NodeType::Directory => NodeContent::Directory {
                parent: data1,
                skip_node: data2,
            },
            NodeType::File => {
                let data_start = data1;

                let mut data = reader.clone();
                data.set_position(data_start as u64);

                NodeContent::File { data }
            }
        };

        Ok(Self { name, data })
    }
}

pub struct UnknownFile {
    pub reader: RefCursor<[u8]>,
}

impl Visitable for UnknownFile {
    fn accept(&self, node: VisitorContextNode<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_unknown(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut<'_>,
        visitor: &mut dyn Visitor,
    ) -> ControlFlow<()> {
        visitor.visit_unknown_mut(VisitorContextMut::new(node, self))
    }
}

#[tracing::instrument(skip_all, fields(label))]
fn parse_leaf_node(
    reader: &mut RefCursor<[u8]>,
    parent_id: IrNodeKey,
    arena: &IrArena,
    label: String,
) -> SlipstreamResult<IrNodeKey> {
    let magic: [u8; 4] = reader.read_u8_array()?;
    reader.set_position(reader.position() - 4);

    match magic {
        ARC_MAGIC => deserialize(reader, Some(parent_id), arena, label),
        BRRES_MAGIC => brres::deserialize(reader, Some(parent_id), arena, label),
        _ => {
            let key = arena.insert(IrNodeDescriptor {
                label,
                ty: IrNodeType::Unknown,
                parent: Some(parent_id),
                children: Vec::new(),
                contents: ContentSlot::eager(Box::new(UnknownFile {
                    reader: reader.clone(),
                })),
            });

            Ok(key)
        }
    }
}

pub struct ArcDirectory {
    pub uncompressed_size: i32,
}

impl Visitable for ArcDirectory {
    fn accept(&self, node: VisitorContextNode<'_>, visitor: &mut dyn Visitor) -> ControlFlow<()> {
        visitor.visit_arc(VisitorContext::new(node, self))
    }

    fn accept_mut(
        &mut self,
        node: VisitorContextNodeMut<'_>,
        visitor: &mut dyn Visitor,
    ) -> ControlFlow<()> {
        visitor.visit_arc_mut(VisitorContextMut::new(node, self))
    }
}

/// Constructs a tree of the directories in an ARC file.
///
/// ARC files store their nodes in a linear list, this function converts it into
/// a tree by resolving references.
#[tracing::instrument(skip_all, fields(label))]
fn construct_directory_tree(
    node_list: &mut [Node],
    parent: Option<IrNodeKey>,
    arena: &IrArena,
    label: String,
    is_root: bool,
    uncompressed_size: i32,
    cursor: &mut usize,
) -> SlipstreamResult<IrNodeKey> {
    let &NodeContent::Directory { skip_node, .. } = &node_list[*cursor].data else {
        return Err(CorruptionError {
            reason: "expected directory at root, found file instead".to_owned(),
            ..Default::default()
        }
        .into());
    };

    *cursor += 1;

    let key = arena.reserve_key();

    let mut children = Vec::new();
    while *cursor < skip_node as usize && *cursor < node_list.len() {
        let curr_node = &mut node_list[*cursor];

        let name = std::mem::take(&mut curr_node.name);
        match &mut curr_node.data {
            NodeContent::Directory { .. } => {
                let child = construct_directory_tree(
                    node_list,
                    Some(key),
                    arena,
                    name,
                    false,
                    uncompressed_size,
                    cursor,
                )?;
                children.push(child);
            }
            NodeContent::File { data } => {
                let sections = parse_leaf_node(data, key, arena, name)?;
                children.push(sections);

                *cursor += 1;
            }
        }
    }

    arena.insert_at(
        key,
        IrNodeDescriptor {
            label,
            ty: IrNodeType::ArcDirectory {
                is_root,
                empty: children.is_empty(),
            },
            parent,
            children,
            contents: ContentSlot::eager(Box::new(ArcDirectory { uncompressed_size })),
        },
    );

    Ok(key)
}

pub fn deserialize(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<IrNodeKey>,
    arena: &IrArena,
    name: String,
) -> SlipstreamResult<IrNodeKey> {
    tracing::trace!("Parsing ARC file `{name}`");

    let header = Header::deserialize(reader)?;

    let ty = NodeType::deserialize(reader)?;
    if ty != NodeType::Directory {
        return Err(CorruptionError {
            reason: String::from("expected directory at root, found file"),
            ..Default::default()
        }
        .into());
    }

    let _offset = reader.read_u24::<BigEndian>()?;
    let _data1 = reader.read_u32::<BigEndian>()?;
    let node_count = reader.read_u32::<BigEndian>()?;

    let mut string_pool = {
        let start = header.node_offset as i64 + ARC_NODE_SIZE as i64 * node_count as i64;
        let end = (header.node_offset + header.size) as u64;

        tracing::trace!("ARC string pool is in range {start}..{end}");

        let mut pool = reader.clone();
        pool.set_position(start as u64);

        pool
    };

    let mut nodes = Vec::with_capacity(node_count as usize);
    nodes.push(Node {
        name: "<null>".to_owned(),
        data: NodeContent::Directory {
            parent: _data1,
            skip_node: node_count,
        },
    });

    tracing::trace!("Reading {node_count} nodes");
    for _ in 1..node_count {
        let node = Node::deserialize(reader, &mut string_pool)?;
        nodes.push(node);
    }

    let mut cursor = 0;

    tracing::trace!("Constructing directory tree and parsing nodes...");
    let ret = construct_directory_tree(
        &mut nodes,
        parent_id,
        arena,
        name,
        true,
        header.size,
        &mut cursor,
    )?;
    tracing::trace!("Constructed directory tree successfully");
    Ok(ret)
}

pub fn serialize_inner(
    arena: &IrArena,
    node: IrNodeKey,
    writer: &mut MutCursor,
    string_pool: &mut StringPool,
    pass: &mut DeferredPass,
    id_counter: &mut u32,
    parent: u32,
) -> SlipstreamResult<()> {
    arena
        .inspect(node, |node| {
            let root_id = *id_counter;
            *id_counter += 1;

            // Skip over the root ARC.
            //             let arc_node = match node.ty {
            //                 IrNodeType::ArcDirectory { is_root: true, .. } => None,
            //                 IrNodeType::ArcDirectory { is_root: false, .. } => {
            //                     // NodeType::Directory.serialize(writer)?;
            //                     //
            //                     // pass.defer_arc_string(writer, node.label())?;
            //                     // writer.write_u32::<BigEndian>(parent)?;
            //                     //
            //                     // let skip_node_offset = writer.len();
            //                     // writer.write_u32::<BigEndian>()?;
            //                     todo!();
            //                 }
            //                 _ => {
            //                     // We encountered a file. The file itself will take over from here, so stop treating them like ARC nodes.
            //                     // Some(Node {
            //                     //     name: node.label.clone(),
            //                     //     data: NodeContent::File {
            //                     //         data: RefCursor::new(Arc::new([])),
            //                     //     },
            //                     // });
            //
            //                     return Ok(());
            //                 }
            //             };
            todo!();

            for child in &node.children {
                serialize_inner(
                    arena,
                    *child,
                    writer,
                    string_pool,
                    pass,
                    id_counter,
                    root_id,
                )?;
            }

            Ok::<_, SlipstreamError>(())
        })
        .transpose()?;

    Ok(())
}

pub fn serialize(
    arena: &IrArena,
    node: IrNodeKey,
    writer: &mut MutCursor,
    pass: &mut DeferredPass,
) -> SlipstreamResult<()> {
    tracing::trace!("Serializing ARC file");

    let mut node_count = 0;
    let mut string_pool = StringPool::new();

    serialize_inner(
        arena,
        node,
        writer,
        &mut string_pool,
        pass,
        &mut node_count,
        0,
    )?;

    let string_pool = string_pool.finish();
    let header_start = writer.len();
    let arc_size = node_count as usize * ARC_NODE_SIZE + string_pool.len();

    let header: Header = Header {
        node_offset: 0x20, // Header is 32 bytes long, first node starts directly after header.
        file_offset: (header_start + arc_size) as i32 + 1, // Files start directly after the string pool.
        size: 0x20 + arc_size as i32, // node_offset + node_count * node_size + string_pool_size
    };
    tracing::debug!("ARC header {header:?}");
    header.serialize(writer)?;

    // Root is always a directory.
    NodeType::Directory.serialize(writer)?;

    writer.write_all(&string_pool.into_inner())?;

    Ok(())
}
