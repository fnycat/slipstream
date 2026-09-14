use std::{collections::HashMap, ffi::CStr, io::Cursor};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::{
    brres,
    encoding::{Deserialize, Encode, ReadArrayExt, WriteArrayExt},
    error::{CorruptionError, EncodingError, EncodingResult, IncorrectFormat, UnsupportedError},
};

/// Magic of an ARC file.
pub const ARC_MAGIC: u32 = 0x55AA382D;

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
        let magic = reader.read_u32::<BigEndian>()?;
        if magic != ARC_MAGIC {
            return Err(IncorrectFormat {
                expected_magic: ARC_MAGIC.to_be_bytes().to_vec(),
                found_magic: magic.to_be_bytes().to_vec(),
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

impl Encode for Header {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u32::<BigEndian>(ARC_MAGIC)?;
        writer.write_i32::<BigEndian>(self.node_offset)?;
        writer.write_i32::<BigEndian>(self.size)?;
        writer.write_i32::<BigEndian>(self.file_offset)?;
        writer.write_i32_array::<4, BigEndian>(self.reserved)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum RawNodeType {
    File,
    Directory,
}

impl Deserialize for RawNodeType {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let b = reader.read_u8()?;
        Self::try_from(b)
    }
}

impl Encode for RawNodeType {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl TryFrom<u8> for RawNodeType {
    type Error = EncodingError;

    fn try_from(value: u8) -> EncodingResult<Self> {
        Ok(match value {
            0 => RawNodeType::File,
            1 => RawNodeType::Directory,
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

/// Raw ARC node directly from the ARC file.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RawNode {
    /// The type this node is.
    pub ty: RawNodeType,
    /// Offset into the string pool for the file name.
    pub pool_offset: u32,
    /// Data content depends on the node type:
    /// - File: Offset of begin of data.
    /// - Directory: Index of the parent directory.
    pub data1: u32,
    /// Data content depends on the node type:
    /// - File: Size of data
    /// - Directory: Index of the first node that is not part of this directory.
    pub data2: u32,
}

impl RawNode {
    pub fn get_name_from_pool<'pool>(
        &self,
        string_pool: &'pool [u8],
    ) -> EncodingResult<&'pool str> {
        let null_pos = string_pool[self.pool_offset as usize..]
            .iter()
            .position(|&b| b == 0x00)
            .unwrap_or(string_pool.len() - self.pool_offset as usize);

        Ok(str::from_utf8(
            &string_pool[self.pool_offset as usize..self.pool_offset as usize + null_pos],
        )?)
    }
}

impl Encode for RawNode {
    fn encode_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()> {
        self.ty.encode_into(writer)?;
        writer.write_u24::<BigEndian>(self.pool_offset)?;
        writer.write_u32::<BigEndian>(self.data1)?;
        writer.write_u32::<BigEndian>(self.data2)?;

        Ok(())
    }
}

impl Deserialize for RawNode {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let ty = RawNodeType::deserialize(reader)?;
        let pool_offset = reader.read_u24::<BigEndian>()?;
        let data1 = reader.read_u32::<BigEndian>()?;
        let data2 = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            ty,
            pool_offset,
            data1,
            data2,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Brres(brres::Archive),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArcNode {
    File {
        name: String,
        data: FileType,
    },
    Directory {
        name: String,
        children: Vec<ArcNode>,
    },
}

impl ArcNode {
    pub fn name(&self) -> &str {
        match self {
            Self::File { name, .. } => name,
            Self::Directory { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Archive {
    pub root: Option<ArcNode>,
}

impl Archive {
    /// Deserializes all folders in this ARC archive.
    pub fn read_filetree<B: AsRef<[u8]>>(data: B) -> EncodingResult<Self> {
        let mut cursor = Cursor::new(data.as_ref());
        Self::deserialize(&mut cursor)
    }

    fn deserialize_file_contents(buffer: &[u8]) -> EncodingResult<FileType> {
        if &buffer[..4] != brres::BRRES_MAGIC {
            tracing::error!("unimplemented file format (not BRRES)");
            return Ok(FileType::Raw(buffer.to_owned()));
        }

        let mut cursor = Cursor::new(buffer);
        let archive = brres::Archive::deserialize(&mut cursor)?;

        Ok(FileType::Brres(archive))
    }

    fn parse_directory_node(
        nodes: &[RawNode],
        cursor: &mut usize,
        buffer: &[u8],
        string_pool: &[u8],
    ) -> EncodingResult<Option<ArcNode>> {
        let raw_dir = &nodes[*cursor];
        let end_idx = raw_dir.data2 as usize;

        *cursor += 1;

        let mut children = Vec::new();
        while *cursor < end_idx && *cursor < nodes.len() {
            let curr_node = &nodes[*cursor];

            if curr_node.ty == RawNodeType::Directory {
                if let Some(child_dir) =
                    Self::parse_directory_node(nodes, cursor, buffer, string_pool)?
                {
                    children.push(child_dir);
                }
            } else {
                let data_start = curr_node.data1 as usize;
                let data_end = data_start + curr_node.data2 as usize;
                let data = &buffer[data_start..data_end];

                let name = curr_node.get_name_from_pool(string_pool)?.to_owned();
                tracing::trace!("Reading `{name}` contents");

                children.push(ArcNode::File {
                    name,
                    data: Self::deserialize_file_contents(data)?,
                });

                *cursor += 1;
            }
        }

        Ok(Some(ArcNode::Directory {
            name: raw_dir.get_name_from_pool(string_pool)?.to_owned(),
            children,
        }))
    }

    /// Constructs the file tree for this archive and deserializes the inner files.
    fn parse_arc_tree(
        nodes: &[RawNode],
        buffer: &[u8],
        string_pool: &[u8],
    ) -> EncodingResult<Option<ArcNode>> {
        if nodes.is_empty() {
            return Ok(None);
        }

        let mut cursor = 0;
        Self::parse_directory_node(nodes, &mut cursor, buffer, string_pool)
    }
}

impl Deserialize for Archive {
    fn deserialize(reader: &mut Cursor<&[u8]>) -> EncodingResult<Self> {
        let header = Header::deserialize(reader)?;
        let root_node = RawNode::deserialize(reader)?;
        let node_count = root_node.data2;
        tracing::trace!("Reading {node_count} ARC nodes");

        // The string pool starts right after the last node.
        let spool_start = header.node_offset as usize + ARC_NODE_SIZE * node_count as usize;
        let spool_end = header.node_offset as usize + header.size as usize;
        tracing::trace!("ARC string pool range is {spool_start}...{spool_end}");

        let spool = &reader.get_ref()[spool_start..spool_end];

        let mut raw_nodes = Vec::with_capacity(node_count as usize);
        raw_nodes.push(root_node);

        for _ in 1..node_count {
            let node = RawNode::deserialize(reader)?;
            raw_nodes.push(node);
        }

        tracing::trace!("Loading ARC node names and content...");

        let nodes = Self::parse_arc_tree(&raw_nodes, reader.get_ref(), spool)?;

        tracing::trace!("Parsed file tree of {node_count} nodes");

        debug_assert_eq!(
            reader.position(),
            spool_start as u64,
            "not all bytes of the arc file were read"
        );

        tracing::trace!("Successfully loaded node names and data pointers");

        Ok(Self { root: nodes })
    }
}
