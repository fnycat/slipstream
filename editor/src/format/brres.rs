use byteorder::{BigEndian, ReadBytesExt};

use crate::error::{
    CorruptionError, EditorError, EditorResult, IncorrectFormat, RangeError, UnsupportedError,
};
use crate::node::defer::Deferred;
use crate::node::node::{VirtualNode, VirtualNodeBody, VirtualNodeKind};
use crate::node::refs::{VirtualNodeId, VirtualNodeMap};
use crate::{
    format::{
        chr0::Chr0Subfile,
        encoding::{Deserialize, ReadArrayExt, ReadStringExt},
        mdl0::{self, MDL0_MAGIC},
    },
    panes::inspector::raw::Raw,
    shared::util::RefCursor,
};

pub const BRRES_MAGIC: [u8; 4] = [0x62, 0x72, 0x65, 0x73];
const LE_BOM: [u8; 2] = [0xFF, 0xFE];
const BE_BOM: [u8; 2] = [0xFE, 0xFF];

/// Returns the amount of sections a subfile has, which depends on the subfile type and version.
///
/// This info comes from [`BRRES Subfiles (File Format)`](https://mkwiiki.org/wiki/BRRES_Subfiles_(File_Format))
pub fn get_section_count(ty: SubfileType, version: u32) -> EditorResult<usize> {
    Ok(match ty {
        SubfileType::Root => 0,
        SubfileType::Mdl0 => match version {
            8 => 11,
            11 => 14,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid MDL0 version: {version} (must be 8, 11)"),
                    ..Default::default()
                }
                .into());
            }
        },
        SubfileType::Chr0 => match version {
            // 3 => 1,
            3 => {
                return Err(UnsupportedError {
                    reason: "CHR0 version 3".to_owned(),
                    ..Default::default()
                }
                .into());
            }
            5 => 2,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid CHR0 version: {version} (must be 3, 5)"),
                    ..Default::default()
                }
                .into());
            }
        },
        SubfileType::Pat0 => match version {
            4 => 6,
            _ => {
                return Err(CorruptionError {
                    reason: format!("invalid PAT0 version: {version} (must be 4)"),
                    ..Default::default()
                }
                .into());
            }
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Header {
    pub size: u32,
    pub root_offset: u16,
    pub section_count: u16,
}

impl Deserialize for Header {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != BRRES_MAGIC {
            return Err(IncorrectFormat {
                expected_magic: BRRES_MAGIC.to_vec(),
                found_magic: magic.to_vec(),
                location: Some(reader.position()),
            }
            .into());
        }

        let is_be = match reader.read_u8_array::<2>()? {
            BE_BOM => true,
            LE_BOM => false,
            bom => {
                return Err(CorruptionError {
                    reason: format!(
                        "byte order mark is incorrect, expected FEFF or FFFE, found {bom:x?}"
                    ),
                    ..Default::default()
                }
                .into());
            }
        };

        if !is_be {
            return Err(UnsupportedError {
                reason: "little endian brres files are not supported".to_owned(),
                ..Default::default()
            }
            .into());
        }

        let _padding = reader.read_u16::<BigEndian>()?;
        let size = reader.read_u32::<BigEndian>()?;
        let root_offset = reader.read_u16::<BigEndian>()?;
        let section_count = reader.read_u16::<BigEndian>()?;

        Ok(Self {
            root_offset,
            size,
            section_count,
        })
    }
}

pub trait Subfile {
    const MAGIC: [u8; 4];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootSubfile {
    pub size: u32,
}

impl Deserialize for RootSubfile {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let magic = reader.read_u8_array::<4>()?;
        if magic != Self::MAGIC {
            return Err(IncorrectFormat {
                expected_magic: Self::MAGIC.to_vec(),
                found_magic: magic.to_vec(),
                location: Some(reader.position()),
            }
            .into());
        }

        Ok(RootSubfile {
            size: reader.read_u32::<BigEndian>()?,
        })
    }
}

impl Subfile for RootSubfile {
    const MAGIC: [u8; 4] = [0x72, 0x6f, 0x6f, 0x74]; // "root"
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubfileHeader {
    /// Start position of this header. This is used to compute subfile section positions using their
    /// offsets.
    pub header_start: u32,
    /// Length of this subfile.
    pub subfile_length: u32,
    /// Version of ths subfile. For MDL0 this is either 8 or 11.
    pub subfile_version: u32,
    /// The offset to the outer BRRES file.
    pub brres_offset: i32,
    /// Offsets within this BRRES file. The number of offsets is implied by the version.
    /// The number can be obtained using the [`get_section_count`] function.
    pub offsets: Vec<i32>,
    /// String offset to the name of this subfile.
    /// This offset is relative to [`header_start`](Self::header_start).
    ///
    /// Note that the offset points to the start of the string data, the length prefix is 4 bytes ahead of it.
    pub name_offset: i32,
}

impl SubfileHeader {
    pub fn deserialize(reader: &mut RefCursor<[u8]>, ty: SubfileType) -> EditorResult<Self> {
        let header_start = reader.position() as u32 - 4; // Subtract 4 for magic.
        let subfile_length = reader.read_u32::<BigEndian>()?;
        let subfile_version = reader.read_u32::<BigEndian>()?;
        let brres_offset = reader.read_i32::<BigEndian>()?;

        let section_count = get_section_count(ty, subfile_version)?;

        let mut offsets = Vec::with_capacity(section_count);
        for _ in 0..section_count {
            offsets.push(reader.read_i32::<BigEndian>()?);
        }

        let name_offset = reader.read_i32::<BigEndian>()?;

        Ok(Self {
            header_start,
            subfile_length,
            subfile_version,
            brres_offset,
            offsets,
            name_offset,
        })
    }

    /// Obtains the starting index of the specified section.
    pub fn get_section_start(&self, section_index: usize) -> EditorResult<u32> {
        let offset = *self.offsets.get(section_index).ok_or_else(|| {
            EditorError::from(RangeError {
                requested: section_index as u64,
                range: 0..self.offsets.len() as u64,
                ..Default::default()
            })
        })?;

        Ok((self.header_start as i32 + offset) as u32)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubfileType {
    Root,
    Mdl0,
    Chr0,
    Pat0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroupHeader {
    pub length: u32,
    pub number: u32,
}

impl Deserialize for IndexGroupHeader {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        Ok(Self {
            length: reader.read_u32::<BigEndian>()?,
            number: reader.read_u32::<BigEndian>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroupEntry {
    pub entry_id: u16,
    pub flag: u16,
    pub left_index: u16,
    pub right_index: u16,
    pub name_pointer: u32,
    pub data_pointer: u32,
}

impl Deserialize for IndexGroupEntry {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let entry_id = reader.read_u16::<BigEndian>()?;
        let flag = reader.read_u16::<BigEndian>()?;
        let left_index = reader.read_u16::<BigEndian>()?;
        let right_index = reader.read_u16::<BigEndian>()?;
        let name_pointer = reader.read_u32::<BigEndian>()?;
        let data_pointer = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            entry_id,
            flag,
            left_index,
            right_index,
            name_pointer,
            data_pointer,
        })
    }
}

/// Describes locations of the subfiles in this BRRES file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexGroup {
    /// Index into the BRRES file where this group starts. Generally this is just right after the BRRES header.
    pub group_start: u32,
    /// The index group header.
    pub header: IndexGroupHeader,
    /// `header.number + 1` entries.
    ///
    /// The first entry is a dummy entry that has no name.
    pub entries: Vec<IndexGroupEntry>,
}

impl IndexGroup {
    /// Obtains the name of the given index group entry using its name pointer.
    ///
    /// `data` should be the entire BRRES file (including header).
    ///
    /// # Conditions
    /// - The given index group entry must be owned by the current index group.
    ///
    /// Violating these conditions will not cause unsoundness but will either cause a panic due to invalid
    /// UTF-8 or return incorrect strings.
    pub fn get_entry_name(
        &self,
        reader: &mut RefCursor<[u8]>,
        entry: &IndexGroupEntry,
    ) -> EditorResult<String> {
        if entry.name_pointer == 0 {
            return Ok(String::new()); // This entry has no name.
        }

        // Move cursor to name and then back after reading.
        let name_start = self.group_start + entry.name_pointer;
        reader.set_position(name_start as u64 - 4);

        let name = reader.read_u32_string::<BigEndian>()?;

        Ok(name)
    }

    pub fn get_entry_data_start(&self, entry: &IndexGroupEntry) -> u32 {
        self.group_start + entry.data_pointer
    }
}

impl Deserialize for IndexGroup {
    fn deserialize(reader: &mut RefCursor<[u8]>) -> EditorResult<Self> {
        let group_start = reader.position() as u32;
        let header = IndexGroupHeader::deserialize(reader)?;

        let mut entries = Vec::with_capacity(header.number as usize);
        for _ in 0..header.number + 1 {
            let entry = IndexGroupEntry::deserialize(reader)?;
            entries.push(entry);
        }

        debug_assert_eq!(
            reader.position() as u32 - group_start,
            header.length,
            "an incorrect number of index group entries was read"
        );

        Ok(Self {
            group_start,
            header,
            entries,
        })
    }
}

fn deserialize_subfile(
    reader: &mut RefCursor<[u8]>,
    parent_id: VirtualNodeId,
    node_map: &VirtualNodeMap,
    name: String,
) -> EditorResult<VirtualNodeId> {
    // Check magic
    let magic = reader.read_u8_array::<4>()?;

    match magic {
        MDL0_MAGIC => mdl0::deserialize_virtual(reader, parent_id, node_map, name),
        // Chr0Subfile::MAGIC => Chr0Subfile::deserialize_lazy(reader),
        _ => {
            let id = node_map.next_id();
            let node = VirtualNode::from(VirtualNode {
                label: String::from("TODO, UNPARSED FORMAT"),
                id,
                parent: Some(parent_id),
                kind: VirtualNodeKind::Unknown,
                body: Deferred::evaluated(VirtualNodeBody {
                    children: Vec::new(),
                    inspectable: Some(Box::new(Raw {
                        bytes: reader.clone(),
                    })),
                }),
            });

            node_map.insert(id, node);
            Ok(id)
        }
    }
}

pub fn deserialize_virtual(
    reader: &mut RefCursor<[u8]>,
    parent_id: Option<VirtualNodeId>,
    node_map: &VirtualNodeMap,
    name: String,
) -> EditorResult<VirtualNodeId> {
    let mut reader = reader.clone();
    let brres_id = node_map.next_id();

    let node_map2 = node_map.clone();

    let name2 = name.clone();
    let parse_brres = move |_data| {
        tracing::trace!("Triggered deferred parse of `{name2}`");

        let header = Header::deserialize(&mut reader)?;

        // Skip to root start
        reader.set_position(header.root_offset as u64);

        let _root = RootSubfile::deserialize(&mut reader)?;
        let root_index = IndexGroup::deserialize(&mut reader)?;

        let mut directories = Vec::with_capacity(root_index.entries.len());

        // Do not include root subfile.
        for dir in &root_index.entries[1..] {
            let dir_name = root_index.get_entry_name(&mut reader, dir)?.to_owned();
            let dir_id = node_map2.next_id();

            tracing::trace!(
                "Discovered folder `{dir_name}` at location {}",
                reader.position()
            );

            reader.set_position(root_index.get_entry_data_start(dir) as u64);

            let child_index = IndexGroup::deserialize(&mut reader)?;
            let mut subfiles = Vec::with_capacity(child_index.entries.len());

            // Skip root subfile
            for subfile in &child_index.entries[1..] {
                let subfile_name = child_index.get_entry_name(&mut reader, subfile)?.to_owned();

                tracing::trace!(
                    "Discovered file `{dir_name}/{subfile_name}` at location `{}`",
                    reader.position()
                );

                reader.set_position(child_index.get_entry_data_start(subfile) as u64);

                // Skip over unimplemented formats for testing for now
                {
                    let magic = &reader.as_remaining()[..4];
                    if magic != MDL0_MAGIC && magic != Chr0Subfile::MAGIC {
                        tracing::error!("SKIPPING {}", String::from_utf8_lossy(magic));
                        continue;
                    }
                }

                let file = tracing::trace_span!("deserialize_subfile", %dir_name, %subfile_name)
                    .in_scope(|| {
                        deserialize_subfile(&mut reader, dir_id, &node_map2, subfile_name)
                    })?;

                subfiles.push(file);
            }

            let node = VirtualNode::from(VirtualNode {
                label: dir_name,
                id: dir_id,
                parent: Some(brres_id),
                kind: VirtualNodeKind::BrresDirectory,
                body: Deferred::evaluated(VirtualNodeBody {
                    children: subfiles,
                    inspectable: None,
                }),
            });

            node_map2.insert(dir_id, node);
            directories.push(dir_id);
        }

        Ok(VirtualNodeBody {
            inspectable: None,
            children: directories,
        })
    };

    let node = VirtualNode::from(VirtualNode {
        label: name,
        id: brres_id,
        parent: parent_id,
        kind: VirtualNodeKind::BrresDirectory,
        body: Deferred::defer((), parse_brres)?,
    });

    node_map.insert(brres_id, node);
    Ok(brres_id)
}
