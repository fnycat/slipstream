use std::{any::Any, collections::HashMap, ffi::CStr, io::Cursor, rc::Rc};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::pages::editor::inspector::raw::Raw;
use crate::r#virtual::defer::Deferred;
use crate::r#virtual::refs::{VirtualRefCache, VirtualRefCacheExt, VirtualRefCacheMap};
use crate::r#virtual::node::{VirtualNodeContent, VirtualNodeRef};
use crate::{
    format::{
        brres::{self, BRRES_MAGIC},
        encoding::{Deserialize, ReadArrayExt, ReadStringExt, Serialize, WriteArrayExt},
    },
    shared::util::RefCursor,
};
use crate::error::{CorruptionError, EditorError, EditorResult, IncorrectFormat, UnsupportedError};
use crate::r#virtual::node::{Inspectable, VirtualNode, VirtualNodeKind};

/// Magic of an ARC file.
pub const ARC_MAGIC: [u8; 4] = [0x55, 0xAA, 0x38, 0x2D];

/// See [`Custom Mario Kart Wiiki`](https://mkwiiki.org/wiki/ARC_(File_Format)) for more info.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    /// Offset to the first node in the archive.
    pub node_offset: i32,
    /// Size of all nodes including the string table.
    pub size: i32,
    /// File offset of data.
    pub file_offset: i32,
    /// Reserved.
    pub reserved: [i32; 4],
}

impl Deserialize for Header {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
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
        let reserved = reader.read_i32_array::<4, BigEndian>()?;

        Ok(Self {
            node_offset,
            size,
            file_offset,
            reserved,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum NodeType {
    File,
    Directory,
}

impl Deserialize for NodeType {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let b = reader.read_u8()?;
        Self::try_from(b)
    }
}

impl TryFrom<u8> for NodeType {
    type Error = EditorError;

    fn try_from(value: u8) -> EditorResult<Self> {
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

/// Exact size of a single ARC node.
const ARC_NODE_SIZE: usize = 0x0c;

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
    ) -> EditorResult<Self> {
        let ty = NodeType::deserialize(reader)?;
        let name_offset = reader.read_u24::<BigEndian>()?;
        let data1 = reader.read_u32::<BigEndian>()?;
        let data2 = reader.read_u32::<BigEndian>()?;

        string_pool.set_position(name_offset as u64);

        let name = string_pool.read_null_string::<BigEndian>()?;

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
                data.set_tail();

                NodeContent::File { data }
            }
        };

        Ok(Self { name, data })
    }
}

fn parse_leaf_node(
    reader: &mut RefCursor<[u8]>,
    ref_cache: &VirtualRefCache,
    name: String,
) -> EditorResult<VirtualNodeRef> {
    let magic: [u8; 4] = reader.read_u8_array()?;
    reader.set_position(reader.position() - 4);

    match magic {
        ARC_MAGIC => deserialize_virtual(reader, ref_cache, name),
        BRRES_MAGIC => brres::deserialize_virtual(reader, ref_cache, name),
        _ => {
            let id = ref_cache.next_id();
            let node = VirtualNodeRef::from(VirtualNode {
                label: name,
                id,
                kind: VirtualNodeKind::Terminal,
                content: Deferred::evaluated(VirtualNodeContent {
                    inspectable: Some(Box::new(Raw {
                        bytes: reader.clone(),
                    })),
                    children: Vec::new(),
                }),
            });

            ref_cache.insert(id, Rc::downgrade(&node));
            Ok(node)
        }
    }
}

fn parse_directory_tree(
    node_list: &mut [Node],
    ref_cache: &VirtualRefCache,
    label: String,
    cursor: &mut usize,
) -> EditorResult<VirtualNodeRef> {
    let &NodeContent::Directory { skip_node, .. } = &node_list[*cursor].data else {
        return Err(CorruptionError {
            reason: "expected directory at root, found file instead".to_owned(),
            ..Default::default()
        }
        .into());
    };

    *cursor += 1;

    let mut children = Vec::new();
    while *cursor < skip_node as usize && *cursor < node_list.len() {
        let curr_node = &mut node_list[*cursor];

        let name = std::mem::take(&mut curr_node.name);
        match &mut curr_node.data {
            NodeContent::Directory { .. } => {
                let child = parse_directory_tree(node_list, ref_cache, name, cursor)?;
                children.push(child);
            }
            NodeContent::File { data } => {
                let sections = parse_leaf_node(data, ref_cache, name)?;
                children.push(sections);

                *cursor += 1;
            }
        }
    }

    let id = ref_cache.next_id();
    let node = VirtualNodeRef::from(VirtualNode {
        label,
        id,
        kind: VirtualNodeKind::Container,
        content: Deferred::evaluated(VirtualNodeContent {
            inspectable: None,
            children,
        }),
    });

    ref_cache.insert(id, Rc::downgrade(&node));
    Ok(node)
}

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    ref_cache: &VirtualRefCache,
    name: String,
) -> EditorResult<VirtualNodeRef> {
    tracing::trace!("Parsing ARC file `{name}`");

    let header = Header::deserialize(reader)?;

    let ty = NodeType::deserialize(reader)?;
    if ty != NodeType::Directory {
        return Err(CorruptionError {
            reason: format!("expected directory at root, found file"),
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
        pool.set_tail();

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
    let ret = parse_directory_tree(&mut nodes, ref_cache, name, &mut cursor)?;
    tracing::trace!("Constructed directory tree successfully");
    Ok(ret)
}
