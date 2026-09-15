use std::{collections::HashMap, ffi::CStr, io::Cursor};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    format::{
        brres,
        encoding::{Deserialize, ReadArrayExt, ReadStringExt, Serialize, WriteArrayExt},
        error::{
            CorruptionError, EncodingError, EncodingResult, IncorrectFormat, UnsupportedError,
        },
    },
    r#virtual::{ResourceId, VirtualNode, VirtualNodeKind},
};

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
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
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
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let b = reader.read_u8()?;
        Self::try_from(b)
    }
}

impl TryFrom<u8> for NodeType {
    type Error = EncodingError;

    fn try_from(value: u8) -> EncodingResult<Self> {
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
    File { data: Vec<u8> },
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
    pub fn deserialize(reader: &mut Cursor<&[u8]>, string_pool: &[u8]) -> EncodingResult<Self> {
        let ty = NodeType::deserialize(reader)?;
        let name_offset = reader.read_u24::<BigEndian>()?;
        let data1 = reader.read_u32::<BigEndian>()?;
        let data2 = reader.read_u32::<BigEndian>()?;

        let mut name_cursor = Cursor::new(string_pool);
        name_cursor.set_position(name_offset as u64);

        let name = name_cursor.read_null_string::<BigEndian>()?;
        let data = match ty {
            NodeType::Directory => NodeContent::Directory {
                parent: data1,
                skip_node: data2,
            },
            NodeType::File => {
                let data_start = data1;
                let data_end = data_start + data2;

                let data = reader.get_ref()[data_start as usize..data_end as usize].to_vec();
                NodeContent::File { data }
            }
        };

        Ok(Self { name, data })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Archive {
    pub name: String,
    pub root: Node,
}

fn parse_directory_tree(
    node_list: &[Node],
    label: String,
    cursor: &mut usize,
) -> EncodingResult<VirtualNode> {
    let &NodeContent::Directory { parent, skip_node } = &node_list[*cursor].data else {
        todo!();
    };

    *cursor += 1;

    let mut children = Vec::new();
    while *cursor < skip_node as usize && *cursor < node_list.len() {
        let curr_node = &node_list[*cursor];

        if curr_node.data.is_directory() {
            let child = parse_directory_tree(node_list, curr_node.name.clone(), cursor)?;
            children.push(child);
        } else {
            let child = VirtualNode {
                label: curr_node.name.clone(),
                kind: VirtualNodeKind::File {
                    format_tag: "hello",
                    cache_id: ResourceId::ZERO,
                },
                children: Vec::new(),
            };
            children.push(child);

            *cursor += 1;
        }
    }

    Ok(VirtualNode {
        label,
        kind: VirtualNodeKind::Directory,
        children,
    })
}

pub fn deserialize_virtual(
    reader: &mut Cursor<&[u8]>,
    name: String,
) -> EncodingResult<VirtualNode> {
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

    let string_pool = {
        let start = header.node_offset as usize + ARC_NODE_SIZE * node_count as usize;
        let end = header.node_offset as usize + header.size as usize;

        tracing::trace!("ARC string pool is in range {start}..{end}");

        &reader.get_ref()[start..end]
    };

    let mut nodes = Vec::with_capacity(node_count as usize);
    nodes.push(Node {
        name: "<null>".to_owned(),
        data: NodeContent::Directory {
            parent: _data1,
            skip_node: node_count,
        },
    });

    for _ in 1..node_count {
        let node = Node::deserialize(reader, string_pool)?;
        nodes.push(node);
    }

    let mut cursor = 0;
    parse_directory_tree(&nodes, name, &mut cursor)
}
